use anyhow::Result;
use famcache::{Config, Famcache};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Famcache::new(Config::new("localhost", 3577));

    client.connect().await?;

    let mut subscription = client.messaging.subscribe("topic1").await?;

    tokio::spawn(async move {
        loop {
            let message = subscription.recv().await.unwrap();
            println!("Received message: {}", message);
        }
    })
    .await?;

    client.messaging.unsubscribe("topic1").await?;
    Ok(())
}
