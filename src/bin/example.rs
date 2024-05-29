use famcache::{Config, Famcache};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut client = Famcache::new(Config::new("localhost", 3577));

    client.connect().await?;

    client.set("test", "rust", None).await?;
    client.set("test1", "rust2", None).await?;

    let val = client.get("test").await?;

    println!("Connected to server: {:?}", val);

    Ok(())
}
