
use borsh::{BorshDeserialize};

// From IDL: EvtSwap discriminator
pub const SWAP_EVENT_DISCRIMINATOR: [u8; 8] = [27, 60, 21, 213, 138, 170, 187, 147];

#[derive(BorshDeserialize, Debug, Clone)]
pub struct SwapParameters {
    pub amount_in: u64,
    pub minimum_amount_out: u64,
}

#[derive(BorshDeserialize, Debug, Clone)]
pub struct SwapResult {
    pub actual_input_amount: u64,
    // NOTE: This was the source of the original error. The IDL specifies u64,
    // but the previous code used u128, which caused deserialization to fail.
    pub output_amount: u64,
    pub next_sqrt_price: u128,
    pub trading_fee: u64,
    pub protocol_fee: u64,
    pub referral_fee: u64,
}

/// This represents the full event structure for an `EvtSwap` as defined in the IDL.
/// We deserialize into this struct first, then extract the `swap_result` field.
#[derive(BorshDeserialize, Debug)]
pub struct EvtSwap {
    // We use [u8; 32] which is the byte representation of a Pubkey.
    // This is simpler for Borsh deserialization than handling the Pubkey type directly.
    pub pool: [u8; 32],
    pub config: [u8; 32],
    pub trade_direction: u8,
    pub has_referral: bool,
    pub params: SwapParameters,
    pub swap_result: SwapResult,
    pub amount_in: u64,
    pub current_timestamp: u64,
} 