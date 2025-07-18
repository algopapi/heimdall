use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use std::collections::VecDeque;
use store::Store;
use tokio::time::{Duration, Instant};
use tracing::{error, info};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AccountUpdate {
    pub slot: u64,
    pub pubkey: String,
    pub lamports: u64,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: Option<String>,
    pub write_version: u64,
    pub txn_signature: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SlotUpdate {
    pub slot: u64,
    pub parent: u64,
    pub status: u32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TransactionUpdate {
    pub signature: String,
    pub is_vote: bool,
    pub slot: u64,
    pub index: u64,
}

pub struct DatabaseProcessor {
    store: Store,
    account_buffer: VecDeque<AccountUpdate>,
    slot_buffer: VecDeque<SlotUpdate>,
    transaction_buffer: VecDeque<TransactionUpdate>,
    buffer_size_limit: usize,
    last_flush: Instant,
    flush_interval: Duration,
}

impl DatabaseProcessor {
    pub fn new() -> Self {
        Self {
            store: Store::default(),
            account_buffer: VecDeque::new(),
            slot_buffer: VecDeque::new(),
            transaction_buffer: VecDeque::new(),
            buffer_size_limit: 1000,
            last_flush: Instant::now(),
            flush_interval: Duration::from_secs(5),
        }
    }

    pub async fn store_account_update(&mut self, event: AccountUpdate) -> Result<()> {
        self.account_buffer.push_back(event);
        info!(
            "Added account update to buffer (buffer size: {})",
            self.account_buffer.len()
        );
        self.check_and_flush().await?;
        Ok(())
    }

    pub async fn store_slot_update(&mut self, event: SlotUpdate) -> Result<()> {
        self.slot_buffer.push_back(event);
        info!(
            "Added slot update to buffer (buffer size: {})",
            self.slot_buffer.len()
        );
        self.check_and_flush().await?;
        Ok(())
    }

    pub async fn store_transaction_update(&mut self, event: TransactionUpdate) -> Result<()> {
        self.transaction_buffer.push_back(event);
        info!(
            "Added transaction update to buffer (buffer size: {})",
            self.transaction_buffer.len()
        );
        self.check_and_flush().await?;
        Ok(())
    }

    async fn check_and_flush(&mut self) -> Result<()> {
        let total_buffered =
            self.account_buffer.len() + self.slot_buffer.len() + self.transaction_buffer.len();
        let time_since_last_flush = self.last_flush.elapsed();
        if total_buffered >= self.buffer_size_limit || time_since_last_flush >= self.flush_interval
        {
            info!(
                "Flushing buffers: {} total items, {}s since last flush",
                total_buffered,
                time_since_last_flush.as_secs()
            );
            self.flush_all_buffers().await?;
            self.last_flush = Instant::now();
        }
        Ok(())
    }

    async fn flush_all_buffers(&mut self) -> Result<()> {
        info!(
            "Starting batch flush: {} accounts, {} slots, {} transactions",
            self.account_buffer.len(),
            self.slot_buffer.len(),
            self.transaction_buffer.len()
        );
        while let Some(event) = self.account_buffer.pop_front() {
            if let Err(e) = self.process_account_update(event).await {
                error!("Failed to process buffered account update: {}", e);
            }
        }
        while let Some(event) = self.slot_buffer.pop_front() {
            if let Err(e) = self.process_slot_update(event).await {
                error!("Failed to process buffered slot update: {}", e);
            }
        }
        while let Some(event) = self.transaction_buffer.pop_front() {
            if let Err(e) = self.process_transaction_update(event).await {
                error!("Failed to process buffered transaction update: {}", e);
            }
        }
        info!("Successfully flushed all buffers");
        Ok(())
    }

    async fn process_account_update(&mut self, event: AccountUpdate) -> Result<()> {
        let pubkey =
            hex::decode(&event.pubkey).map_err(|e| anyhow::anyhow!("Invalid pubkey hex: {}", e))?;
        let owner =
            hex::decode(&event.owner).map_err(|e| anyhow::anyhow!("Invalid owner hex: {}", e))?;
        let data = if let Some(data_str) = &event.data {
            if data_str.is_empty() {
                None
            } else {
                Some(
                    general_purpose::STANDARD
                        .decode(data_str)
                        .map_err(|e| anyhow::anyhow!("Invalid data base64: {}", e))?,
                )
            }
        } else {
            None
        };
        let txn_signature = if let Some(sig_str) = &event.txn_signature {
            if sig_str.is_empty() {
                None
            } else {
                Some(
                    hex::decode(sig_str)
                        .map_err(|e| anyhow::anyhow!("Invalid signature hex: {}", e))?,
                )
            }
        } else {
            None
        };
        let result = self.store.create_account(
            event.slot as i64,
            pubkey,
            event.lamports as i64,
            owner,
            event.executable,
            event.rent_epoch as i64,
            data,
            event.write_version as i64,
            txn_signature,
        );
        match result {
            Ok(_) => {
                info!("Successfully stored account update for slot {}", event.slot);
                Ok(())
            }
            Err(e) => {
                error!("Failed to store account update: {}", e);
                Err(e.into())
            }
        }
    }

    async fn process_slot_update(&mut self, event: SlotUpdate) -> Result<()> {
        let result = self.store.create_slot(
            event.slot as i64,
            if event.parent == 0 {
                None
            } else {
                Some(event.parent as i64)
            },
            event.status as i32,
        );
        match result {
            Ok(_) => {
                info!("Successfully stored slot update for slot {}", event.slot);
                Ok(())
            }
            Err(e) => {
                error!("Failed to store slot update: {}", e);
                Err(e.into())
            }
        }
    }

    async fn process_transaction_update(&mut self, event: TransactionUpdate) -> Result<()> {
        let signature = hex::decode(&event.signature)
            .map_err(|e| anyhow::anyhow!("Invalid signature hex: {}", e))?;
        let result = self.store.create_transaction(
            signature,
            event.is_vote,
            event.slot as i64,
            event.index as i64,
        );
        match result {
            Ok(_) => {
                info!(
                    "Successfully stored transaction update for slot {}",
                    event.slot
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to store transaction update: {}", e);
                Err(e.into())
            }
        }
    }
}

impl Default for DatabaseProcessor {
    fn default() -> Self {
        Self::new()
    }
}
