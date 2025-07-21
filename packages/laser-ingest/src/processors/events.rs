use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct DbcSwapPayload {
    pub signature: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub next_sqrt_price: u128,
}

#[derive(Serialize, Debug)]
pub struct DbcBalanceUpdatePayload {
    pub quote_vault_address: String,
    pub new_balance: f64,
}

#[derive(Serialize, Debug)]
#[serde(tag = "event_type", content = "payload")]
#[serde(rename_all = "snake_case")]
pub enum PoolEvent {
    DbcSwap(DbcSwapPayload),
    DbcBalanceUpdate(DbcBalanceUpdatePayload),
    // Future variants like AmmSwap, etc. would go here
}

#[derive(Serialize, Debug)]
pub struct StreamedEvent<'a> {
    pub pool_id: &'a str,
    pub variant: &'a str,
    #[serde(flatten)]
    pub event: PoolEvent,
} 