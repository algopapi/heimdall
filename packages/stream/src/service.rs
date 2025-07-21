use crate::proto_stream::{
    heimdall_stream_server::HeimdallStream, AccountUpdate, EventUpdate, PoolUpdate,
    PoolUpdateRequest, SlotUpdate, StreamRequest, TransactionUpdate,
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
    type StreamPoolUpdatesStream = ReceiverStream<Result<PoolUpdate, Status>>;

    // Legacy streams are no longer used, but the types must match the trait
    type StreamAccountsStream = ReceiverStream<Result<AccountUpdate, Status>>;
    type StreamSlotsStream = ReceiverStream<Result<SlotUpdate, Status>>;
    type StreamTransactionsStream = ReceiverStream<Result<TransactionUpdate, Status>>;
    type StreamAllStream = ReceiverStream<Result<EventUpdate, Status>>;

    async fn stream_accounts(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamAccountsStream>, Status> {
        Err(Status::unimplemented("Legacy stream not supported"))
    }

    async fn stream_slots(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamSlotsStream>, Status> {
        Err(Status::unimplemented("Legacy stream not supported"))
    }

    async fn stream_transactions(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamTransactionsStream>, Status> {
        Err(Status::unimplemented("Legacy stream not supported"))
    }

    async fn stream_all(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamAllStream>, Status> {
        Err(Status::unimplemented("Legacy stream not supported"))
    }

    async fn stream_pool_updates(
        &self,
        request: Request<PoolUpdateRequest>,
    ) -> Result<Response<Self::StreamPoolUpdatesStream>, Status> {
        let pool_id = request.into_inner().pool_id;
        if pool_id.is_empty() {
            return Err(Status::invalid_argument("pool_id cannot be empty"));
        }

        let (tx, rx) = mpsc::channel(100);
        let redis_client = self.redis_client.clone();

        tracing::info!(pool_id = %pool_id, "Client subscribed to pool updates");

        tokio::spawn(async move {
            if let Err(e) =
                crate::worker::stream_pool_updates_worker(redis_client, pool_id, tx).await
            {
                tracing::error!("Pool updates stream worker failed: {}", e);
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
