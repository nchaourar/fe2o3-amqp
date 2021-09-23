use std::convert::TryInto;

use crate::error::EngineError;
pub use crate::transport::Transport;
use fe2o3_types::performatives::{Begin, ChannelMax, MaxFrameSize};
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use url::Url;

use self::{builder::WithoutContainerId, mux::ConnMuxControl};

mod builder;
mod heartbeat;
mod mux;

pub use builder::Builder;

use super::session::{Session, SessionFrame};

pub const MIN_MAX_FRAME_SIZE: u32 = 512;

/// Incoming channel id / remote channel id
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IncomingChannelId(pub u16);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutgoingChannelId(pub u16);

impl From<u16> for OutgoingChannelId {
    fn from(val: u16) -> Self {
        Self(val)
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Start,

    HeaderReceived,

    HeaderSent,

    HeaderExchange,

    OpenPipe,

    OpenClosePipe,

    OpenReceived,

    OpenSent,

    ClosePipe,

    Opened,

    CloseReceived,

    CloseSent,

    Discarding,

    End,
}

// brw can still be used as background task
// The broker message can be
// ```rust
// enum Message {
// Incoming(Frame),
// Outgoing(Frame)
// }
// ```
pub struct Connection {
    // FIXME: is this really needed?
    // local_open: Arc<Open>, // parameters should be set using the builder and not change before reconnect
    // tx to conn_mux for session
    // mux: ConnMuxHandle,
    
    mux: Sender<ConnMuxControl>,
    handle: JoinHandle<Result<(), EngineError>>,
    session_tx: Sender<SessionFrame>,
}

impl Connection {
    // pub fn mux(&self) -> &ConnMuxHandle {
    //     &self.mux
    // }

    // pub fn mux_mut(&mut self) -> &mut ConnMuxHandle {
    //     &mut self.mux
    // }

    pub fn builder() -> Builder<WithoutContainerId> {
        Builder::new()
    }

    pub async fn open(
        container_id: String,
        max_frame_size: impl Into<MaxFrameSize>,
        channel_max: impl Into<ChannelMax>,
        url: impl TryInto<Url, Error = url::ParseError>,
    ) -> Result<Connection, EngineError> {
        Connection::builder()
            .container_id(container_id)
            .max_frame_size(max_frame_size)
            .channel_max(channel_max)
            .open(url)
            .await
    }

    pub async fn close(&mut self) -> Result<(), EngineError> {
        self.mux.send(mux::ConnMuxControl::Close).await?;
        // Ok(())
        match (&mut self.handle).await {
            Ok(res) => res,
            Err(_) => Err(EngineError::Message("JoinError"))
        }
        // self.mux.close().await
    }

    pub(crate) async fn create_session(&mut self, local_begin: Begin) -> Result<Session, EngineError> {
        todo!()
    }

}

// impl From<ConnMuxHandle> for Connection {
//     fn from(mux: ConnMuxHandle) -> Self {
//         Self { mux }
//     }
// }

#[cfg(test)]
mod tests {
    use crate::transport::connection::Connection;

    #[tokio::test]
    async fn test_connection_codec() {
        use tokio_test::io::Builder;
        /* tokio_test doesn't seem to handle spawning new tasks */
        let mock = Builder::new()
            .write(b"AMQP")
            .write(&[0, 1, 0, 0])
            .read(b"AMQP")
            .read(&[0, 1, 0, 0])
            .build();

        let _ = Connection::builder()
            .container_id("1234")
            .hostname("127.0.0.1")
            .max_frame_size(100)
            .channel_max(9)
            .idle_time_out(10u32)
            .open_with_stream(mock)
            .await
            .unwrap();
    }
}
