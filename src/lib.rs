use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, hash::Hash};
use xarc::{AtomicXarc, Xarc};

pub struct BlockhashService;

impl BlockhashService {
    /// Requires tokio. Spawns a tokio task in the background to periodically update
    /// the blockhash served by the service.
    pub fn start(url: impl ToString, update_frequency_millis: u64) -> LatestBlockhash {
        // Initialize rpc client
        let client = RpcClient::new(url.to_string());

        // Initialize interval
        let mut interval = tokio::time::interval(Duration::from_millis(update_frequency_millis));

        // Initialize hash xarc
        let hash_atomic_xarc = Arc::new(AtomicXarc::null());
        let hash_atomic_xarc_task_handle = Arc::clone(&hash_atomic_xarc);

        tokio::task::spawn(async move {
            // Track consecutive failures to fetch hash
            let mut num_failures = 0;

            loop {
                // Wait until next tick
                interval.tick().await;

                // Fetch blockhash
                let Ok((blockhash, _slot)) = client
                    .get_latest_blockhash_with_commitment(CommitmentConfig::finalized())
                    .await
                else {
                    num_failures += 1;
                    log::warn!(
                        "failed to fetch blockhash ({} consecutive failures)",
                        num_failures
                    );
                    continue;
                };

                // Swap hash
                log::trace!("updating blockhash to {blockhash}");
                let hash_xarc = Xarc::new(blockhash);
                drop(hash_atomic_xarc_task_handle.swap(&hash_xarc, Ordering::Relaxed));

                // Reset failure counter
                num_failures = 0;
            }
        });

        // Wait for first update before returning to guarantee nonnull
        while hash_atomic_xarc.load(Ordering::Relaxed).is_null() {
            log::warn!("waiting for blockhash service");
            #[allow(deprecated)]
            std::thread::sleep_ms(1000);
        }

        log::info!("blockhash service started");

        LatestBlockhash(hash_atomic_xarc)
    }
}

#[derive(Clone)]
pub struct LatestBlockhash(Arc<AtomicXarc<Hash>>);

impl LatestBlockhash {
    pub fn load(&self) -> Hash {
        *self
            .0
            .load(Ordering::Acquire)
            .maybe_deref()
            .expect("we waited for blockhash before returning handle")
    }
}
