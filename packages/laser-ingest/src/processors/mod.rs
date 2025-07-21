use async_trait::async_trait;
use helius_laserstream::grpc::{SubscribeUpdate, SubscribeRequestFilterAccounts, SubscribeRequestFilterTransactions};
use redis::aio::MultiplexedConnection as RedisConn;

use std::collections::HashMap;

use crate::types::{PoolMeta, PoolVariant};

pub mod events;
pub mod dbc;
pub mod amm;
pub mod damm;

/// Contribution of filters from a processor for a particular pool.
pub struct FilterContribution {
    pub accounts: HashMap<String, SubscribeRequestFilterAccounts>,
    pub transactions: HashMap<String, SubscribeRequestFilterTransactions>,
}

#[async_trait]
pub trait Processor: Send + Sync {
    /// Build Helius filters required to receive updates for this pool.
    fn build_filters(&self, pool: &PoolMeta) -> FilterContribution;

    /// Process an incoming update. Implementations must publish to Redis.
    async fn handle_update(
        &self,
        pool: &PoolMeta,
        update: &SubscribeUpdate,
        conn: &mut RedisConn,
    );
}

/// Registry helper
pub fn default_registry() -> HashMap<PoolVariant, Box<dyn Processor>> {
    use std::iter::FromIterator;
    HashMap::from_iter([
        (PoolVariant::Dbc, Box::new(dbc::DbcProcessor) as Box<dyn Processor>),
        (PoolVariant::Amm, Box::new(amm::AmmProcessor) as Box<dyn Processor>),
        (PoolVariant::Damm, Box::new(damm::DammProcessor) as Box<dyn Processor>),
    ])
}

/// A no-op processor which can be used as a placeholder.
pub struct NoopProcessor;

#[async_trait]
impl Processor for NoopProcessor {
    fn build_filters(&self, _pool: &PoolMeta) -> FilterContribution {
        FilterContribution { accounts: HashMap::new(), transactions: HashMap::new() }
    }

    async fn handle_update(&self, _pool: &PoolMeta, _update: &SubscribeUpdate, _conn: &mut RedisConn) {}
} 