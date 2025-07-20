use futures_util::{pin_mut, StreamExt};
use helius_laserstream::{
    subscribe,
    grpc::{
        subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterAccounts,
    },
    LaserstreamConfig,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("HELIUS_API_KEY")?;
    // Ensure you set HELIUS_ENDPOINT for devnet, otherwise it defaults to mainnet
    let endpoint = std::env::var("HELIUS_ENDPOINT")
        .unwrap_or_else(|_| "https://laserstream-devnet.helius-rpc.com".to_string());

    let config = LaserstreamConfig {
        api_key,
        endpoint: endpoint.parse()?,
        ..Default::default()
    };

    // -------------------------------------------------
    // Build SubscribeRequest with an account filter for the pool
    // -------------------------------------------------
    let mut accounts_to_subscribe = HashMap::new();
    accounts_to_subscribe.insert(
        "pool_state".to_owned(),
        SubscribeRequestFilterAccounts {
            account: vec![
                // Your Devnet Meteora Pool
                "ApvLFu3ecKSdabEcGK25vTMFoErPoKHK4XUVZDdcoeb4".to_owned(),
            ],
            ..Default::default()
        },
    );

    // -------------------------------------------------
    // (Optional) transaction filter: listen to any tx touching the pool
    // -------------------------------------------------
    use helius_laserstream::grpc::SubscribeRequestFilterTransactions;

    let mut tx_filters = HashMap::new();
    tx_filters.insert(
        "pool_txs".to_owned(),
        SubscribeRequestFilterTransactions {
            account_include: vec!["dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN".to_owned()],
            account_required: vec!["ApvLFu3ecKSdabEcGK25vTMFoErPoKHK4XUVZDdcoeb4".to_owned()],
            vote: Some(false),
            failed: Some(false),
            ..Default::default()
        },
    );

    let req = SubscribeRequest {
        accounts: accounts_to_subscribe,
        slots: HashMap::new(),
        transactions: tx_filters,
        blocks: HashMap::new(),
        blocks_meta: HashMap::new(),
        ..Default::default()
    };

    // -------------------------------------------------
    // Connect and stream
    // -------------------------------------------------
    println!("Connecting to LaserStream at {} ...", endpoint);
    let stream = subscribe(config, req);
    pin_mut!(stream);

    while let Some(message) = stream.next().await {
        match message {
            Ok(mut update) => {
                if let Some(oneof) = update.update_oneof.take() {
                    match oneof {
                        UpdateOneof::Account(acc_update) => {
                            println!("[{}] Account data changed ({} bytes)", acc_update.slot, acc_update.account.as_ref().map_or(0, |a| a.data.len()));
                        }
                        UpdateOneof::Transaction(tx_update) => {
                            if let Some(tx_info) = &tx_update.transaction {
                                println!("[{}] Tx touching pool: {}", tx_update.slot, bs58::encode(&tx_info.signature).into_string());
                            }
                        }
                        _ => {
                            // Slot updates will appear here, which is normal and confirms the connection is live.
                            println!("Received heartbeat: {:?}", oneof);
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Stream error: {err}");
                break;
            }
        }
    }

    Ok(())
} 