pub mod proto_stream {
    tonic::include_proto!("heimdall.stream");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("stream_descriptor");
}

pub mod proto_events {
    tonic::include_proto!("heimdall.types");
}

mod service;
mod worker;

use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let addr = "[::1]:50051".parse()?;
    info!("Starting Heimdall Stream Server on {}", addr);

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = redis::Client::open(redis_url)?;

    let stream_service = crate::service::StreamService::new(redis_client);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto_stream::FILE_DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        .accept_http1(true)
        .layer(tower_http::cors::CorsLayer::permissive())
        .add_service(reflection_service)
        .add_service(tonic_web::enable(
            crate::proto_stream::heimdall_stream_server::HeimdallStreamServer::new(stream_service),
        ))
        .serve(addr)
        .await?;

    Ok(())
}
