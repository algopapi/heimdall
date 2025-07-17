use anyhow::Result;
use tracing::info;

mod database;
mod subscriber;
use subscriber::RedisStreamSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Heimdall DB Processor");
    
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    let client = redis::Client::open(redis_url)?;

    let mut subscriber = RedisStreamSubscriber::new(client).await?;

    subscriber.run().await?;
    Ok(())
}
