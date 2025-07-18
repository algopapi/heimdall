use std::error::Error;
use tonic::Request;

use crate::proto_stream::heimdall_stream_client::HeimdallStreamClient;

pub mod proto_stream {
    tonic::include_proto!("heimdall.stream");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "http://[::1]:50051";
    let mut client = HeimdallStreamClient::connect(url).await?;

    let request = Request::new(proto_stream::StreamRequest {});
    println!("Starting account stream...");
    let mut account_stream = client.stream_accounts(request).await?.into_inner();

    let mut count = 0;
    while let Some(account_update) = account_stream.message().await? {
        println!(
            "Received account update: slot={}, pubkey={}",
            account_update.slot,
            hex::encode(&account_update.pubkey)
        );

        count += 1;
        if count >= 5 {
            break;
        }
    }

    let request = Request::new(proto_stream::StreamRequest {});
    println!("Starting slot stream...");
    let mut slot_stream = client.stream_slots(request).await?.into_inner();

    let mut count = 0;
    while let Some(slot_update) = slot_stream.message().await? {
        println!(
            "Received slot update: slot={}, parent={}, status={}",
            slot_update.slot, slot_update.parent, slot_update.status
        );

        count += 1;
        if count >= 5 {
            break;
        }
    }

    let request = Request::new(proto_stream::StreamRequest {});
    println!("Starting transaction stream...");
    let mut transaction_stream = client.stream_transactions(request).await?.into_inner();

    let mut count = 0;
    while let Some(transaction_update) = transaction_stream.message().await? {
        println!(
            "Received transaction update: signature={}, is_vote={}, slot={}, index={}",
            hex::encode(&transaction_update.signature),
            transaction_update.is_vote,
            transaction_update.slot,
            transaction_update.index
        );

        count += 1;
        if count >= 5 {
            break;
        }
    }

    Ok(())
}
