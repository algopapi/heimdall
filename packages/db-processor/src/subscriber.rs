
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use prost::Message;
use redis::{aio::MultiplexedConnection, streams::StreamReadOptions, AsyncCommands, RedisResult};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use uuid;

use crate::database::{AccountUpdate, DatabaseProcessor, SlotUpdate, TransactionUpdate};

include!(concat!(env!("OUT_DIR"), "/heimdall.types.rs"));

pub struct RedisStreamSubscriber {
    connection: MultiplexedConnection,
    db_processor: DatabaseProcessor,
    consumer_group: String,
    consumer_name: String,
}

impl RedisStreamSubscriber {
    pub async fn new(client: redis::Client) -> Result<Self> {
        let connection = client.get_multiplexed_async_connection().await?;
        let db_processor = DatabaseProcessor::new();
        let consumer_group = "db-processor-group".to_string();
        let consumer_name = format!("db-processor-{}", uuid::Uuid::new_v4());
        Ok(Self {
            connection,
            db_processor,
            consumer_group,
            consumer_name,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Starting Redis stream subscriber");
        self.create_consumer_groups().await?;
        loop {
            if let Err(e) = self.consume_streams().await {
                error!("Error consuming streams: {}", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    async fn create_consumer_groups(&mut self) -> Result<()> {
        let streams = vec![
            "heimdall:accounts",
            "heimdall:slots",
            "heimdall:transactions",
        ];
        for stream in streams {
            let _: RedisResult<String> = self
                .connection
                .xgroup_create_mkstream(stream, &self.consumer_group, "0")
                .await;
            info!(
                "Consumer group '{}' ready for stream '{}'",
                self.consumer_group, stream
            );
        }
        Ok(())
    }

    async fn consume_streams(&mut self) -> Result<()> {
        let streams = vec!["heimdall:accounts", "heimdall:slots", "heimdall:transactions"];
        let ids = vec![">", ">", ">"];
        let opts = StreamReadOptions::default()
            .group(&self.consumer_group, &self.consumer_name)
            .count(10)
            .block(1000);
        let results: RedisResult<redis::streams::StreamReadReply> =
            self.connection
                .xread_options(&streams, &ids, &opts)
                .await;
        match results {
            Ok(stream_reply) => {
                for stream_key in stream_reply.keys {
                    for stream_id in stream_key.ids {
                        let mut fields = HashMap::new();
                        for (key, value) in stream_id.map.iter() {
                            fields.insert(key.clone(), value.clone());
                        }
                        if let Err(e) = self
                            .process_message(&stream_key.key, &stream_id.id, fields)
                            .await
                        {
                            error!("Error processing message {}: {}", stream_id.id, e);
                        } else {
                            let _: RedisResult<i32> = self
                                .connection
                                .xack(&stream_key.key, &self.consumer_group, &[&stream_id.id])
                                .await;
                        }
                    }
                }
            }
            Err(e) => {
                warn!("No messages or error reading streams: {}", e);
                sleep(Duration::from_millis(100)).await;
            }
        }
        Ok(())
    }

    async fn process_message(
        &mut self,
        stream_name: &str,
        message_id: &str,
        fields: HashMap<String, redis::Value>,
    ) -> Result<()> {
        info!(
            "Processing message {} from stream {}",
            message_id, stream_name
        );
        let data = match fields.get("data") {
            Some(redis::Value::Data(bytes)) => bytes,
            _ => {
                warn!("No data field found in message {}", message_id);
                return Ok(());
            }
        };
        match stream_name {
            "heimdall:accounts" => self.process_account_update(data).await?,
            "heimdall:slots" => self.process_slot_update(data).await?,
            "heimdall:transactions" => self.process_transaction_update(data).await?,
            _ => warn!("Unknown stream: {}", stream_name),
        }
        Ok(())
    }

    async fn process_account_update(&mut self, data: &[u8]) -> Result<()> {
        match UpdateAccountEvent::decode(data) {
            Ok(proto_event) => {
                let account_update = AccountUpdate {
                    slot: proto_event.slot,
                    pubkey: hex::encode(proto_event.pubkey),
                    lamports: proto_event.lamports,
                    owner: hex::encode(proto_event.owner),
                    executable: proto_event.executable,
                    rent_epoch: proto_event.rent_epoch,
                    data: if proto_event.data.is_empty() {
                        None
                    } else {
                        Some(general_purpose::STANDARD.encode(proto_event.data))
                    },
                    write_version: proto_event.write_version,
                    txn_signature: proto_event.txn_signature.map(|sig| hex::encode(sig)),
                };
                return self.db_processor.store_account_update(account_update).await;
            }
            Err(e) => {
                warn!("Failed to decode account update data as protobuf: {}", e);
                warn!("Raw data: {:?}", String::from_utf8_lossy(data));
            }
        }
        Ok(())
    }

    async fn process_slot_update(&mut self, data: &[u8]) -> Result<()> {
        match SlotStatusEvent::decode(data) {
            Ok(proto_event) => {
                let slot_update = SlotUpdate {
                    slot: proto_event.slot,
                    parent: proto_event.parent,
                    status: proto_event.status as u32,
                };
                return self.db_processor.store_slot_update(slot_update).await;
            }
            Err(e) => {
                warn!("Failed to decode slot update data as protobuf: {}", e);
                warn!("Raw data: {:?}", String::from_utf8_lossy(data));
            }
        }
        Ok(())
    }

    async fn process_transaction_update(&mut self, data: &[u8]) -> Result<()> {
        match TransactionEvent::decode(data) {
            Ok(proto_event) => {
                let transaction_update = TransactionUpdate {
                    signature: hex::encode(proto_event.signature),
                    is_vote: proto_event.is_vote,
                    slot: proto_event.slot,
                    index: proto_event.index,
                };
                return self
                    .db_processor
                    .store_transaction_update(transaction_update)
                    .await;
            }
            Err(e) => {
                warn!("Failed to decode transaction update data as protobuf: {}", e);
                warn!("Raw data: {:?}", String::from_utf8_lossy(data));
            }
        }
        Ok(())
    }
}
