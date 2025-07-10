use dotenv::dotenv;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::*;
use futures::StreamExt;
use std::collections::HashMap;
use std::env;
use tokio::time::{sleep, Duration};
use bs58;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    let endpoint = env::var("GRPC_ENDPOINT")
        .unwrap();
    let token: Option<String> = env::var("GRPC_TOKEN").ok(); // Read token from env if available
    
    let mut client = if let Some(auth_token) = token {
        GeyserGrpcClient::build_from_shared(endpoint)?
            .x_token(Some(auth_token))?
            .connect()
            .await?
    } else {
        GeyserGrpcClient::build_from_shared(endpoint)?
            .connect()
            .await?
    };
    
    // USDC mint account
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    
    let mut accounts = HashMap::new();
    accounts.insert(
        "usdc_mint".to_string(),
        SubscribeRequestFilterAccounts {
            account: vec![usdc_mint.to_string()],
            owner: vec![],
            filters: vec![],
        }
    );
    
    let request = SubscribeRequest {
        accounts,
        ..Default::default()
    };
    
    let mut stream = client.subscribe_once(request).await?;

    println!("🚀 Connected! Monitoring USDC mint account...");
    
    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                if let Some(account) = msg.update_oneof {
                    match account {
                        subscribe_update::UpdateOneof::Account(account_update) => {
                            println!("\n📊 Account Update:");
                            let account_key = account_update.account.as_ref()
                                .map(|a| bs58::encode(&a.pubkey).into_string())
                                .unwrap_or("N/A".to_string());
                            println!("  Account: {}", account_key);
                            println!("  Lamports: {}", account_update.account.as_ref()
                                .map(|a| a.lamports).unwrap_or(0));
                            println!("  Slot: {}", account_update.slot);
                        }
                        _ => {} // Handle other update types as needed
                    }
                }
            }
            Err(error) => {
                eprintln!("❌ Stream error: {}", error);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
    
    Ok(())
}