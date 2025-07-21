use anyhow::Result;
use futures_util::StreamExt;
use processors::{default_registry, Processor};
use redis::aio::MultiplexedConnection as RedisConn;
use std::collections::{HashMap, HashSet};
use tokio::sync::watch;
use bs58;

mod processors;
mod types;
mod watchlist;

use types::PoolMeta;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // --------------------------------------------------
    // Load watchlist from local JSON (for dev / isolated mode)
    // --------------------------------------------------
    let path = std::env::var("WATCHLIST_JSON").unwrap_or_else(|_| "watchlist.json".to_string());
    let pools = watchlist::load_from_json(&path)?;
    tracing::info!(?pools, "Loaded watchlist");

    // Channel if we later want dynamic reloads (not used in this isolated example)
    let (tx, rx) = watch::channel(pools);

    ingest_worker(rx).await
}

async fn ingest_worker(mut pool_rx: watch::Receiver<Vec<PoolMeta>>) -> Result<()> {
    let processors_registry = default_registry();

    // Redis connection (shared across restarts)
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url)?;

    loop {
        let current_pools = pool_rx.borrow().clone();
        tracing::info!(pools = current_pools.len(), "Building SubscribeRequest for pools");

        // Build combined filters
        let mut accounts: HashMap<String, helius_laserstream::grpc::SubscribeRequestFilterAccounts> = HashMap::new();
        let mut tx_filters: HashMap<String, helius_laserstream::grpc::SubscribeRequestFilterTransactions> = HashMap::new();

        for pool in &current_pools {
            if let Some(proc) = processors_registry.get(&pool.variant) {
                let c = proc.build_filters(pool);
                accounts.extend(c.accounts);
                tx_filters.extend(c.transactions);
            }
        }

        let request = helius_laserstream::grpc::SubscribeRequest {
            accounts,
            transactions: tx_filters,
            blocks: HashMap::new(),
            slots: HashMap::new(),
            blocks_meta: HashMap::new(),
            commitment: Some(helius_laserstream::grpc::CommitmentLevel::Processed as i32),
            ..Default::default()
        };

        // Helius config
        let api_key = std::env::var("HELIUS_API_KEY").unwrap_or_else(|_| "demo".to_string());
        let endpoint_url = std::env::var("HELIUS_ENDPOINT")
            .unwrap_or_else(|_| "https://laserstream-devnet.helius-rpc.com".to_string());
        let config = helius_laserstream::LaserstreamConfig {
            api_key,
            endpoint: endpoint_url.parse()?,
            ..Default::default()
        };

        // Connect & subscribe
        tracing::info!("Connecting to Helius LaserStream...");
        let stream = helius_laserstream::subscribe(config, request);
        futures_util::pin_mut!(stream);

        let mut redis_conn: RedisConn = redis_client.get_multiplexed_async_connection().await?;

        // Main loop
        loop {
            tokio::select! {
                maybe_msg = stream.next() => {
                    if let Some(Ok(msg)) = maybe_msg {
                        // Determine which pool this update relates to (naive implementation)
                        if let Some(pool) = match_update_to_pool(&msg, &current_pools) {
                            if let Some(proc) = processors_registry.get(&pool.variant) {
                                proc.handle_update(pool, &msg, &mut redis_conn).await;
                            }
                        }
                    }
                },
                _ = pool_rx.changed() => {
                    // Rebuild filters on next loop iteration
                    tracing::info!("Watchlist changed, reconnecting...");
                    break;
                }
            }
        }
    }
}

fn match_update_to_pool<'a>(
    update: &helius_laserstream::grpc::SubscribeUpdate,
    pools: &'a [PoolMeta],
) -> Option<&'a PoolMeta> {
    if let Some(oneof) = &update.update_oneof {
        match oneof {
            helius_laserstream::grpc::subscribe_update::UpdateOneof::Account(acc_upd) => {
                if let Some(acc) = &acc_upd.account {
                    let key_str = bs58::encode(&acc.pubkey).into_string();
                    pools.iter().find(|p| p.quote_vault.as_ref().map(|v| v == &key_str).unwrap_or(false) || p.pool_id == key_str)
                } else { None }
            }
            helius_laserstream::grpc::subscribe_update::UpdateOneof::Transaction(tx_upd) => {
                if let Some(tx_info) = &tx_upd.transaction {
                    if let Some(tx) = &tx_info.transaction {
                        let accounts: HashSet<_> = tx.message.as_ref().unwrap().account_keys.iter().map(|k| bs58::encode(k).into_string()).collect();
                        pools.iter().find(|p| accounts.contains(&p.pool_id))
                    } else { None }
                } else { None }
            }
            _ => None,
        }
    } else { None }
} 