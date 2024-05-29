# Famcache-rs

`Famcache-rs` is a Rust client for `Famcache`, a caching server written in Go. This client provides a simple interface for connecting to `Famcache` and performing basic cache operations like setting, getting, and deleting values.

## Features

- **Asynchronous API**: Fully asynchronous operations to interact with the `Famcache` server using `tokio`.
- **Concurrency Safe**: Uses `RwLock` for safe concurrent access.
- **Simple and Intuitive**: Easy to use API with straightforward methods for cache operations.

## Installation

Add famcache to your `Cargo.toml` or simply run

```sh
cargo add famcache
```

## Usage

Here's a quick example of how to use famcache-rs:

```rust
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
```

## API Reference

### `Famcache`

#### `new(config: Config) -> Self`

Creates a new `Famcache` client instance.

- `config`: Configuration object with the following properties:
  - `host`: Hostname of the `Famcache` server.
  - `port`: Port of the `Famcache` server.

#### `connect(&mut self) -> Result<()>`

Opens a connection to the `Famcache` server.

#### `set(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<()>`

Sets a value in the cache.

- `key`: The key to set.
- `value`: The value to set.
- `ttl`: Optional time-to-live in milliseconds.

#### `get<T>(&self, key: &str) -> Result<Option<T>>`

Gets a value from the cache.

- `key`: The key to retrieve.

Returns the value associated with the key or `None` if the key does not exist.

#### `del(&self, key: &str) -> Result<()>`

Deletes a value from the cache.

- `key`: The key to delete.

## Development

### Prerequisites

- Rust
- Cargo

### Installing Dependencies

```sh
cargo build
```

### Running Tests
```sh
cargo test
```

### Contributing
Contributions are welcome! Please open an issue or submit a pull request for any bugs or improvements.


### License
This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.

