use actix::{Addr, MailboxError};
use async_trait::async_trait;
use mockall::automock;

use super::ws_server::{SessionMessage, WebsocketServer};

#[automock]
#[async_trait]
pub trait WebsocketServerSender: Sync + Send {
    async fn do_send(&self, m: SessionMessage) -> Result<(), MailboxError>;
}

pub struct WsSenderWrapper {
    websocket_server: Addr<WebsocketServer>,
}

impl WsSenderWrapper {
    pub fn new(websocket_server: Addr<WebsocketServer>) -> Self {
        WsSenderWrapper { websocket_server }
    }
}

#[async_trait]
impl WebsocketServerSender for WsSenderWrapper {
    async fn do_send(&self, message: SessionMessage) -> Result<(), MailboxError> {
        self.websocket_server.send(message).await
    }
}
