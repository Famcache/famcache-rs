use crate::{commands, query::holder::CacheQuery};

use super::{messaging::Messaging, resolver::QueueResolver, Config};
use anyhow::{Context, Result};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{oneshot, RwLock},
};
use uuid::Uuid;

pub struct Famcache {
    socket: Arc<RwLock<Option<TcpStream>>>,
    queue: Arc<RwLock<HashMap<String, QueueResolver>>>,
    config: Config,

    pub messaging: Messaging,
}

impl Famcache {
    /// Create a new instance of the client
    ///
    /// # Example
    ///
    /// ```no_run
    /// use famcache::{Famcache, Config};
    ///
    /// let client = Famcache::new(Config::new("localhost", 3577));
    /// ```
    pub fn new(config: Config) -> Self {
        Famcache {
            socket: Arc::new(RwLock::new(None)),
            queue: Arc::new(RwLock::new(HashMap::new())),
            config,
            messaging: Messaging::new(Arc::new(RwLock::new(None))),
        }
    }

    fn gen_id(&self) -> String {
        Uuid::new_v4().to_string()
    }

    async fn ensure_initialized(&self) -> Result<()> {
        let socket_guard = self.socket.read().await;
        if socket_guard.is_none() {
            panic!("Client is not initialized");
        }
        Ok(())
    }

    fn create_sync_channel() -> (oneshot::Sender<CacheQuery>, oneshot::Receiver<CacheQuery>) {
        oneshot::channel()
    }

    async fn write_command(&self, command: String) -> Result<()> {
        let mut socket_guard = self.socket.write().await;

        if let Some(ref mut socket) = *socket_guard {
            socket
                .write_all(command.as_bytes())
                .await
                .with_context(|| "Failed to send command to server")?;
        }
        Ok(())
    }

    async fn enqueue(&self, id: &str) -> oneshot::Receiver<CacheQuery> {
        let (sender, receiver) = Self::create_sync_channel();

        let mut queue_guard = self.queue.write().await;

        queue_guard.insert(id.to_owned(), QueueResolver::new(sender));

        receiver
    }

    fn listen(&self) {
        let socket = self.socket.clone();
        let queue = self.queue.clone();

        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            loop {
                let socket_guard = socket.read().await;

                if (*socket_guard).is_none() {
                    continue;
                }

                let ref socket = socket_guard.as_ref().unwrap();

                let bytes_read = match socket.try_read(&mut buffer) {
                    Ok(bytes_read) => bytes_read,
                    Err(err) => {
                        if err.kind() == std::io::ErrorKind::WouldBlock {
                            continue;
                        }
                        return;
                    }
                };

                if bytes_read == 0 {
                    return;
                }

                let response = String::from_utf8_lossy(&buffer[..bytes_read]);
                let result = CacheQuery::from_str(&response);

                if result.is_err() {
                    continue;
                }

                let result = result.unwrap();

                let mut queue_guard = queue.write().await;

                if let Some(resolver) = queue_guard.remove(&result.id) {
                    resolver.resolve(result);
                }
            }
        });
    }
}

impl Famcache {
    /// Open a connection to the server
    ///
    /// # Example
    ///
    /// ```no_run
    /// use famcache::{Famcache, Config};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let mut client = Famcache::new(Config::new("localhost", 3577));
    ///
    ///   client.connect().await.unwrap();
    ///
    ///   println!("Connected to server");
    /// }
    pub async fn connect(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let socket = TcpStream::connect(addr)
            .await
            .with_context(|| "Failed to connect to server")?;

        let mut socket_guard = self.socket.write().await;
        *socket_guard = Some(socket);

        self.messaging = Messaging::new(self.socket.clone());

        self.listen();

        Ok(())
    }

    /// Get a value from the server
    ///
    /// # Example
    ///
    /// ```no_run
    /// use famcache::{Famcache, Config};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let mut client = Famcache::new(Config::new("localhost", 3577));
    ///
    ///   client.connect().await.unwrap();
    ///
    ///   client.get("test").await.unwrap();
    /// }
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        self.ensure_initialized().await?;

        let query_id = self.gen_id();
        let command = commands::get(&query_id, key);

        let receiver = self.enqueue(&query_id).await;

        self.write_command(command).await?;

        match receiver.await {
            Ok(result) => {
                if result.is_error {
                    return Err(anyhow::anyhow!("Failed to get value from server"));
                }

                Ok(result.value)
            }
            Err(_) => Err(anyhow::anyhow!("Failed to get value from server")),
        }
    }

    /// Set a value on the server
    ///
    /// # Example
    ///
    /// ```no_run
    /// use famcache::{Famcache, Config};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let mut client = Famcache::new(Config::new("localhost", 3577));
    ///
    ///   client.connect().await.unwrap();
    ///
    ///   client.set("test", "val", None).await.unwrap();
    /// }
    pub async fn set(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<()> {
        self.ensure_initialized().await?;

        let query_id = self.gen_id();
        let command = commands::set(&query_id, key, value, ttl);

        let receiver = self.enqueue(&query_id).await;

        self.write_command(command).await?;

        match receiver.await {
            Ok(result) => {
                if result.is_error {
                    return Err(anyhow::anyhow!("Failed to set value on server"));
                }

                Ok(())
            }
            Err(_) => Err(anyhow::anyhow!("Failed to get reply from server")),
        }
    }

    /// Delete a value from the server
    ///
    /// # Example
    ///
    /// ```no_run
    /// use famcache::{Famcache, Config};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let mut client = Famcache::new(Config::new("localhost", 3577));
    ///
    ///   client.connect().await.unwrap();
    ///
    ///   client.del("test").await.unwrap();
    /// }
    pub async fn del(&self, key: &str) -> Result<()> {
        self.ensure_initialized().await?;

        let query_id = self.gen_id();
        let command = commands::del(&query_id, key);

        let receiver = self.enqueue(&query_id).await;

        self.write_command(command).await?;

        match receiver.await {
            Ok(result) => {
                if result.is_error {
                    return Err(anyhow::anyhow!("Failed to delete value on server"));
                }

                Ok(())
            }
            Err(_) => Err(anyhow::anyhow!("Failed to get reply from server")),
        }
    }
}
