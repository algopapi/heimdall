use crate::proto_events;
use crate::proto_stream::{AccountUpdate, EventUpdate, SlotUpdate, TransactionUpdate};
use anyhow::Result;
use prost::Message;
use redis::{streams::StreamReadOptions, AsyncCommands, RedisResult};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tonic::Status;
use uuid::Uuid;

pub async fn stream_accounts_worker(
    redis_client: redis::Client,
    tx: mpsc::Sender<Result<AccountUpdate, Status>>,
) -> Result<()> {
    let mut connection = redis_client.get_multiplexed_async_connection().await?;
    let consumer_group = "stream-accounts-group";
    let consumer_name = format!("stream-accounts-{}", Uuid::new_v4());

    let _: RedisResult<String> = connection
        .xgroup_create_mkstream("heimdall:accounts", consumer_group, "0")
        .await;

    loop {
        let opts = StreamReadOptions::default()
            .group(consumer_group, &consumer_name)
            .count(10)
            .block(1000);

        let results: RedisResult<redis::streams::StreamReadReply> = connection
            .xread_options(&["heimdall:accounts"], &[">"], &opts)
            .await;

        match results {
            Ok(stream_reply) => {
                for stream_key in stream_reply.keys {
                    for stream_id in stream_key.ids {
                        let mut fields = HashMap::new();
                        for (key, value) in stream_id.map.iter() {
                            fields.insert(key.clone(), value.clone());
                        }

                        if let Some(redis::Value::Data(data)) = fields.get("data") {
                            if let Ok(proto_event) =
                                proto_events::UpdateAccountEvent::decode(&data[..])
                            {
                                let account_update = AccountUpdate {
                                    slot: proto_event.slot,
                                    pubkey: proto_event.pubkey,
                                    lamports: proto_event.lamports,
                                    owner: proto_event.owner,
                                    executable: proto_event.executable,
                                    rent_epoch: proto_event.rent_epoch,
                                    data: proto_event.data,
                                    write_version: proto_event.write_version,
                                    txn_signature: proto_event.txn_signature,
                                };

                                if tx.send(Ok(account_update)).await.is_err() {
                                    break;
                                }

                                let _: RedisResult<i32> = connection
                                    .xack("heimdall:accounts", consumer_group, &[&stream_id.id])
                                    .await;
                            }
                        }
                    }
                }
            }
            Err(_) => {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

pub async fn stream_slots_worker(
    redis_client: redis::Client,
    tx: mpsc::Sender<Result<SlotUpdate, Status>>,
) -> Result<()> {
    let mut connection = redis_client.get_multiplexed_async_connection().await?;
    let consumer_group = "stream-slots-group";
    let consumer_name = format!("stream-slots-{}", Uuid::new_v4());

    let _: RedisResult<String> = connection
        .xgroup_create_mkstream("heimdall:slots", consumer_group, "0")
        .await;

    loop {
        let opts = StreamReadOptions::default()
            .group(consumer_group, &consumer_name)
            .count(10)
            .block(1000);

        let results: RedisResult<redis::streams::StreamReadReply> = connection
            .xread_options(&["heimdall:slots"], &[">"], &opts)
            .await;

        match results {
            Ok(stream_reply) => {
                for stream_key in stream_reply.keys {
                    for stream_id in stream_key.ids {
                        let mut fields = HashMap::new();
                        for (key, value) in stream_id.map.iter() {
                            fields.insert(key.clone(), value.clone());
                        }

                        if let Some(redis::Value::Data(data)) = fields.get("data") {
                            if let Ok(proto_event) =
                                proto_events::SlotStatusEvent::decode(&data[..])
                            {
                                let slot_update = SlotUpdate {
                                    slot: proto_event.slot,
                                    parent: proto_event.parent,
                                    status: proto_event.status as u32,
                                };

                                if tx.send(Ok(slot_update)).await.is_err() {
                                    break;
                                }

                                let _: RedisResult<i32> = connection
                                    .xack("heimdall:slots", consumer_group, &[&stream_id.id])
                                    .await;
                            }
                        }
                    }
                }
            }
            Err(_) => {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

pub async fn stream_transactions_worker(
    redis_client: redis::Client,
    tx: mpsc::Sender<Result<TransactionUpdate, Status>>,
) -> Result<()> {
    let mut connection = redis_client.get_multiplexed_async_connection().await?;
    let consumer_group = "stream-transactions-group";
    let consumer_name = format!("stream-transactions-{}", Uuid::new_v4());

    let _: RedisResult<String> = connection
        .xgroup_create_mkstream("heimdall:transactions", consumer_group, "0")
        .await;

    loop {
        let opts = StreamReadOptions::default()
            .group(consumer_group, &consumer_name)
            .count(10)
            .block(1000);

        let results: RedisResult<redis::streams::StreamReadReply> = connection
            .xread_options(&["heimdall:transactions"], &[">"], &opts)
            .await;

        match results {
            Ok(stream_reply) => {
                for stream_key in stream_reply.keys {
                    for stream_id in stream_key.ids {
                        let mut fields = HashMap::new();
                        for (key, value) in stream_id.map.iter() {
                            fields.insert(key.clone(), value.clone());
                        }

                        if let Some(redis::Value::Data(data)) = fields.get("data") {
                            if let Ok(proto_event) =
                                proto_events::TransactionEvent::decode(&data[..])
                            {
                                let transaction_update = TransactionUpdate {
                                    signature: proto_event.signature,
                                    is_vote: proto_event.is_vote,
                                    slot: proto_event.slot,
                                    index: proto_event.index,
                                };

                                if tx.send(Ok(transaction_update)).await.is_err() {
                                    break;
                                }

                                let _: RedisResult<i32> = connection
                                    .xack("heimdall:transactions", consumer_group, &[&stream_id.id])
                                    .await;
                            }
                        }
                    }
                }
            }
            Err(_) => {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

pub async fn stream_all_worker(
    redis_client: redis::Client,
    tx: mpsc::Sender<Result<EventUpdate, Status>>,
) -> Result<()> {
    let mut connection = redis_client.get_multiplexed_async_connection().await?;
    let consumer_group = "stream-all-group";
    let consumer_name = format!("stream-all-{}", Uuid::new_v4());

    let streams = [
        "heimdall:accounts",
        "heimdall:slots",
        "heimdall:transactions",
    ];

    for stream in &streams {
        let _: RedisResult<String> = connection
            .xgroup_create_mkstream(stream, consumer_group, "0")
            .await;
    }

    loop {
        let opts = StreamReadOptions::default()
            .group(consumer_group, &consumer_name)
            .count(10)
            .block(1000);

        let results: RedisResult<redis::streams::StreamReadReply> = connection
            .xread_options(&streams, &[">", ">", ">"], &opts)
            .await;

        match results {
            Ok(stream_reply) => {
                for stream_key in stream_reply.keys {
                    for stream_id in stream_key.ids {
                        let mut fields = HashMap::new();
                        for (key, value) in stream_id.map.iter() {
                            fields.insert(key.clone(), value.clone());
                        }

                        if let Some(redis::Value::Data(data)) = fields.get("data") {
                            let event_update = match stream_key.key.as_str() {
                                "heimdall:accounts" => {
                                    if let Ok(proto_event) =
                                        proto_events::UpdateAccountEvent::decode(&data[..])
                                    {
                                        Some(EventUpdate {
                                            event: Some(
                                                crate::proto_stream::event_update::Event::Account(
                                                    AccountUpdate {
                                                        slot: proto_event.slot,
                                                        pubkey: proto_event.pubkey,
                                                        lamports: proto_event.lamports,
                                                        owner: proto_event.owner,
                                                        executable: proto_event.executable,
                                                        rent_epoch: proto_event.rent_epoch,
                                                        data: proto_event.data,
                                                        write_version: proto_event.write_version,
                                                        txn_signature: proto_event.txn_signature,
                                                    },
                                                ),
                                            ),
                                        })
                                    } else {
                                        None
                                    }
                                }
                                "heimdall:slots" => {
                                    if let Ok(proto_event) =
                                        proto_events::SlotStatusEvent::decode(&data[..])
                                    {
                                        Some(EventUpdate {
                                            event: Some(
                                                crate::proto_stream::event_update::Event::Slot(
                                                    SlotUpdate {
                                                        slot: proto_event.slot,
                                                        parent: proto_event.parent,
                                                        status: proto_event.status as u32,
                                                    },
                                                ),
                                            ),
                                        })
                                    } else {
                                        None
                                    }
                                }
                                "heimdall:transactions" => {
                                    if let Ok(proto_event) =
                                        proto_events::TransactionEvent::decode(&data[..])
                                    {
                                        Some(EventUpdate {
                                            event: Some(crate::proto_stream::event_update::Event::Transaction(TransactionUpdate {
                                                signature: proto_event.signature,
                                                is_vote: proto_event.is_vote,
                                                slot: proto_event.slot,
                                                index: proto_event.index,
                                            })),
                                        })
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };

                            if let Some(event) = event_update {
                                if tx.send(Ok(event)).await.is_err() {
                                    break;
                                }
                            }

                            let _: RedisResult<i32> = connection
                                .xack(&stream_key.key, consumer_group, &[&stream_id.id])
                                .await;
                        }
                    }
                }
            }
            Err(_) => {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
