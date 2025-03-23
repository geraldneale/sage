use std::time::Duration;

use futures_lite::StreamExt;
use futures_util::stream::FuturesUnordered;
use sage_assets::fetch_uri;
use sage_database::{Database, NftData};
use tokio::{
    sync::mpsc,
    time::{sleep, timeout},
};
use tracing::{debug, info, warn};

use crate::{compute_nft_info, SyncEvent, WalletError};

#[derive(Debug)]
pub struct NftUriQueue {
    db: Database,
    sync_sender: mpsc::Sender<SyncEvent>,
}

impl NftUriQueue {
    pub fn new(db: Database, sync_sender: mpsc::Sender<SyncEvent>) -> Self {
        Self { db, sync_sender }
    }

    pub async fn start(self, delay: Duration) -> Result<(), WalletError> {
        loop {
            self.process_batch().await?;
            sleep(delay).await;
        }
    }

    async fn process_batch(&self) -> Result<(), WalletError> {
        let batch = self.db.unchecked_nft_uris(30).await?;

        if batch.is_empty() {
            return Ok(());
        }

        info!("Processing batch of {} NFT URIs", batch.len());

        let mut futures = FuturesUnordered::new();

        for item in batch {
            futures.push(async move {
                let result = timeout(Duration::from_secs(15), fetch_uri(item.uri.clone())).await;
                (item, result)
            });
        }
        // Track which hash types we've already found matches for
        let mut matched_hashes = std::collections::HashSet::new();

        while let Some((item, result)) = futures.next().await {
            // Skip if we already found a match for this hash type
            if matched_hashes.contains(&item.hash) {
                continue;
            }
            let mut tx = self.db.tx().await?;

            let hash_matches = match result {
                Ok(Ok(data)) => {
                    let hash_matches = data.hash == item.hash;

                    // if !hash_matches {
                    //     warn!(
                    //         "Hash mismatch for URI {} (expected {} but found {})",
                    //         item.uri, item.hash, data.hash
                    //     );
                    // }
                    if hash_matches {
                        // If we found a match, add it to our tracking set
                        // Use the hash directly without clone since it implements Copy
                        matched_hashes.insert(item.hash);
                    } else {
                        warn!(
                            "Hash mismatch for URI {} (expected {} but found {})",
                            item.uri, item.hash, data.hash
                        );
                    }

                    let existing = tx.fetch_nft_data(item.hash).await?;

                    if existing.as_ref().is_none_or(|data| !data.hash_matches) {
                        tx.insert_nft_data(
                            item.hash,
                            NftData {
                                mime_type: data.mime_type,
                                blob: data.blob.clone(),
                                hash_matches,
                            },
                        )
                        .await?;

                        let nfts = tx.nfts_by_metadata_hash(item.hash).await?;

                        for mut nft in nfts {
                            let info = compute_nft_info(nft.minter_did, Some(&data.blob));

                            nft.sensitive_content = info.sensitive_content;
                            nft.name = info.name;

                            // TODO: Is this correct?
                            if hash_matches {
                                nft.collection_id =
                                    info.collection.as_ref().map(|col| col.collection_id);

                                if let Some(collection) = info.collection {
                                    tx.insert_collection(collection).await?;
                                }
                            }

                            tx.insert_nft(nft).await?;
                        }
                    }

                    Some(hash_matches)
                }
                Ok(Err(error)) => {
                    debug!("Error fetching URI {}: {error}", item.uri);

                    None
                }
                Err(_error) => {
                    debug!("Timed out fetching URI {}", item.uri);

                    None
                }
            };

            tx.set_nft_uri_checked(item.uri, item.hash, hash_matches)
                .await?;

            tx.commit().await?;
        }

        self.sync_sender.send(SyncEvent::NftData).await.ok();

        Ok(())
    }
}
