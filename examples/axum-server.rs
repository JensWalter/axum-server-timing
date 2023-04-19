use axum::{response::Html, routing::get, Router};
use std::{net::SocketAddr, time::Duration};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler)).layer(
        axum_server_timing::ServerTimingLayer::new("HelloService").with_description("whatever"),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    // intentional sleep, so the duration does not report 0
    tokio::time::sleep(Duration::from_millis(100)).await;
    Html("<h1>Hello, World!</h1>")
}
