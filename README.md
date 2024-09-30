# `blockhash-service`

lightweight background service for maintaining latest blockhash

## Usage

```rust
use blockhash_service::BlockhashService;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let update_frequency_millis = 5_000;
    let latest_blockhash = BlockhashService::start(
        "YOUR RPC",
        update_frequency_millis
    );

    loop {
        let blockhash = latest_blockhash.load();

        // Sign and send tx
    }
}
```
