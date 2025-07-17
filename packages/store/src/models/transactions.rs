use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// Main transaction table
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
    pub id: i32,
    pub signature: Vec<u8>,
    pub is_vote: bool,
    pub slot: i64,
    pub idx: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transactions)]
pub struct NewTransaction {
    pub signature: Vec<u8>,
    pub is_vote: bool,
    pub slot: i64,
    pub idx: i64,
}

// Sanitized transactions
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::sanitized_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SanitizedTransaction {
    pub id: i32,
    pub transaction_id: Option<i32>,
    pub message_hash: Vec<u8>,
    pub is_simple_vote_transaction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::sanitized_transactions)]
pub struct NewSanitizedTransaction {
    pub transaction_id: Option<i32>,
    pub message_hash: Vec<u8>,
    pub is_simple_vote_transaction: bool,
}

// Transaction signatures
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transaction_signatures)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionSignature {
    pub id: i32,
    pub sanitized_transaction_id: Option<i32>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transaction_signatures)]
pub struct NewTransactionSignature {
    pub sanitized_transaction_id: Option<i32>,
    pub signature: Vec<u8>,
}

// Transaction status meta
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transaction_status_meta)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionStatusMeta {
    pub id: i32,
    pub transaction_id: Option<i32>,
    pub is_status_err: bool,
    pub error_info: Option<String>,
    pub fee: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transaction_status_meta)]
pub struct NewTransactionStatusMeta {
    pub transaction_id: Option<i32>,
    pub is_status_err: bool,
    pub error_info: Option<String>,
    pub fee: i64,
}

// Transaction pre balances
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transaction_pre_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionPreBalance {
    pub id: i32,
    pub status_meta_id: Option<i32>,
    pub balance: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transaction_pre_balances)]
pub struct NewTransactionPreBalance {
    pub status_meta_id: Option<i32>,
    pub balance: i64,
}

// Transaction post balances
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transaction_post_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionPostBalance {
    pub id: i32,
    pub status_meta_id: Option<i32>,
    pub balance: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transaction_post_balances)]
pub struct NewTransactionPostBalance {
    pub status_meta_id: Option<i32>,
    pub balance: i64,
}

// Transaction log messages
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transaction_log_messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionLogMessage {
    pub id: i32,
    pub status_meta_id: Option<i32>,
    pub log_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transaction_log_messages)]
pub struct NewTransactionLogMessage {
    pub status_meta_id: Option<i32>,
    pub log_message: String,
}

// Transaction inner instructions
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transaction_inner_instructions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionInnerInstructions {
    pub id: i32,
    pub status_meta_id: Option<i32>,
    pub idx: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transaction_inner_instructions)]
pub struct NewTransactionInnerInstructions {
    pub status_meta_id: Option<i32>,
    pub idx: i32,
}

// Transaction inner instruction
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transaction_inner_instruction)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionInnerInstruction {
    pub id: i32,
    pub inner_instructions_id: Option<i32>,
    pub stack_height: Option<i32>,
    pub program_id_index: i32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transaction_inner_instruction)]
pub struct NewTransactionInnerInstruction {
    pub inner_instructions_id: Option<i32>,
    pub stack_height: Option<i32>,
    pub program_id_index: i32,
    pub data: Vec<u8>,
}

// Transaction pre token balances
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transaction_pre_token_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionPreTokenBalance {
    pub id: i32,
    pub status_meta_id: Option<i32>,
    pub account_index: i32,
    pub mint: String,
    pub owner: Option<String>,
    pub ui_amount: Option<f64>,
    pub decimals: Option<i32>,
    pub amount: Option<String>,
    pub ui_amount_string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transaction_pre_token_balances)]
pub struct NewTransactionPreTokenBalance {
    pub status_meta_id: Option<i32>,
    pub account_index: i32,
    pub mint: String,
    pub owner: Option<String>,
    pub ui_amount: Option<f64>,
    pub decimals: Option<i32>,
    pub amount: Option<String>,
    pub ui_amount_string: Option<String>,
}

// Transaction post token balances
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transaction_post_token_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionPostTokenBalance {
    pub id: i32,
    pub status_meta_id: Option<i32>,
    pub account_index: i32,
    pub mint: String,
    pub owner: Option<String>,
    pub ui_amount: Option<f64>,
    pub decimals: Option<i32>,
    pub amount: Option<String>,
    pub ui_amount_string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transaction_post_token_balances)]
pub struct NewTransactionPostTokenBalance {
    pub status_meta_id: Option<i32>,
    pub account_index: i32,
    pub mint: String,
    pub owner: Option<String>,
    pub ui_amount: Option<f64>,
    pub decimals: Option<i32>,
    pub amount: Option<String>,
    pub ui_amount_string: Option<String>,
}

// Transaction rewards
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::transaction_rewards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionReward {
    pub id: i32,
    pub status_meta_id: Option<i32>,
    pub pubkey: String,
    pub lamports: i64,
    pub post_balance: i64,
    pub reward_type: i32,
    pub commission: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::transaction_rewards)]
pub struct NewTransactionReward {
    pub status_meta_id: Option<i32>,
    pub pubkey: String,
    pub lamports: i64,
    pub post_balance: i64,
    pub reward_type: i32,
    pub commission: Option<i32>,
}

// Implementation methods
impl Transaction {
    pub fn new(
        signature: Vec<u8>,
        is_vote: bool,
        slot: i64,
        idx: i64,
    ) -> NewTransaction {
        NewTransaction {
            signature,
            is_vote,
            slot,
            idx,
        }
    }
}

impl SanitizedTransaction {
    pub fn new(
        transaction_id: Option<i32>,
        message_hash: Vec<u8>,
        is_simple_vote_transaction: bool,
    ) -> NewSanitizedTransaction {
        NewSanitizedTransaction {
            transaction_id,
            message_hash,
            is_simple_vote_transaction,
        }
    }
}

impl TransactionStatusMeta {
    pub fn new(
        transaction_id: Option<i32>,
        is_status_err: bool,
        error_info: Option<String>,
        fee: i64,
    ) -> NewTransactionStatusMeta {
        NewTransactionStatusMeta {
            transaction_id,
            is_status_err,
            error_info,
            fee,
        }
    }
}
