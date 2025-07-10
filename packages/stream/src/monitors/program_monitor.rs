use dotenv::dotenv;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::*;
use futures::StreamExt;
use std::collections::HashMap;
use std::env;
use tokio::time::{sleep, Duration};
use bs58;

use crate::utils::deserializer::deserialize_pool_account_simple;
use crate::utils::dashboard::{
    create_shared_dashboard_state, 
    handle_pool_account_update, 
    update_dashboard_periodically,
    get_pool_statistics
};

const PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";

pub async fn monitor_program_accounts(grpc_endpoint: String) -> anyhow::Result<()> {
    dotenv().ok();
    
    let token: Option<String> = env::var("GRPC_TOKEN").ok();
    
    let dashboard_state = create_shared_dashboard_state();
    
    let dashboard_updater = {
        let state = dashboard_state.clone();
        tokio::spawn(update_dashboard_periodically(state))
    };
    
    println!("Initializing Raydium CP Dashboard...");
    
    let mut client = if let Some(auth_token) = token {
        GeyserGrpcClient::build_from_shared(grpc_endpoint)?
            .x_token(Some(auth_token))?
            .connect()
            .await?
    } else {
        GeyserGrpcClient::build_from_shared(grpc_endpoint)?
            .connect()
            .await?
    };
    
    let mut accounts = HashMap::new();
    accounts.insert(
        "raydium_cp_pools".to_string(),
        SubscribeRequestFilterAccounts {
            account: vec![],
            owner: vec![PROGRAM_ID.to_string()],
            filters: vec![],
        }
    );
    
    let mut transactions = HashMap::new();
    transactions.insert(
        "raydium_transactions".to_string(),
        SubscribeRequestFilterTransactions {
            vote: Some(false),
            failed: Some(false),
            signature: None,
            account_include: vec![PROGRAM_ID.to_string()],
            account_exclude: vec![],
            account_required: vec![PROGRAM_ID.to_string()],
        }
    );
    
    let request = SubscribeRequest {
        accounts,
        transactions,
        slots: HashMap::new(),
        blocks: HashMap::new(),
        blocks_meta: HashMap::new(),
        entry: HashMap::new(),
        commitment: Some(CommitmentLevel::Confirmed as i32),
        accounts_data_slice: vec![],
        ping: None,
        transactions_status: HashMap::new(),
    };
    
    let mut stream = client.subscribe_once(request).await?;

    println!("Connected! Monitoring Raydium CP program accounts and transactions...");
    println!("Dashboard updates every 10 seconds");
    
    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                if let Some(update) = msg.update_oneof {
                    match update {
                        subscribe_update::UpdateOneof::Account(account_update) => {
                            if let Err(e) = handle_account_update(dashboard_state.clone(), account_update).await {
                                eprintln!("Error handling account update: {}", e);
                            }
                        }
                        subscribe_update::UpdateOneof::Transaction(tx_update) => {
                            handle_transaction_update(tx_update).await;
                        }
                        subscribe_update::UpdateOneof::Slot(slot_update) => {
                            if slot_update.slot % 100 == 0 {
                                println!("Slot: {}", slot_update.slot);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(error) => {
                eprintln!("Stream error: {}", error);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
    
    dashboard_updater.abort();
    Ok(())
}

async fn handle_account_update(
    dashboard_state: crate::utils::dashboard::SharedDashboardState,
    account_update: SubscribeUpdateAccount,
) -> anyhow::Result<()> {
    if let Some(account) = &account_update.account {
        let account_key = bs58::encode(&account.pubkey).into_string();
        
        match deserialize_pool_account_simple(&account.data) {
            Ok(raw_pool_data) => {
                let pool = raw_pool_data.to_pool_data(account_key.clone(), account_update.slot);
                
                handle_pool_account_update(dashboard_state.clone(), pool.clone()).await;
                
                let state = dashboard_state.lock().await;
                let stats = get_pool_statistics(&pool, &state.token_prices);
                println!("{}", stats);
                
                println!("Pool account updated: {} (Slot: {})", 
                    &account_key[0..8], account_update.slot);
            }
            Err(e) => {
                println!("Account {} - Deserialization failed: {}", 
                    &account_key[0..8], e);
            }
        }
    }
    
    Ok(())
}

async fn handle_transaction_update(tx_update: SubscribeUpdateTransaction) {
    if let Some(transaction) = &tx_update.transaction {
        if let Some(meta) = &transaction.meta {
            let signature = "tx_sig".to_string();
            
            println!("Transaction: {} | Slot: {} | Success: {}", 
                signature, 
                tx_update.slot,
                meta.err.is_none()
            );
        }
    }
}

#[allow(dead_code)]
async fn perform_initial_snapshot(_dashboard_state: crate::utils::dashboard::SharedDashboardState) -> anyhow::Result<()> {
    println!("Performing initial snapshot of program accounts...");
    println!("Initial snapshot completed (streaming mode)");
    Ok(())
} 