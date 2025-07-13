use chrono::{DateTime, Utc};
use diesel::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Order {
    pub id: i64,
    pub user_id: i32,
    pub pool_id: i32,
    pub protocol_id: i32,
    pub order_type: String,
    pub side: String,
    pub price: Option<Decimal>,
    pub amount: Decimal,
    pub filled_amount: Decimal,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::orders)]
pub struct NewOrder {
    pub user_id: i32,
    pub pool_id: i32,
    pub protocol_id: i32,
    pub order_type: String,
    pub side: String,
    pub price: Option<Decimal>,
    pub amount: Decimal,
    pub filled_amount: Decimal,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Order {
    pub fn new(
        user_id: i32,
        pool_id: i32,
        protocol_id: i32,
        order_type: String,
        side: String,
        price: Option<Decimal>,
        amount: Decimal,
        filled_amount: Decimal,
        status: String,
        expires_at: Option<DateTime<Utc>>,
    ) -> NewOrder {
        NewOrder {
            user_id,
            pool_id,
            protocol_id,
            order_type,
            side,
            price,
            amount,
            filled_amount,
            status,
            expires_at,
        }
    }
}
