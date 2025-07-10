use dotenv::dotenv;
use std::env;

mod utils;
mod monitors;
mod feeds;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    let endpoint = env::var("GRPC_ENDPOINT")
        .expect("GRPC_ENDPOINT environment variable must be set");
    
    monitors::program_monitor::monitor_program_accounts(endpoint).await
}