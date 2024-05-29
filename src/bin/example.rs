use famcache::{Famcache, Config};

#[tokio::main]
async fn main() {
  let mut client = Famcache::new(Config::new("localhost", 3577));

  client.connect().await.unwrap();

  tokio::time::sleep(std::time::Duration::from_secs(1)).await;

  println!("Calling 1st set");
  client.set("test", "rust", None).await.unwrap();

  println!("Calling 2nd set");
  client.set("test1", "rust2", None).await.unwrap();

  let val = client.get("test").await.unwrap();

  println!("Connected to server: {:?}", val);
}
