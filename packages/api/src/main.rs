use poem::{get, listener::TcpListener, middleware::Tracing, EndpointExt, Route, Server};

mod routes;
use routes::{get_accounts, get_slots, get_transactions};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let api_v1 = Route::new()
        .at("/transactions", get(get_transactions))
        .at("/slots", get(get_slots))
        .at("/accounts", get(get_accounts));

    let app = Route::new().nest("/api/v1", api_v1).with(Tracing);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("heimdall-api")
        .run(app)
        .await
}
