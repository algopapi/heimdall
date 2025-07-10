use crate::models::pool_data::{DashboardState, PoolData};
use crate::feeds::price_feeds::{fetch_token_prices, calculate_token_decimals, format_token_amount};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type SharedDashboardState = Arc<Mutex<DashboardState>>;

pub async fn calculate_dashboard_aggregates(dashboard_state: &mut DashboardState) {
    if let Ok(prices) = fetch_token_prices().await {
        dashboard_state.token_prices = prices;
    }

    let mut total_tvl = 0.0;
    let mut pool_tvls = Vec::new();

    for (pubkey, pool) in &dashboard_state.pools {
        let pool_tvl = calculate_pool_tvl(pool, &dashboard_state.token_prices);
        total_tvl += pool_tvl;
        pool_tvls.push((pubkey.clone(), pool_tvl));
    }

    pool_tvls.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    dashboard_state.total_tvl = total_tvl;
    dashboard_state.pool_count = dashboard_state.pools.len();
    dashboard_state.top_pools_by_tvl = pool_tvls.into_iter().take(10).collect();
}

pub fn calculate_pool_tvl(pool: &PoolData, token_prices: &std::collections::HashMap<String, f64>) -> f64 {
    let base_price = token_prices.get(&pool.base_mint).unwrap_or(&0.0);
    let quote_price = token_prices.get(&pool.quote_mint).unwrap_or(&0.0);

    let base_decimals = calculate_token_decimals(&pool.base_mint);
    let quote_decimals = calculate_token_decimals(&pool.quote_mint);

    let base_amount = format_token_amount(pool.base_vault_balance, base_decimals);
    let quote_amount = format_token_amount(pool.quote_vault_balance, quote_decimals);

    let base_value = base_amount * base_price;
    let quote_value = quote_amount * quote_price;

    base_value + quote_value
}

pub async fn update_dashboard_periodically(dashboard_state: SharedDashboardState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    
    loop {
        interval.tick().await;
        
        {
            let mut state = dashboard_state.lock().await;
            calculate_dashboard_aggregates(&mut state).await;
            
            state.print_summary();
        }
    }
}

pub async fn handle_pool_account_update(
    dashboard_state: SharedDashboardState,
    pool: PoolData,
) {
    let mut state = dashboard_state.lock().await;
    state.add_or_update_pool(pool);
    
    calculate_dashboard_aggregates(&mut state).await;
}

pub fn create_shared_dashboard_state() -> SharedDashboardState {
    Arc::new(Mutex::new(DashboardState::new()))
}

pub fn get_top_pools_by_volume(
    dashboard_state: &DashboardState, 
    limit: usize
) -> Vec<(String, f64)> {
    dashboard_state.top_pools_by_tvl.iter()
        .take(limit)
        .cloned()
        .collect()
}

pub fn get_pool_statistics(pool: &PoolData, token_prices: &std::collections::HashMap<String, f64>) -> String {
    let tvl = calculate_pool_tvl(pool, token_prices);
    let base_decimals = calculate_token_decimals(&pool.base_mint);
    let quote_decimals = calculate_token_decimals(&pool.quote_mint);
    
    let base_amount = format_token_amount(pool.base_vault_balance, base_decimals);
    let quote_amount = format_token_amount(pool.quote_vault_balance, quote_decimals);
    
    format!(
        "Pool {} | TVL: ${:.2} | Base: {:.4} | Quote: {:.4} | LP Supply: {} | Fee: {}/{}",
        &pool.pubkey[0..8],
        tvl,
        base_amount,
        quote_amount,
        pool.lp_supply,
        pool.fee_numerator,
        pool.fee_denominator
    )
} 