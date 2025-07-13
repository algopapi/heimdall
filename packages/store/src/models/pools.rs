use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::pools)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Pool {
    pub id: i32,
    pub protocol_id: i32,
    pub pool_pubkey: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub base_decimals: i16,
    pub quote_decimals: i16,
    pub fee_numerator: i64,
    pub fee_denominator: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::pools)]
pub struct NewPool {
    pub protocol_id: i32,
    pub pool_pubkey: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub base_decimals: i16,
    pub quote_decimals: i16,
    pub fee_numerator: i64,
    pub fee_denominator: i64,
    pub is_active: bool,
}

impl Pool {
    pub fn new(
        protocol_id: i32,
        pool_pubkey: String,
        base_mint: String,
        quote_mint: String,
        base_decimals: i16,
        quote_decimals: i16,
        fee_numerator: i64,
        fee_denominator: i64,
        is_active: bool,
    ) -> NewPool {
        NewPool {
            protocol_id,
            pool_pubkey,
            base_mint,
            quote_mint,
            base_decimals,
            quote_decimals,
            fee_numerator,
            fee_denominator,
            is_active,
        }
    }
}
