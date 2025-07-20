use crate::filters::FilterSet;
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use reqwest;
use serde_json::Value;
use std::time::Duration;
use tokio::sync::watch;
use tokio_tungstenite::connect_async;
use tracing::info;
use tungstenite::Message;

/// Fetches the initial filter set from Supabase via a direct HTTP request.
pub async fn get_initial_filters() -> Result<FilterSet> {
    let supabase_url = std::env::var("SUPABASE_URL")?;
    let anon_key = std::env::var("SUPABASE_ANON_KEY")?;
    let rest_url = format!(
        "{}/rest/v1/laserstream_filters?id=eq.1&select=*",
        supabase_url
    );

    let client = reqwest::Client::new();
    let res = client
        .get(&rest_url)
        .header("apikey", anon_key)
        .header("Authorization", format!("Bearer {}", std::env::var("SUPABASE_ANON_KEY")?))
        .header("Cache-Control", "no-cache")
        .send()
        .await?
        .json::<Vec<FilterSet>>()
        .await?;

    Ok(res.into_iter().next().unwrap_or_default())
}

/// Connects to Supabase Realtime over WebSocket and sends updated FilterSet whenever the
/// underlying database row changes.
pub async fn supabase_listener_task(tx: watch::Sender<FilterSet>) -> Result<()> {
    //--------------------------------------------------
    // WebSocket connect
    //--------------------------------------------------
    let anon_key = std::env::var("SUPABASE_ANON_KEY")?;
    let supabase_url = std::env::var("SUPABASE_URL")?;
    let realtime_url = supabase_url
        .replace("https://", "wss://")
        + "/realtime/v1/websocket?apikey="
        + &anon_key;
    info!(?realtime_url, "Connecting to Supabase Realtime");
    let (ws_stream, _) = connect_async(realtime_url).await.context("WebSocket connect")?;
    let (mut write, read) = ws_stream.split();

    //--------------------------------------------------
    // Subscribe to changes
    //--------------------------------------------------
    let join_msg = serde_json::json!({
        "topic": "realtime:public:laserstream_filters:id=eq.1",
        "event": "phx_join",
        "payload": {},
        "ref": "1"
    });
    info!("Joining Supabase Realtime channel");
    write
        .send(Message::Text(join_msg.to_string()))
        .await
        .context("WebSocket send")?;

    //--------------------------------------------------
    // Heartbeat loop
    //--------------------------------------------------
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(15)).await;
            let heartbeat = serde_json::json!({
                "topic": "phoenix",
                "event": "heartbeat",
                "payload": {},
                "ref": "2"
            });
            if write
                .send(Message::Text(heartbeat.to_string()))
                .await
                .is_err()
            {
                // Stream is dead
                break;
            }
        }
    });

    //--------------------------------------------------
    // Read loop
    //--------------------------------------------------
    info!("Starting Supabase Realtime read loop");
    read
        .try_for_each(|msg| async {
            if let Message::Text(txt) = msg {
                info!(?txt, "Received raw message from Supabase");
                if let Ok(json) = serde_json::from_str::<Value>(&txt) {
                    if json["event"] == "postgres_changes" {
                        if let Some(payload) = json.get("payload") {
                            if let Some(new_row) = payload.get("new") {
                                if let Ok(fs) = serde_json::from_value::<FilterSet>(new_row.clone()) {
                                    info!(?fs, "Successfully parsed new FilterSet");
                                    let _ = tx.send(fs);
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        })
        .await
        .context("WebSocket read")?;

    info!("Supabase listener terminated");
    Ok(())
} 