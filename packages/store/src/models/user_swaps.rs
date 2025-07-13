use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::user_swaps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserSwap {
    pub id: i64,
    pub user_id: i32,
    pub tx_id: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::user_swaps)]
pub struct NewUserSwap {
    pub user_id: i32,
    pub tx_id: i64,
}

impl UserSwap {
    pub fn new(user_id: i32, tx_id: i64) -> NewUserSwap {
        NewUserSwap { user_id, tx_id }
    }
}
