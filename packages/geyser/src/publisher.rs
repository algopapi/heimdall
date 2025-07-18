use {
    crate::{
        message_wrapper::EventMessage::{self, Account, Slot, Transaction},
        prom::{UPLOAD_ACCOUNTS_TOTAL, UPLOAD_SLOTS_TOTAL, UPLOAD_TRANSACTIONS_TOTAL},
        Config, MessageWrapper, SlotStatusEvent, TransactionEvent, UpdateAccountEvent,
    },
    prost::Message,
    redis::{AsyncCommands, RedisError},
    std::time::Duration,
};

#[allow(dead_code)]
pub struct Publisher {
    client: redis::Client,
    shutdown_timeout: Duration,
}

impl Publisher {
    pub fn new(client: redis::Client, config: &Config) -> Self {
        Self {
            client,
            shutdown_timeout: Duration::from_millis(config.shutdown_timeout_ms),
        }
    }

    pub async fn update_account(
        &self,
        ev: UpdateAccountEvent,
        wrap_messages: bool,
        stream: &str,
    ) -> Result<(), RedisError> {
        if stream.is_empty() {
            return Ok(());
        }

        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let (key, data) = if wrap_messages {
            let key = self.create_key_with_prefix(&ev.pubkey, 65u8);
            let buf = Self::encode_with_wrapper(Account(Box::new(ev)));
            (key, buf)
        } else {
            (hex::encode(&ev.pubkey), ev.encode_to_vec())
        };

        let fields = vec![("key", key.as_bytes()), ("data", &data)];

        let _: String = conn.xadd(stream, "*", &fields).await?;

        UPLOAD_ACCOUNTS_TOTAL.with_label_values(&["success"]).inc();

        Ok(())
    }

    pub async fn update_slot_status(
        &self,
        ev: SlotStatusEvent,
        wrap_messages: bool,
        stream: &str,
    ) -> Result<(), RedisError> {
        if stream.is_empty() {
            return Ok(());
        }

        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let (key, data) = if wrap_messages {
            let slot_bytes = ev.slot.to_le_bytes();
            let key = self.create_key_with_prefix(&slot_bytes, 83u8);
            let buf = Self::encode_with_wrapper(Slot(Box::new(ev)));
            (key, buf)
        } else {
            (ev.slot.to_string(), ev.encode_to_vec())
        };

        let fields = vec![("key", key.as_bytes()), ("data", &data)];

        let _: String = conn.xadd(stream, "*", &fields).await?;

        UPLOAD_SLOTS_TOTAL.with_label_values(&["success"]).inc();

        Ok(())
    }

    pub async fn update_transaction(
        &self,
        ev: TransactionEvent,
        wrap_messages: bool,
        stream: &str,
    ) -> Result<(), RedisError> {
        if stream.is_empty() {
            return Ok(());
        }

        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let (key, data) = if wrap_messages {
            let key = self.create_key_with_prefix(&ev.signature, 84u8);
            let buf = Self::encode_with_wrapper(Transaction(Box::new(ev)));
            (key, buf)
        } else {
            (hex::encode(&ev.signature), ev.encode_to_vec())
        };

        let fields = vec![("key", key.as_bytes()), ("data", &data)];

        let _: String = conn.xadd(stream, "*", &fields).await?;

        UPLOAD_TRANSACTIONS_TOTAL
            .with_label_values(&["success"])
            .inc();

        Ok(())
    }

    fn encode_with_wrapper(message: EventMessage) -> Vec<u8> {
        MessageWrapper {
            event_message: Some(message),
        }
        .encode_to_vec()
    }

    fn create_key_with_prefix(&self, data: &[u8], prefix: u8) -> String {
        let mut temp_key = Vec::with_capacity(data.len() + 1);
        temp_key.push(prefix);
        temp_key.extend_from_slice(data);
        hex::encode(temp_key)
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {}
}
