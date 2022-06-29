//! Implements OwnedTransaction

use async_trait::async_trait;
use fe2o3_amqp_types::{transaction::{Declared, TransactionId, TransactionalState}, messaging::{Outcome, DeliveryState,}, definitions::{SequenceNo, Fields}, primitives::Symbol};
use serde_amqp::Value;

use crate::{session::SessionHandle, link::{SendError, DispositionError, FlowError, delivery::DeliveryFut}, Receiver, Delivery, endpoint::ReceiverLink, Sender, Sendable};

use super::{Controller, OwnedDeclareError, TransactionDischarge, TransactionExt, TransactionalRetirement, TxnAcquisition, TXN_ID_KEY, OwnedDischargeError};

/// An owned transaction that has exclusive access to its own control link
#[derive(Debug)]
pub struct OwnedTransaction {
    controller: Controller,
    declared: Declared,
    is_discharged: bool
}

#[async_trait]
impl TransactionDischarge for OwnedTransaction {
    type Error = OwnedDischargeError;

    fn is_discharged(&self) -> bool {
        self.is_discharged
    }

    async fn discharge(&mut self, fail: bool) -> Result<(), Self::Error> {
        if !self.is_discharged {
            self.controller.discharge(self.declared.txn_id.clone(), fail).await?;
            self.is_discharged = true;
        }
        Ok(())
    }

    async fn rollback(mut self) -> Result<(), Self::Error> {
        self.discharge(true).await?;
        self.controller.close().await?;
        Ok(())
    }

    async fn commit(mut self) -> Result<(), Self::Error> {
        self.discharge(false).await?;
        self.controller.close().await?;
        Ok(())
    }
}

#[async_trait]
impl TransactionalRetirement for OwnedTransaction {
    type RetireError = DispositionError;

    /// Associate an outcome with a transaction
    ///
    /// The delivery itself need not be associated with the same transaction as the outcome, or
    /// indeed with any transaction at all. However, the delivery MUST NOT be associated with a
    /// different non-discharged transaction than the outcome. If this happens then the control link
    /// MUST be terminated with a transaction-rollback error.
    async fn retire<T>(
        &mut self,
        recver: &mut Receiver,
        delivery: &Delivery<T>,
        outcome: Outcome,
    ) -> Result<(), Self::RetireError> 
    where T: Send + Sync,
    {
        let txn_state = TransactionalState {
            txn_id: self.declared.txn_id.clone(),
            outcome: Some(outcome),
        };
        let state = DeliveryState::TransactionalState(txn_state);
        recver
            .inner
            .dispose(
                delivery.delivery_id.clone(),
                delivery.delivery_tag.clone(),
                None,
                state,
            )
            .await
    }
}

impl TransactionExt for OwnedTransaction {
    fn txn_id(&self) -> &TransactionId {
        &self.declared.txn_id
    }
}

impl OwnedTransaction {
    /// Declare an transaction with an owned control link
    pub async fn declare<R>(
        session: &mut SessionHandle<R>, 
        name: impl Into<String>, 
        global_id: impl Into<Option<TransactionId>>,
    ) -> Result<OwnedTransaction, OwnedDeclareError> {
        let controller = Controller::attach(session, name).await?;
        Self::declare_with_controller(controller, global_id).await.map_err(Into::into)
    }

    /// Declare an transaction with an owned control link
    pub async fn declare_with_controller(
        controller: Controller,
        global_id: impl Into<Option<TransactionId>>
    ) -> Result<OwnedTransaction, SendError> {
        let declared = controller.declare_inner(global_id.into()).await?;
        Ok(Self {
            controller,
            declared,
            is_discharged: false,
        })
    }


    /// Post a transactional work without waiting for the acknowledgement.
    async fn post_batchable<T>(
        &mut self,
        sender: &mut Sender,
        sendable: impl Into<Sendable<T>>,
    ) -> Result<DeliveryFut<Result<(), SendError>>, SendError> 
    where
        T: serde::Serialize,
    {
        // If the transaction controller wishes to associate an outgoing transfer with a
        // transaction, it MUST set the state of the transfer with a transactional-state carrying
        // the appropriate transaction identifier

        // Note that if delivery is split across several transfer frames then all frames MUST be
        // explicitly associated with the same transaction.
        let sendable = sendable.into();
        let state = TransactionalState {
            txn_id: self.declared.txn_id.clone(),
            outcome: None,
        };
        let state = DeliveryState::TransactionalState(state);
        let settlement = sender.inner.send_with_state(sendable, Some(state)).await?;

        Ok(DeliveryFut::from(settlement))
    }

    /// Post a transactional work
    pub async fn post<T>(
        &mut self,
        sender: &mut Sender,
        sendable: impl Into<Sendable<T>>,
    ) -> Result<(), SendError>
    where
        T: serde::Serialize,
    {
        // If the transaction controller wishes to associate an outgoing transfer with a
        // transaction, it MUST set the state of the transfer with a transactional-state carrying
        // the appropriate transaction identifier

        // Note that if delivery is split across several transfer frames then all frames MUST be
        // explicitly associated with the same transaction.
        let fut = self.post_batchable(sender, sendable).await?;

        // On receiving a non-settled delivery associated with a live transaction, the transactional
        // resource MUST inform the controller of the presumptive terminal outcome before it can
        // successfully discharge the transaction. That is, the resource MUST send a disposition
        // performative which covers the posted transfer with the state of the delivery being a
        // transactional-state with the correct transaction identified, and a terminal outcome. This
        // informs the controller of the outcome that will be in effect at the point that the
        // transaction is successfully discharged.
        fut.await
    }
    
    /// Acquire a transactional work
    ///
    /// This will send
    pub async fn acquire<'r>(
        self,
        recver: &'r mut Receiver,
        credit: SequenceNo,
    ) -> Result<TxnAcquisition<'r, OwnedTransaction>, FlowError> {
        {
            let mut writer = recver.inner.link.flow_state.lock.write().await;
            let key = Symbol::from(TXN_ID_KEY);
            let value = Value::Binary(self.declared.txn_id.clone());
            match &mut writer.properties {
                Some(fields) => {
                    if fields.contains_key(&key) {
                        return Err(FlowError::IllegalState);
                    }

                    fields.insert(key, value);
                }
                None => {
                    let mut fields = Fields::new();
                    fields.insert(key, value);
                }
            }
        }

        recver
            .inner
            .link
            .send_flow(&mut recver.inner.outgoing, Some(credit), None, false)
            .await?;
        Ok(TxnAcquisition { txn: self, recver })
    }
}