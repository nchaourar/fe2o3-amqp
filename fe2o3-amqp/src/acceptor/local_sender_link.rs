//! Implements acceptor for a remote receiver link

use std::{marker::PhantomData, sync::Arc};

use fe2o3_amqp_types::{
    definitions::SequenceNo,
    messaging::{Source, Target},
    performatives::Attach,
    primitives::Symbol,
};
use parking_lot::RwLock;
use tokio::sync::{mpsc, Notify};

use crate::{
    endpoint::{InputHandle, LinkAttach, LinkExt},
    link::{
        sender::SenderInner,
        state::{LinkFlowState, LinkFlowStateInner, LinkState},
        LinkRelay, SenderAttachError, SenderLink,
    },
    session::SessionHandle,
    util::{Consumer, Producer},
    Sender,
};

use super::link::SharedLinkAcceptorFields;

/// An acceptor for a remote receiver link
///
/// the sender is considered to hold the authoritative version of the
/// source properties, the receiver is considered to hold the authoritative version of the target properties.
#[derive(Debug, Clone)]
pub(crate) struct LocalSenderLinkAcceptor<C, F>
where
    F: Fn(Source) -> Option<Source>,
{
    /// This MUST NOT be null if role is sender,
    /// and it is ignored if the role is receiver.
    /// See subsection 2.6.7.
    pub initial_delivery_count: SequenceNo,

    /// the extension capabilities the sender supports/desires
    pub source_capabilities: Option<Vec<C>>,

    pub on_dynamic_source: F,
}

fn reject_dynamic_source(_: Source) -> Option<Source> {
    None
}

impl<C> Default for LocalSenderLinkAcceptor<C, fn(Source) -> Option<Source>>
where
    C: From<Symbol>,
{
    fn default() -> Self {
        Self {
            initial_delivery_count: 0,
            source_capabilities: None,
            on_dynamic_source: reject_dynamic_source,
        }
    }
}

impl<F> LocalSenderLinkAcceptor<Symbol, F>
where
    F: Fn(Source) -> Option<Source>,
{
    /// Accepts an incoming attach as a local sender
    pub async fn accept_incoming_attach<R>(
        &self,
        shared: &SharedLinkAcceptorFields,
        remote_attach: Attach,
        session: &mut SessionHandle<R>,
    ) -> Result<Sender, SenderAttachError> {
        let snd_settle_mode = if shared
            .supported_snd_settle_modes
            .supports(&remote_attach.snd_settle_mode)
        {
            remote_attach.snd_settle_mode.clone()
        } else {
            shared.fallback_snd_settle_mode.clone()
        };
        let rcv_settle_mode = if shared
            .supported_rcv_settle_modes
            .supports(&remote_attach.rcv_settle_mode)
        {
            remote_attach.rcv_settle_mode.clone()
        } else {
            shared.fallback_rcv_settle_mode.clone()
        };

        let (incoming_tx, mut incoming_rx) = mpsc::channel(shared.buffer_size);

        let flow_state_inner = LinkFlowStateInner {
            initial_delivery_count: self.initial_delivery_count,
            delivery_count: self.initial_delivery_count,
            link_credit: 0,
            available: 0,
            drain: false,
            properties: shared.properties.clone(),
        };
        let flow_state = Arc::new(LinkFlowState::sender(flow_state_inner));
        let notifier = Arc::new(Notify::new());
        let flow_state_producer = Producer::new(notifier.clone(), flow_state.clone());
        let flow_state_consumer = Consumer::new(notifier, flow_state);

        let unsettled = Arc::new(RwLock::new(None));
        let link_handle = LinkRelay::Sender {
            tx: incoming_tx,
            output_handle: (),
            flow_state: flow_state_producer,
            unsettled: unsettled.clone(),
            receiver_settle_mode: remote_attach.rcv_settle_mode.clone(),
        };

        // Allocate link in session
        let input_handle = InputHandle::from(remote_attach.handle.clone());
        let output_handle = super::session::allocate_incoming_link(
            &session.control,
            remote_attach.name.clone(),
            link_handle,
            input_handle,
        )
        .await?;

        // In this case, the sender is considered to hold the authoritative version of the
        // version of the source properties
        let local_source = remote_attach.source.clone().and_then(|s| {
            if s.dynamic {
                (self.on_dynamic_source)(*s).map(|mut s| {
                    s.capabilities = self.source_capabilities.clone().map(Into::into);
                    s
                })
            } else {
                let mut source = *s;
                source.capabilities = self.source_capabilities.clone().map(Into::into);
                Some(source)
            }
        });

        let mut link = SenderLink::<Target> {
            role: PhantomData,
            local_state: LinkState::Unattached, // will be set in `on_incoming_attach`

            name: remote_attach.name.clone(),
            output_handle: Some(output_handle),
            input_handle: None, // this will be set in `on_incoming_attach`
            snd_settle_mode,
            rcv_settle_mode,
            source: local_source,
            target: None, // Will take value from incoming attach
            max_message_size: shared.max_message_size.unwrap_or(0),
            offered_capabilities: shared.offered_capabilities.clone(),
            desired_capabilities: shared.desired_capabilities.clone(),
            flow_state: flow_state_consumer,
            unsettled,
        };

        let outgoing = session.outgoing.clone();

        match link.on_incoming_attach(remote_attach) {
            Ok(_) => link.send_attach(&outgoing, &session.control, false).await?,
            Err(attach_error) => {
                // Complete attach then detach should any error happen
                link.send_attach(&outgoing, &session.control, false).await?;
                match attach_error {
                    SenderAttachError::SndSettleModeNotSupported
                    | SenderAttachError::RcvSettleModeNotSupported => {
                        // FIXME: The initiating side is responsible for checking whether the desired modes are supported?
                    }
                    _ => {
                        return Err(link
                            .handle_attach_error(
                                attach_error,
                                &outgoing,
                                &mut incoming_rx,
                                &session.control,
                            )
                            .await);
                    }
                }
            }
        }

        let inner = SenderInner {
            link,
            buffer_size: shared.buffer_size,
            session: session.control.clone(),
            outgoing,
            incoming: incoming_rx,
        };
        Ok(Sender { inner })
    }
}
