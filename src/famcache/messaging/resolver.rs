use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct MessageResolver {
    pub(crate) sender: Sender<String>,
}

impl MessageResolver {
    pub fn new(sender: Sender<String>) -> MessageResolver {
        MessageResolver {
            sender,
        }
    }

    pub async fn resolve(&self, result: String) {
        let _ = self.sender.send(result).await;
    }
}