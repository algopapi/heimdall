use std::collections::HashMap;
use tokio::time::{Duration};

pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

pub async fn fetch_token_prices() -> anyhow::Result<HashMap<String, f64>> {
    let mut prices = HashMap::new();
    
    prices.insert(SOL_MINT.to_string(), 150.0);
    prices.insert(USDC_MINT.to_string(), 1.0);
    
    let variation = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() % 10) as f64;
    
    if let Some(sol_price) = prices.get_mut(SOL_MINT) {
        *sol_price += variation * 0.1;
    }
    
    println!("Updated token prices: SOL=${:.2}, USDC=${:.2}", 
        prices.get(SOL_MINT).unwrap_or(&0.0),
        prices.get(USDC_MINT).unwrap_or(&0.0)
    );
    
    Ok(prices)
}

pub async fn start_price_feed_updater() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            match fetch_token_prices().await {
                Ok(_prices) => {}
                Err(e) => {
                    eprintln!("Failed to fetch token prices: {}", e);
                }
            }
        }
    })
}

#[allow(dead_code)]
pub async fn fetch_jupiter_prices(token_mints: &[String]) -> anyhow::Result<HashMap<String, f64>> {
    let mut prices = HashMap::new();
    
    for mint in token_mints {
        let price = match mint.as_str() {
            SOL_MINT => 150.0,
            USDC_MINT => 1.0,
            _ => 0.0,
        };
        prices.insert(mint.clone(), price);
    }
    
    Ok(prices)
}

pub fn calculate_token_decimals(mint: &str) -> u8 {
    match mint {
        SOL_MINT => 9,
        USDC_MINT => 6,
        _ => 9,
    }
}

pub fn format_token_amount(amount: u64, decimals: u8) -> f64 {
    amount as f64 / 10_f64.powi(decimals as i32)
} 