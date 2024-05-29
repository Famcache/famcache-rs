use tokio::sync::oneshot;

use crate::query::holder::CacheQuery;

/// Holds channel sender to resolve query
#[derive(Debug)]
pub struct QueueResolver {
  pub(crate) sender: oneshot::Sender<CacheQuery>,
}

impl QueueResolver {
  pub fn new(sender: oneshot::Sender<CacheQuery>) -> QueueResolver {
    QueueResolver {
      sender,
    }
  }

  /// Send the result of the query to the channel
  pub fn resolve(self, result: CacheQuery) {
    let _ = self.sender.send(result);

  }
}