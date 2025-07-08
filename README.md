# [Cyrup Sugars](https://github.com/cyrup-ai/sugars)

[![Crates.io](https://img.shields.io/crates/v/cyrup_sugars.svg)](https://crates.io/crates/cyrup_sugars)
[![Documentation](https://docs.rs/cyrup_sugars/badge.svg)](https://docs.rs/cyrup_sugars)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

Syntactic sugar utilities for Rust - collections, async patterns, and macros.

## Features

- `collections` - Collection types: `ZeroOneOrMany`, `OneOrMany`, `ByteSize`
- `async` - Async utilities: `AsyncTask` and `AsyncStream`
- `macros` - Collection and async macros
- `hashbrown-json` - JSON object syntax for collections
- `gix-interop` - Git object hash tables

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
cyrup_sugars = "0.1.0"
```

Or with specific features:

```toml
[dependencies]
cyrup_sugars = { version = "0.1", features = ["hashbrown-json"] }
```

## Examples

### Collections

```rust
use cyrup_sugars::collections::{ByteSizeExt, OneOrMany, ZeroOneOrMany};

// Byte sizes
let cache_size = 512.mb();
let memory_limit = 8.gb();

// Collections that handle single or multiple values
let servers = ZeroOneOrMany::many(vec![
    "api.example.com",
    "db.example.com", 
    "cache.example.com"
]);

let primary_server = OneOrMany::one("main.example.com");
let backup_servers = OneOrMany::many(vec!["backup1.com", "backup2.com"])?;
```

### Async Operations

```rust
use cyrup_sugars::{AsyncTask, AsyncStream};

fn fetch_user(id: u64) -> AsyncTask<User> {
    AsyncTask::spawn(async move {
        api_client.get_user(id).await
    })
}

let user = fetch_user(123)
    .with_timeout(Duration::from_secs(5))
    .with_retry(3)
    .on_success(|user| println!("Welcome {}!", user.name))
    .on_error(|e| log::error!("Failed: {}", e))
    .await;

let events: AsyncStream<Event> = subscribe_to_events();
events
    .filter(|e| e.priority == Priority::High)
    .on_each(|event| process_event(event))
    .on_complete(|| println!("Stream ended"))
    .collect_async()
    .await;
```

### JSON Object Syntax

With the `hashbrown-json` feature, you can use clean JSON-like syntax in builders:

```rust
// Configuration with JSON syntax
client
    .with_headers({"Authorization" => "Bearer token123"})
    .with_options({
        "timeout" => "30",
        "retries" => "3",
        "max_connections" => "100"
    })

// Database configuration
let db = Database::connect({
    "host" => "localhost",
    "port" => "5432",
    "database" => "myapp",
    "user" => "postgres"
})

// API client setup
let api = ApiClient::new()
    .endpoint("https://api.example.com")
    .auth({
        "type" => "oauth2",
        "client_id" => "abc123",
        "scope" => "read write"
    })
    .rate_limit({
        "requests_per_minute" => "100",
        "burst" => "10"
    })
    .build()
```

### Feature Gates

All modules are feature-gated for minimal dependencies:

```toml
# Minimal - only collections
cyrup_sugars = { version = "0.1", features = ["collections"] }

# Async support - choose your runtime:
cyrup_sugars = { version = "0.1", features = ["tokio-async"] }     # Tokio ecosystem
cyrup_sugars = { version = "0.1", features = ["std-async"] }       # Runtime-agnostic
cyrup_sugars = { version = "0.1", features = ["crossbeam-async"] } # Compute-heavy workloads

# 🔥 Hashbrown with JSON magic
cyrup_sugars = { version = "0.1", features = ["hashbrown-json"] }

# Everything
cyrup_sugars = { version = "0.1", features = ["full"] }
```

### Zero-Cost Abstractions

Collections like `ZeroOneOrMany` optimize for common cases:

- `None` variant uses zero heap allocations
- `One` variant stores the element directly
- `Many` variant pre-allocates capacity when transitioning from `One`

## Examples

Run the examples to see the library in action:

```bash
# Collections usage
cargo run --example collections_basic --features collections

# Async task pipeline
cargo run --example async_task_pipeline --features async

# Stream processing
cargo run --example async_stream_processing --features async

# Macro usage
cargo run --example macro_usage --features macros

# Full application
cargo run --example full_application --features full
```

## Testing

Run tests with full coverage:

```bash
# Run all tests
cargo test --all-features

# Generate coverage report
cargo tarpaulin --all-features --out Html
```

## Documentation

Generate and view documentation:

```bash
cargo doc --all-features --open
```

## Benchmarks

Run performance benchmarks:

```bash
cargo bench --all-features
```

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test --all-features`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy --all-features -- -D warnings`
4. Documentation is updated: `cargo doc --all-features`
5. Examples work: `cargo run --example <name> --features <features>`

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
