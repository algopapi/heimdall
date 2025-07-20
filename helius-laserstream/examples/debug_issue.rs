use futures_util::StreamExt;
use helius_laserstream::{
    grpc::{SubscribeRequest, SubscribeRequestFilterTransactions},
    subscribe, LaserstreamConfig,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("HELIUS_API_KEY").expect("HELIUS_API_KEY must be set");
    let endpoint_url = String::from("https://laserstream-mainnet-ewr.helius-rpc.com");

    let config = LaserstreamConfig {
        api_key,
        endpoint: endpoint_url.parse()?,
        ..Default::default()
    };

    let mut filters = HashMap::new();
    filters.insert(
        "debug_filter".to_string(),
        SubscribeRequestFilterTransactions {
            account_include: vec!["dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN".to_string()],
            account_required: vec!["ApvLFu3ecKSdabEcGK25vTMFoErPoKHK4XUVZDdcoeb4".to_string()],
            vote: Some(false),
            failed: Some(false),
            ..Default::default()
        },
    );

    let request = SubscribeRequest {
        transactions: filters,
        ..Default::default()
    };

    println!("Connecting and subscribing with filter: {:?}", request);
    let stream = subscribe(config, request);

    futures::pin_mut!(stream);

    while let Some(result) = stream.next().await {
        match result {
            Ok(update) => {
                println!("Received update: {:?}", update);
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
            }
        }
    }

    println!("Stream finished.");
    Ok(())
} 