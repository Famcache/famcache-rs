use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver},
        RwLock,
    },
};
use uuid::Uuid;

use super::resolver::MessageResolver;

pub struct Messaging {
    subscription: Arc<RwLock<HashMap<String, Arc<MessageResolver>>>>,
    socket: Arc<RwLock<Option<TcpStream>>>,
}

impl Messaging {
    pub(crate) fn is_messaging_event(msg: &str) -> bool {
        msg.starts_with("MESSAGE")
    }

    // example: MESSAGE topic1 payload
    pub(crate) fn parse_body(msg: &str) -> (&str, &str) {
        let parts: Vec<&str> = msg.split(' ').collect();
        (parts[1], parts[2])
    }

    pub(crate) fn new_resolver() -> (Arc<MessageResolver>, Receiver<String>) {
        let (sender, receiver) = mpsc::channel(2);
        (Arc::new(MessageResolver::new(sender)), receiver)
    }

    /// Create a new instance of the client
    pub(crate) fn new(socket: Arc<RwLock<Option<TcpStream>>>) -> Self {
        Messaging {
            socket,
            subscription: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn gen_id(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub(crate) async fn trigger(&self, topic: &str, message: &str) {
        let subscription = self.subscription.read().await;
        if let Some(resolver) = subscription.get(topic) {
            resolver.resolve(message.to_string()).await;
        } else {

        }
    }

    pub async fn publish(&self, topic: &str, message: &str) -> Result<()> {
        let id = self.gen_id();

        self.socket
            .write()
            .await
            .as_mut()
            .expect("Socket is not initialized")
            .write_all(format!("{} PUBLISH {} {}\n", id, topic, message).as_bytes())
            .await
            .with_context(|| "Failed to send message to server")
    }

    pub async fn unsubscribe(&self, topic: &str) -> Result<()> {
        let id = self.gen_id();

        self.socket
            .write()
            .await
            .as_mut()
            .expect("Socket is not initialized")
            .write_all(format!("{} UNSUBSCRIBE {}\n", id, topic).as_bytes())
            .await
            .with_context(|| "Failed to send message to server")?;

        {
            let mut subscription = self.subscription.write().await;
            subscription.remove(topic);
        }

        Ok(())
    }

    pub async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<Receiver<String>> {
        let id = self.gen_id();

        self.socket
            .write()
            .await
            .as_mut()
            .expect("Socket is not initialized")
            .write_all(format!("{} SUBSCRIBE {}\n", id, topic).as_bytes())
            .await
            .with_context(|| "Failed to send message to server")?;

        let mut subscription = self.subscription.write().await;
        let (resolver, receiver) = Messaging::new_resolver();
        subscription.insert(topic.to_string(), resolver);

        Ok(receiver)
    }
}
