use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bs58;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolData {
    pub pubkey: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub base_vault_balance: u64,
    pub quote_vault_balance: u64,
    pub lp_supply: u64,
    pub fee_numerator: u64,
    pub fee_denominator: u64,
    pub created_at: Option<i64>,
    pub last_updated_slot: u64,
}

#[derive(Debug, Clone)]
pub struct DashboardState {
    pub pools: HashMap<String, PoolData>,
    pub total_tvl: f64,
    pub pool_count: usize,
    pub top_pools_by_tvl: Vec<(String, f64)>,
    pub token_prices: HashMap<String, f64>,
    pub total_volume_24h: f64,
}

impl DashboardState {
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
            total_tvl: 0.0,
            pool_count: 0,
            top_pools_by_tvl: Vec::new(),
            token_prices: HashMap::new(),
            total_volume_24h: 0.0,
        }
    }

    pub fn add_or_update_pool(&mut self, pool: PoolData) {
        self.pools.insert(pool.pubkey.clone(), pool);
        self.pool_count = self.pools.len();
    }

    pub fn get_pool(&self, pubkey: &str) -> Option<&PoolData> {
        self.pools.get(pubkey)
    }

    pub fn print_summary(&self) {
        println!("\nDashboard Summary:");
        println!("  Total Pools: {}", self.pool_count);
        println!("  Total TVL: ${:.2}", self.total_tvl);
        println!("  24h Volume: ${:.2}", self.total_volume_24h);
        println!("  Top 5 Pools by TVL:");
        
        for (i, (pubkey, tvl)) in self.top_pools_by_tvl.iter().take(5).enumerate() {
            let short_key = if pubkey.len() > 8 {
                format!("{}...{}", &pubkey[0..4], &pubkey[pubkey.len()-4..])
            } else {
                pubkey.clone()
            };
            println!("    {}. {} - ${:.2}", i + 1, short_key, tvl);
        }
    }
}

#[derive(Debug)]
pub struct RawPoolData {
    pub base_mint: [u8; 32],
    pub quote_mint: [u8; 32],
    pub base_vault_balance: u64,
    pub quote_vault_balance: u64,
    pub lp_supply: u64,
    pub fee_numerator: u64,
    pub fee_denominator: u64,
}

impl RawPoolData {
    pub fn to_pool_data(&self, pubkey: String, slot: u64) -> PoolData {
        PoolData {
            pubkey,
            base_mint: bs58::encode(self.base_mint).into_string(),
            quote_mint: bs58::encode(self.quote_mint).into_string(),
            base_vault_balance: self.base_vault_balance,
            quote_vault_balance: self.quote_vault_balance,
            lp_supply: self.lp_supply,
            fee_numerator: self.fee_numerator,
            fee_denominator: self.fee_denominator,
            created_at: None,
            last_updated_slot: slot,
        }
    }
} 