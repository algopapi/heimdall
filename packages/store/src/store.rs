use crate::models::{
    accounts::{Account, NewAccount},
    slots::{NewSlot, Slot},
    transactions::{
        NewSanitizedTransaction, NewTransaction, NewTransactionSignature, NewTransactionStatusMeta,
        SanitizedTransaction, Transaction, TransactionSignature, TransactionStatusMeta,
    },
};
use diesel::{Connection, PgConnection, RunQueryDsl};

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
    // Account operations
    pub fn create_account(
        &mut self,
        slot: i64,
        pubkey: Vec<u8>,
        lamports: i64,
        owner: Vec<u8>,
        executable: bool,
        rent_epoch: i64,
        data: Option<Vec<u8>>,
        write_version: i64,
        txn_signature: Option<Vec<u8>>,
    ) -> Result<Account, diesel::result::Error> {
        use crate::schema::accounts;

        let new_account = NewAccount {
            slot,
            pubkey,
            lamports,
            owner,
            executable,
            rent_epoch,
            data,
            write_version,
            txn_signature,
        };

        diesel::insert_into(accounts::table)
            .values(&new_account)
            .get_result(&mut self.conn)
    }

    pub fn get_accounts(&mut self) -> Result<Vec<Account>, diesel::result::Error> {
        use crate::schema::accounts::dsl::*;

        accounts.load(&mut self.conn)
    }

    // Slot operations
    pub fn create_slot(
        &mut self,
        slot: i64,
        parent: Option<i64>,
        status: i32,
    ) -> Result<Slot, diesel::result::Error> {
        use crate::schema::slots;

        let new_slot = NewSlot {
            slot,
            parent,
            status,
        };

        diesel::insert_into(slots::table)
            .values(&new_slot)
            .get_result(&mut self.conn)
    }

    pub fn get_slots(&mut self) -> Result<Vec<Slot>, diesel::result::Error> {
        use crate::schema::slots::dsl::*;

        slots.load(&mut self.conn)
    }

    // Transaction operations
    pub fn create_transaction(
        &mut self,
        signature: Vec<u8>,
        is_vote: bool,
        slot: i64,
        idx: i64,
    ) -> Result<Transaction, diesel::result::Error> {
        use crate::schema::transactions;

        let new_transaction = NewTransaction {
            signature,
            is_vote,
            slot,
            idx,
        };

        diesel::insert_into(transactions::table)
            .values(&new_transaction)
            .get_result(&mut self.conn)
    }

    pub fn get_transactions(&mut self) -> Result<Vec<Transaction>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;

        transactions.load(&mut self.conn)
    }

    // Sanitized transaction operations
    pub fn create_sanitized_transaction(
        &mut self,
        transaction_id: Option<i32>,
        message_hash: Vec<u8>,
        is_simple_vote_transaction: bool,
    ) -> Result<SanitizedTransaction, diesel::result::Error> {
        use crate::schema::sanitized_transactions;

        let new_sanitized_transaction = NewSanitizedTransaction {
            transaction_id,
            message_hash,
            is_simple_vote_transaction,
        };

        diesel::insert_into(sanitized_transactions::table)
            .values(&new_sanitized_transaction)
            .get_result(&mut self.conn)
    }

    pub fn get_sanitized_transactions(
        &mut self,
    ) -> Result<Vec<SanitizedTransaction>, diesel::result::Error> {
        use crate::schema::sanitized_transactions::dsl::*;

        sanitized_transactions.load(&mut self.conn)
    }

    // Transaction signature operations
    pub fn create_transaction_signature(
        &mut self,
        sanitized_transaction_id: Option<i32>,
        signature: Vec<u8>,
    ) -> Result<TransactionSignature, diesel::result::Error> {
        use crate::schema::transaction_signatures;

        let new_transaction_signature = NewTransactionSignature {
            sanitized_transaction_id,
            signature,
        };

        diesel::insert_into(transaction_signatures::table)
            .values(&new_transaction_signature)
            .get_result(&mut self.conn)
    }

    pub fn get_transaction_signatures(
        &mut self,
    ) -> Result<Vec<TransactionSignature>, diesel::result::Error> {
        use crate::schema::transaction_signatures::dsl::*;

        transaction_signatures.load(&mut self.conn)
    }

    // Transaction status meta operations
    pub fn create_transaction_status_meta(
        &mut self,
        transaction_id: Option<i32>,
        is_status_err: bool,
        error_info: Option<String>,
        fee: i64,
    ) -> Result<TransactionStatusMeta, diesel::result::Error> {
        use crate::schema::transaction_status_meta;

        let new_transaction_status_meta = NewTransactionStatusMeta {
            transaction_id,
            is_status_err,
            error_info,
            fee,
        };

        diesel::insert_into(transaction_status_meta::table)
            .values(&new_transaction_status_meta)
            .get_result(&mut self.conn)
    }

    pub fn get_transaction_status_meta(
        &mut self,
    ) -> Result<Vec<TransactionStatusMeta>, diesel::result::Error> {
        use crate::schema::transaction_status_meta::dsl::*;

        transaction_status_meta.load(&mut self.conn)
    }
}
