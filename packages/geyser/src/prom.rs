use {
    bytes::Bytes,
    http::StatusCode,
    http_body_util::Full,
    hyper::{body::Incoming, service::service_fn, Request, Response},
    hyper_util::rt::TokioIo,
    log::*,
    prometheus::{GaugeVec, IntCounterVec, Opts, Registry, TextEncoder},
    std::{io::Result as IoResult, net::SocketAddr, sync::Once, time::Duration},
    tokio::net::TcpListener,
    tokio::runtime::Runtime,
};

lazy_static::lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    pub static ref UPLOAD_ACCOUNTS_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("upload_accounts_total", "Status of uploaded accounts"),
        &["status"]
    ).unwrap();

    pub static ref UPLOAD_SLOTS_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("upload_slots_total", "Status of uploaded slots"),
        &["status"]
    ).unwrap();

    pub static ref UPLOAD_TRANSACTIONS_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("upload_transactions_total", "Status of uploaded transactions"),
        &["status"]
    ).unwrap();

    static ref REDIS_STATS: GaugeVec = GaugeVec::new(
        Opts::new("redis_stats", "Redis connection metrics"),
        &["metric"]
    ).unwrap();
}

#[derive(Debug)]
pub struct PrometheusService {
    runtime: Runtime,
}

impl PrometheusService {
    pub fn new(address: SocketAddr) -> IoResult<Self> {
        static REGISTER: Once = Once::new();
        REGISTER.call_once(|| {
            macro_rules! register {
                ($collector:ident) => {
                    REGISTRY
                        .register(Box::new($collector.clone()))
                        .expect("collector can't be registered");
                };
            }
            register!(UPLOAD_ACCOUNTS_TOTAL);
            register!(UPLOAD_SLOTS_TOTAL);
            register!(UPLOAD_TRANSACTIONS_TOTAL);
            register!(REDIS_STATS);

        });

        let runtime = Runtime::new()?;
        runtime.spawn(async move {
            let listener = TcpListener::bind(address).await.unwrap();

            loop {
                let (stream, _) = match listener.accept().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                        continue;
                    }
                };

                let io = TokioIo::new(stream);

                let service = service_fn(|req: Request<Incoming>| async move {
                    let response = match req.uri().path() {
                        "/metrics" => metrics_handler(),
                        _ => not_found_handler(),
                    };
                    Ok::<_, hyper::Error>(response)
                });

                tokio::task::spawn(async move {
                    if let Err(err) = hyper::server::conn::http1::Builder::new()
                        .serve_connection(io, service)
                        .await
                    {
                        error!("Error serving connection: {}", err);
                    }
                });
            }
        });
        Ok(PrometheusService { runtime })
    }

    pub fn shutdown(self) {
        self.runtime.shutdown_timeout(Duration::from_secs(10));
    }
}

fn metrics_handler() -> Response<Full<Bytes>> {
    let metrics = TextEncoder::new()
        .encode_to_string(&REGISTRY.gather())
        .unwrap_or_else(|error| {
            error!("could not encode custom metrics: {}", error);
            String::new()
        });
    Response::builder()
        .body(Full::new(Bytes::from(metrics)))
        .unwrap()
}

fn not_found_handler() -> Response<Full<Bytes>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from("")))
        .unwrap()
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RedisStatsContext;
