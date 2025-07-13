use crate::models::{
    orders::{NewOrder, Order},
    pools::{NewPool, Pool},
    protocols::{NewProtocol, Protocol},
    transactions::{NewTransaction, Transaction},
    user::{NewUser, User},
    user_swaps::{NewUserSwap, UserSwap},
};
use chrono::{DateTime, Utc};
use diesel::{Connection, PgConnection, RunQueryDsl};
use rust_decimal::Decimal;

use crate::config::Config;

pub struct Store {
    pub conn: PgConnection,
}

impl Default for Store {
    fn default() -> Self {
        let database_url = Config::default().db_url;

        let connection =
            PgConnection::establish(&database_url).expect("Error connecting to the database");

        Self { conn: connection }
    }
}

impl Store {
    pub fn create_user(
        &mut self,
        pubkey: String,
        signature: Option<String>,
    ) -> Result<User, diesel::result::Error> {
        use crate::schema::users;

        let new_user = NewUser { pubkey, signature };

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(&mut self.conn)
    }

    pub fn create_protocol(
        &mut self,
        name: String,
        program_id: String,
        description: Option<String>,
        is_active: bool,
    ) -> Result<Protocol, diesel::result::Error> {
        use crate::schema::protocols;

        let new_protocol = NewProtocol {
            name,
            program_id,
            description,
            is_active,
        };

        diesel::insert_into(protocols::table)
            .values(&new_protocol)
            .get_result(&mut self.conn)
    }

    pub fn create_pool(
        &mut self,
        protocol_id: i32,
        pool_pubkey: String,
        base_mint: String,
        quote_mint: String,
        base_decimals: i16,
        quote_decimals: i16,
        fee_numerator: i64,
        fee_denominator: i64,
        is_active: bool,
    ) -> Result<Pool, diesel::result::Error> {
        use crate::schema::pools;

        let new_pool = NewPool {
            protocol_id,
            pool_pubkey,
            base_mint,
            quote_mint,
            base_decimals,
            quote_decimals,
            fee_numerator,
            fee_denominator,
            is_active,
        };

        diesel::insert_into(pools::table)
            .values(&new_pool)
            .get_result(&mut self.conn)
    }

    pub fn create_transaction(
        &mut self,
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
    ) -> Result<Transaction, diesel::result::Error> {
        use crate::schema::transactions;

        let new_transaction = NewTransaction {
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
        };

        diesel::insert_into(transactions::table)
            .values(&new_transaction)
            .get_result(&mut self.conn)
    }

    pub fn create_user_swap(
        &mut self,
        user_id: i32,
        tx_id: i64,
    ) -> Result<UserSwap, diesel::result::Error> {
        use crate::schema::user_swaps;

        let new_user_swap = NewUserSwap { user_id, tx_id };

        diesel::insert_into(user_swaps::table)
            .values(&new_user_swap)
            .get_result(&mut self.conn)
    }

    pub fn create_order(
        &mut self,
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
    ) -> Result<Order, diesel::result::Error> {
        use crate::schema::orders;

        let new_order = NewOrder {
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
        };

        diesel::insert_into(orders::table)
            .values(&new_order)
            .get_result(&mut self.conn)
    }
}
