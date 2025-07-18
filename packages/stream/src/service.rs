use crate::proto_stream::heimdall_stream_server::HeimdallStream;
use crate::proto_stream::{
    AccountUpdate, EventUpdate, SlotUpdate, StreamRequest, TransactionUpdate,
};
use anyhow::Result;
use redis::Client;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct StreamService {
    pub redis_client: Client,
}

impl StreamService {
    pub fn new(redis_client: Client) -> Self {
        Self { redis_client }
    }
}

#[tonic::async_trait]
impl HeimdallStream for StreamService {
    type StreamAccountsStream = ReceiverStream<Result<AccountUpdate, Status>>;
    type StreamSlotsStream = ReceiverStream<Result<SlotUpdate, Status>>;
    type StreamTransactionsStream = ReceiverStream<Result<TransactionUpdate, Status>>;
    type StreamAllStream = ReceiverStream<Result<EventUpdate, Status>>;

    async fn stream_accounts(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamAccountsStream>, Status> {
        let (tx, rx) = mpsc::channel(100);
        let redis_client = self.redis_client.clone();

        tokio::spawn(async move {
            if let Err(e) = crate::worker::stream_accounts_worker(redis_client, tx).await {
                tracing::error!("Account stream worker failed: {}", e);
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn stream_slots(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamSlotsStream>, Status> {
        let (tx, rx) = mpsc::channel(100);
        let redis_client = self.redis_client.clone();

        tokio::spawn(async move {
            if let Err(e) = crate::worker::stream_slots_worker(redis_client, tx).await {
                tracing::error!("Slot stream worker failed: {}", e);
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn stream_transactions(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamTransactionsStream>, Status> {
        let (tx, rx) = mpsc::channel(100);
        let redis_client = self.redis_client.clone();

        tokio::spawn(async move {
            if let Err(e) = crate::worker::stream_transactions_worker(redis_client, tx).await {
                tracing::error!("Transaction stream worker failed: {}", e);
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn stream_all(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamAllStream>, Status> {
        let (tx, rx) = mpsc::channel(100);
        let redis_client = self.redis_client.clone();

        tokio::spawn(async move {
            if let Err(e) = crate::worker::stream_all_worker(redis_client, tx).await {
                tracing::error!("All events stream worker failed: {}", e);
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
