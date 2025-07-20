use futures_util::{pin_mut, StreamExt};
use helius_laserstream::{
    subscribe,
    grpc::{
        subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
        SubscribeRequestFilterAccounts, SubscribeRequestFilterTransactions, Transaction,
        TransactionStatusMeta,
    },
    LaserstreamConfig,
};
use std::collections::HashMap;
use borsh::{BorshDeserialize};
use bs58;

// Import the IDL-derived structs and discriminator
mod idl;
use idl::{EvtSwap, SwapResult, SWAP_EVENT_DISCRIMINATOR};

const ANCHOR_CPI_LOG_DISCRIMINATOR: [u8; 8] = [228, 69, 165, 46, 81, 203, 154, 29]; // e445a52e51cb9a1d

// We'll hold the state in memory.
// In a real app, this might be in Redis or a database.
static mut VIRTUAL_RESERVES: u128 = 0;
static mut REAL_RESERVES: u128 = 0;
static mut QUOTE_TOTAL_TX: u128 = 0;

// Token decimals - in a real app, you would fetch these once via RPC
// and store them. For this example, we hardcode them.
const BASE_TOKEN_DECIMALS: u32 = 6; // Assuming 6 for the new token
const QUOTE_TOKEN_DECIMALS: u32 = 9; // WSOL always has 9

// constants
const QUOTE_VAULT_PDA: &str = "HYoG7bCPQeXJ3LeRtfFxAvKixGLb9oeSvPeEF44mGanT";
const CONFIG_PDA: &str = "JGtpWKXxqYHNgRNRn5ECtdLSC4zX37tB5rQqrQ9Kps2";
const DBC_PROGRAM_ID: &str = "dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN";

/// Finds a swap event by searching through the transaction's inner instructions
/// for a specific Anchor self-CPI log pattern. This is the robust method for
/// programs that emit events via CPI instead of standard program logs.
fn find_swap_event_in_tx(
    tx: &Transaction,
    meta: &TransactionStatusMeta,
) -> Option<SwapResult> {
    let account_keys = tx.message.as_ref()?.account_keys.as_slice();
    let inner_instructions = meta.inner_instructions.as_slice();
    let dbc_program_bytes = bs58::decode(DBC_PROGRAM_ID).into_vec().ok()?;

    for inner_ix_list in inner_instructions {
        for ix in &inner_ix_list.instructions {
            let program_id_index = ix.program_id_index as usize;

            if let Some(program_id_bytes) = account_keys.get(program_id_index) {
                if *program_id_bytes == dbc_program_bytes {
                    // This is a CPI to our program. Check for Anchor's self-CPI log signature.
                    if ix.data.starts_with(&ANCHOR_CPI_LOG_DISCRIMINATOR) {
                        let event_with_disc = &ix.data[8..];
                        // Now check for our EvtSwap discriminator inside the CPI data.
                        if event_with_disc.starts_with(&SWAP_EVENT_DISCRIMINATOR) {
                            let event_data = &event_with_disc[8..];
                            if let Ok(evt) = EvtSwap::try_from_slice(event_data) {
                                return Some(evt.swap_result);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("HELIUS_API_KEY")?;
    // Ensure you set HELIUS_ENDPOINT for devnet, otherwise it defaults to mainnet
    let endpoint = std::env::var("HELIUS_ENDPOINT")
        .unwrap_or_else(|_| "https://laserstream-devnet.helius-rpc.com".to_string());
    
    let dbc_program_bytes = bs58::decode(DBC_PROGRAM_ID).into_vec()?;

    let config = LaserstreamConfig {
        api_key,
        endpoint: endpoint.parse()?,
        ..Default::default()
    };

    // -------------------------------------------------
    // Build SubscribeRequest to listen to the pool's QUOTE VAULT.
    // This account's balance IS the total quote deposited.
    // -------------------------------------------------
    let mut accounts_to_subscribe = HashMap::new();
    accounts_to_subscribe.insert(
        "quote_vault".to_owned(),
        SubscribeRequestFilterAccounts { account: vec![QUOTE_VAULT_PDA.to_owned()], ..Default::default() }
    );

    // A transaction filter is now optional, but useful as a trigger.
    let mut tx_filters = HashMap::new();
    tx_filters.insert(
        "pool_swaps".to_owned(),
        SubscribeRequestFilterTransactions {
            account_include: vec![DBC_PROGRAM_ID.to_owned()],
            ..Default::default()
        },
    );

    let req = SubscribeRequest {
        accounts: accounts_to_subscribe,
        slots: HashMap::new(),
        transactions: tx_filters,
        blocks: HashMap::new(),
        blocks_meta: HashMap::new(),
        commitment: Some(CommitmentLevel::Processed as i32),
        ..Default::default()
    };

    // -------------------------------------------------
    // Connect and stream
    // -------------------------------------------------
    println!("Connecting to LaserStream at {} ...", endpoint);
    let stream = subscribe(config, req);
    pin_mut!(stream);

    while let Some(message) = stream.next().await {
        match message {
            Ok(mut update) => {
                if let Some(oneof) = update.update_oneof.take() {
                    match oneof {
                        UpdateOneof::Account(acc_update) => {
                            if let Some(acc) = acc_update.account {
                                // A standard SPL Token Account's balance is a u64 at byte offset 64.
                                if acc.data.len() >= 72 {
                                    let total_quote_deposited_raw = u64::from_le_bytes(
                                        acc.data[64..72].try_into().unwrap(),
                                    );
                                    
                                    // Adjust for decimals to get a human-readable amount
                                    let total_quote_deposited = total_quote_deposited_raw as f64 / 10u64.pow(QUOTE_TOKEN_DECIMALS) as f64;

                                    println!("--------------------------------------------------");
                                    println!("-> TOTAL QUOTE DEPOSITED: {:.6} WSOL", total_quote_deposited);
                                    println!("--------------------------------------------------");
                                }
                            }
                        }
                        UpdateOneof::Transaction(tx_update) => {
                            if let Some(tx_info) = &tx_update.transaction {
                                if let (Some(tx), Some(meta)) =
                                    (&tx_info.transaction, &tx_info.meta)
                                {
                                    if let Some(event) = find_swap_event_in_tx(tx, meta) {
                                        let decimal_adjustment = 10f64.powi(
                                            BASE_TOKEN_DECIMALS as i32 - QUOTE_TOKEN_DECIMALS as i32,
                                        );
                                        let effective_price = if event.actual_input_amount > 0 {
                                            (event.output_amount as f64
                                                / event.actual_input_amount as f64)
                                                * decimal_adjustment
                                        } else {
                                            0.0
                                        };

                                        println!("------------------[ SWAP EVENT ]------------------");
                                        println!(
                                            "  ->      TX SIG: {}",
                                            bs58::encode(&tx_info.signature).into_string()
                                        );
                                        println!(
                                            "  -> Input Amount:      {}",
                                            event.actual_input_amount
                                        );
                                        println!(
                                            "  -> Output Amount:     {}",
                                            event.output_amount
                                        );
                                        println!("  -> Effective Price:   {:.12}", effective_price);
                                        println!("  -> Next Sqrt Price:   {}", event.next_sqrt_price);
                                        println!(
                                            "--------------------------------------------------------------------"
                                        );
                                    } else {
                                        // It is normal for many transactions not to be swaps.
                                        // To reduce noise, we won't log these by default.
                                    }
                                }
                            } else {
                                println!("[WARN] Received a Transaction update with no transaction info.");
                            }
                        }
                        _ => {
                            // Heartbeat can be ignored for cleaner output
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Stream error: {err}");
                break;
            }
        }
    }

    Ok(())
} 