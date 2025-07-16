use serde_json::Value;

pub struct ParsedAccount {
    pub account_stream: String,
    pub account_pubkey: String,
    pub data_json: Value,
}

pub struct ParsedEvent {
    pub event_stream: String,
    pub event_name: String,
    pub transaction_signature: String,
    pub signers: Vec<String>,
    pub data_json: Value,
}

#[derive(Debug)]
pub enum RedisMessage {
    AccountUpdate {
        stream: String,
        account_pubkey: String,
        data: Value,
        slot: u64,
    },
    Event {
        stream: String,
        signature: String,
        signers: Vec<String>,
        data: Value,
        slot: u64,
    },
}
