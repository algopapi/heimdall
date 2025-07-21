use crate::proto_stream::PoolUpdate;
use anyhow::Result;
use redis::{streams::StreamReadOptions, AsyncCommands, RedisResult};
use serde_json::Value;
use tokio::sync::mpsc;
use tonic::Status;
use uuid::Uuid;

pub async fn stream_pool_updates_worker(
    redis_client: redis::Client,
    pool_id_filter: String,
    tx: mpsc::Sender<Result<PoolUpdate, Status>>,
) -> Result<()> {
    let mut connection = redis_client.get_multiplexed_async_connection().await?;
    let consumer_group = "stream-pool-updates-group";
    // Each client gets its own consumer name to avoid competing for messages
    let consumer_name = format!("stream-pool-updates-{}", Uuid::new_v4());

    let stream_name = "heimdall:pool_events";
    let _: RedisResult<String> = connection
        .xgroup_create_mkstream(stream_name, consumer_group, "0")
        .await;

    loop {
        if tx.is_closed() {
            // Client has disconnected, so we can clean up and exit.
            let _: RedisResult<i32> = connection
                .xgroup_delconsumer(stream_name, consumer_group, &consumer_name)
                .await;
            tracing::info!(consumer=%consumer_name, "Client disconnected, closing worker.");
            break;
        }

        let opts = StreamReadOptions::default()
            .group(consumer_group, &consumer_name)
            .count(10)
            .block(5000); // Block for 5 seconds

        let results: RedisResult<redis::streams::StreamReadReply> = connection
            .xread_options(&[stream_name], &[">"], &opts)
            .await;

        if let Ok(stream_reply) = results {
            for stream_key in stream_reply.keys {
                for stream_id in stream_key.ids {
                    if let Some(redis::Value::Data(data)) = stream_id.map.get("data") {
                        if let Ok(json_val) = serde_json::from_slice::<Value>(data) {
                            if let Some(pool_id) = json_val.get("pool_id").and_then(|v| v.as_str())
                            {
                                // ---- THIS IS THE KEY FILTERING LOGIC ----
                                if pool_id == pool_id_filter {
                                    let event_type = json_val
                                        .get("event_type")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string();

                                    let payload =
                                        json_val.get("payload").cloned().unwrap_or(Value::Null);

                                    let pool_update = PoolUpdate {
                                        pool_id: pool_id.to_string(),
                                        event_type,
                                        payload_json: payload.to_string(),
                                    };

                                    if tx.send(Ok(pool_update)).await.is_err() {
                                        // Break inner loop if send fails
                                        break;
                                    }
                                }
                            }
                        }
                        // Always acknowledge the message to prevent it from being re-processed
                        // by this or other consumers.
                        let _: RedisResult<i32> = connection
                            .xack(&stream_key.key, consumer_group, &[&stream_id.id])
                            .await;
                    }
                }
            }
        }
    }
    Ok(())
}
