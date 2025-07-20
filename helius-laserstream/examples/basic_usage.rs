use futures_util::StreamExt;
use helius_laserstream::{
    grpc::{
        subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
        SubscribeRequestFilterAccounts,
    },
    subscribe, LaserstreamConfig,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("HELIUS_API_KEY").expect("HELIUS_API_KEY must be set");
    let endpoint_url = String::from("https://laserstream-devnet-ewr.helius-rpc.com");

    let config = LaserstreamConfig {
        api_key,
        endpoint: endpoint_url.parse()?,
        ..Default::default()
    };

    // --- Subscription Request ---
    // Subscribe to account updates for a specific Meteora pool.
    let mut account_filters = HashMap::new();
    account_filters.insert(
        "meteora_pool_updates".to_string(),
        SubscribeRequestFilterAccounts {
            account: vec!["ApvLFu3ecKSdabEcGK25vTMFoErPoKHK4XUVZDdcoeb4".to_string()],
            owner: vec![],
            filters: vec![],
            nonempty_txn_signature: None,
        },
    );

    let request = SubscribeRequest {
        accounts: account_filters,
        commitment: Some(CommitmentLevel::Processed as i32),
        ..Default::default()
    };

    println!("Connecting and subscribing with filter: {:?}", request);

    let stream = subscribe(config, request);
    futures::pin_mut!(stream);

    while let Some(result) = stream.next().await {
        match result {
            Ok(update) => {
                if let Some(UpdateOneof::Account(acc)) = update.update_oneof {
                    println!(
                        "SUCCESS: Received account update for account: {:?}",
                        acc.account
                            .as_ref()
                            .map(|a| bs58::encode(&a.pubkey).into_string())
                    );
                    println!(" -> Data length: {} bytes", acc.account.map_or(0, |a| a.data.len()));
                } else {
                    // We will still get slot updates as keep-alives
                    println!("Received other update (e.g., Slot): {:?}", update.update_oneof);
                }
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
            }
        }
    }

    println!("Stream finished.");
    Ok(())
}
