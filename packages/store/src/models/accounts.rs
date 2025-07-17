use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::accounts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Account {
    pub id: i32,
    pub slot: i64,
    pub pubkey: Vec<u8>,
    pub lamports: i64,
    pub owner: Vec<u8>,
    pub executable: bool,
    pub rent_epoch: i64,
    pub data: Option<Vec<u8>>,
    pub write_version: i64,
    pub txn_signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::accounts)]
pub struct NewAccount {
    pub slot: i64,
    pub pubkey: Vec<u8>,
    pub lamports: i64,
    pub owner: Vec<u8>,
    pub executable: bool,
    pub rent_epoch: i64,
    pub data: Option<Vec<u8>>,
    pub write_version: i64,
    pub txn_signature: Option<Vec<u8>>,
}

impl Account {
    pub fn new(
        slot: i64,
        pubkey: Vec<u8>,
        lamports: i64,
        owner: Vec<u8>,
        executable: bool,
        rent_epoch: i64,
        data: Option<Vec<u8>>,
        write_version: i64,
        txn_signature: Option<Vec<u8>>,
    ) -> NewAccount {
        NewAccount {
            slot,
            pubkey,
            lamports,
            owner,
            executable,
            rent_epoch,
            data,
            write_version,
            txn_signature,
        }
    }
}
