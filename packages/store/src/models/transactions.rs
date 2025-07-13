use chrono::{DateTime, Utc};
use diesel::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub id: i64,
    pub pool_id: i32,
    pub protocol_id: i32,
    pub user_id: Option<i32>,
    pub tx_signature: String,
    pub tx_type: String,
    pub amount_in: Decimal,
    pub amount_out: Decimal,
    pub token_in: String,
    pub token_out: String,
    pub price: Option<Decimal>,
    pub fee: Decimal,
    pub slot: i64,
    pub block_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transactions)]
pub struct NewTransaction {
    pub pool_id: i32,
    pub protocol_id: i32,
    pub user_id: Option<i32>,
    pub tx_signature: String,
    pub tx_type: String,
    pub amount_in: Decimal,
    pub amount_out: Decimal,
    pub token_in: String,
    pub token_out: String,
    pub price: Option<Decimal>,
    pub fee: Decimal,
    pub slot: i64,
    pub block_time: DateTime<Utc>,
}

impl Transaction {
    pub fn new(
        pool_id: i32,
        protocol_id: i32,
        user_id: Option<i32>,
        tx_signature: String,
        tx_type: String,
        amount_in: Decimal,
        amount_out: Decimal,
        token_in: String,
        token_out: String,
        price: Option<Decimal>,
        fee: Decimal,
        slot: i64,
        block_time: DateTime<Utc>,
    ) -> NewTransaction {
        NewTransaction {
            pool_id,
            protocol_id,
            user_id,
            tx_signature,
            tx_type,
            amount_in,
            amount_out,
            token_in,
            token_out,
            price,
            fee,
            slot,
            block_time,
        }
    }
}
