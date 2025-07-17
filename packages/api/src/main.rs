use poem::{
    get, handler, listener::TcpListener, middleware::Tracing, web::Path, EndpointExt, Route, Server,
};

#[handler]
fn hello(Path(name): Path<String>) -> String {
    format!("hello: {name}")
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let api_v1 = Route::new()
        .at("/transactions", get(transactions))
        .at("/slots", get(slots))
        .at("/accounts", get(accounts));

    let app = Route::new()
        .nest("/api/v1", api_v1)
        .with(Tracing);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world")
        .run(app)
        .await
}

#[handler]
fn transactions() -> String {
    "transactions endpoint".to_string()
}

#[handler]
fn slots() -> String {
    "slots endpoint".to_string()
}

#[handler]
fn accounts() -> String {
    "accounts endpoint".to_string()
}