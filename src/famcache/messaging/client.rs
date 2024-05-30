use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::RwLock};
use uuid::Uuid;

pub struct Messaging {
    socket: Arc<RwLock<Option<TcpStream>>>,
}

impl Messaging {
    /// Create a new instance of the client
    pub(crate) fn new(socket: Arc<RwLock<Option<TcpStream>>>) -> Self {
        Messaging { socket }
    }

    pub async fn publish(&self, topic: &str, message: &str) -> Result<()> {
        let id = Uuid::new_v4().to_string();

        self.socket
            .write()
            .await
            .as_mut()
            .expect("Socket is not initialized")
            .write_all(format!("{} PUBLISH {} {}\n", id, topic, message).as_bytes())
            .await
            .with_context(|| "Failed to send message to server")
    }
}
