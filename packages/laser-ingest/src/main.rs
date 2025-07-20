use anyhow::Result;
use futures_util::StreamExt;
use redis::aio::MultiplexedConnection as RedisConn;
use std::collections::HashSet;
use tokio::sync::watch;

mod filters;
mod supabase;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file from the project root.
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    //--------------------------------------------------
    // Fetch initial filters via REST
    //--------------------------------------------------
    let initial_filters = supabase::get_initial_filters().await?;
    tracing::info!(?initial_filters, "Using initial filters");

    //--------------------------------------------------
    // Channel for dynamic filters
    //--------------------------------------------------
    let (tx, rx) = watch::channel(initial_filters);

    //--------------------------------------------------
    // Spawn Supabase listener task (updates tx)
    //--------------------------------------------------
    tokio::spawn(async move {
        loop {
            // Re-spawn the listener on failure
            if let Err(e) = supabase::supabase_listener_task(tx.clone()).await {
                tracing::error!("Supabase listener failed: {e}");
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    //--------------------------------------------------
    // Run ingest worker (blocks)
    //--------------------------------------------------
    tracing::info!("Starting ingest worker");
    ingest_worker(rx).await
}

async fn ingest_worker(
    mut filter_rx: watch::Receiver<filters::FilterSet>,
) -> Result<()> {
    //--------------------------------------------------
    // Redis connection (shared across restarts)
    //--------------------------------------------------
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url)?;

    loop {
        // Wait for (possibly initial) filters
        let current_filter = filter_rx.borrow().clone();
        tracing::info!(?current_filter, "Applying new filters");

        // Build SubscribeRequest from filter set
        let request = current_filter.clone().into_subscribe_request();

        //--------------------------------------------------
        // LaserStream configuration
        //--------------------------------------------------
        let api_key = std::env::var("HELIUS_API_KEY").expect("Set HELIUS_API_KEY");
        let endpoint_url = std::env::var("HELIUS_ENDPOINT")
            .unwrap_or_else(|_| "https://laserstream-mainnet-ewr.helius-rpc.com".to_string());
        let config = helius_laserstream::LaserstreamConfig {
            api_key,
            endpoint: endpoint_url.parse()?,
            ..Default::default()
        };

        //--------------------------------------------------
        // Connect & subscribe
        //--------------------------------------------------
        tracing::info!("Connecting to Helius LaserStream...");
        let stream = helius_laserstream::subscribe(config, request);
        futures_util::pin_mut!(stream);

        // Open Redis multiplexed connection
        let mut redis_conn: RedisConn = redis_client.get_multiplexed_async_connection().await?;

        //--------------------------------------------------
        // Main loop
        //--------------------------------------------------
        loop {
            if let Some(Ok(mut msg)) = stream.next().await {
                if let Some(update) = msg.update_oneof.take() {
                    match update {
                        helius_laserstream::grpc::subscribe_update::UpdateOneof::Account(u) => {
                            tracing::info!(account = ?u.account.as_ref().map(|a| a.pubkey.clone()), "Received account update");
                            publish(&mut redis_conn, "heimdall:accounts", &u).await?
                        }
                        helius_laserstream::grpc::subscribe_update::UpdateOneof::Slot(u) => {
                            tracing::info!(slot = u.slot, "Received slot update");
                            publish(&mut redis_conn, "heimdall:slots", &u).await?
                        }
                        helius_laserstream::grpc::subscribe_update::UpdateOneof::Transaction(u) => {
                            let required_accounts = &current_filter.account_required;

                            if !required_accounts.is_empty() {
                                if let Some(tx) = &u.transaction {
                                    if let Some(sanitized_tx) = &tx.transaction {
                                        let tx_accounts: HashSet<_> = sanitized_tx
                                            .message
                                            .as_ref()
                                            .unwrap()
                                            .account_keys
                                            .iter()
                                            .map(|key| bs58::encode(key).into_string())
                                            .collect();
                                        let matched =
                                            required_accounts.iter().any(|acc| tx_accounts.contains(acc));
                                        if matched {
                                            tracing::info!(
                                                transaction = ?u.transaction,
                                                "Received transaction for watched pool"
                                            );
                                            publish(&mut redis_conn, "heimdall:transactions", &u)
                                                .await?;
                                        }
                                    }
                                }
                            } else {
                                tracing::info!(transaction = ?u.transaction, "Received transaction update");
                                publish(&mut redis_conn, "heimdall:transactions", &u)
                                    .await?;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

async fn publish<M: prost::Message>(
    conn: &mut RedisConn,
    stream: &str,
    msg: &M,
) -> redis::RedisResult<()> {
    let mut buf = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut buf).unwrap();
    tracing::debug!(stream, bytes = buf.len(), "Publishing to Redis");
    redis::cmd("XADD")
        .arg(stream)
        .arg("*")
        .arg("data")
        .arg(buf)
        .query_async(conn)
        .await
} 