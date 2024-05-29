use crate::{commands, query::holder::CacheQuery};

use super::{resolver::QueueResolver, Config};
use anyhow::{Context, Result};
use core::panic;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{oneshot, Mutex},
};
use uuid::Uuid;

pub struct Famcache {
    socket: Arc<Mutex<Option<TcpStream>>>,
    queue: Arc<Mutex<HashMap<String, QueueResolver>>>,
    config: Config,
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
            socket: Arc::new(Mutex::new(None)),
            queue: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    fn gen_id(&self) -> String {
        Uuid::new_v4().to_string()
    }

    async fn ensure_initialized(&self) {
        if self.socket.clone().lock().await.is_none() {
            panic!("Client is not initialized");
        }
    }

    fn create_sync_channel() -> (oneshot::Sender<CacheQuery>, oneshot::Receiver<CacheQuery>) {
        let (sender, receiver) = oneshot::channel();

        (sender, receiver)
    }

    async fn write_command(&self, command: String) -> Result<()> {
        self.socket
            .lock()
            .await
            .as_mut()
            .unwrap()
            .write_all(command.as_bytes())
            .await
            .with_context(|| "Failed to send command to server")?;

        Ok(())
    }

    async fn enqueue(&self, id: &str) -> oneshot::Receiver<CacheQuery> {
        let (sender, receiver) = Self::create_sync_channel();

        let mut queue = self.queue.lock().await;
        queue.insert(id.to_owned(), QueueResolver::new(sender));
        std::mem::drop(queue);

        receiver
    }

    fn listen(&self) {
        let socket = self.socket.clone();
        let queue = self.queue.clone();

        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            println!("Started Loop");

            loop {
                let mut sock = socket.lock().await;

                let socket = sock.as_mut().unwrap();

                let bytes_read = socket.read(&mut buffer).await.unwrap();

                if bytes_read == 0 {
                    return;
                }

                let response = String::from_utf8_lossy(&buffer[..bytes_read]);

                let result = CacheQuery::from_str(&response);

                if result.is_err() {
                    continue;
                }

                let result = result.unwrap();

                println!("Received response: {:?}", result);
                let mut queue = queue.lock().await;

                let resolver = queue.remove(&result.id).unwrap();

                resolver.resolve(result);

                println!("Resolved");
                // std::mem::drop(queue);
                // std::mem::drop(sock);
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
        let url = format!("{}:{}", self.config.host, self.config.port);
        let socket = TcpStream::connect(url)
            .await
            .with_context(|| "Failed to connect to server")?;

        self.socket = Arc::new(Mutex::new(Some(socket)));
        self.listen();

        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        self.ensure_initialized().await;

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

    pub async fn set(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<()> {
        println!("Ensure initialized");
        self.ensure_initialized().await;

        println!("Gen id");
        let query_id = self.gen_id();
        let command = commands::set(&query_id, key, value, ttl);

        println!("Enqueue");
        let receiver = self.enqueue(&query_id).await;

        println!("Write command");
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

    pub async fn del(&self, key: &str) -> Result<()> {
        self.ensure_initialized().await;

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
