use axum::{response::Html, routing::get, Extension, Router};
use axum_server_timing::ServerTimingExtension;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler)).layer(
        axum_server_timing::ServerTimingLayer::new("HelloService").with_description("whatever"),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on 0.0.0.0:3000");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn handler(Extension(timing): Extension<ServerTimingExtension>) -> Html<&'static str> {
    // intentional sleep, so the duration does not report 0
    tokio::time::sleep(Duration::from_millis(100)).await;
    timing
        .lock()
        .unwrap()
        .record("after-sleep".to_string(), None);
    timing
        .lock()
        .unwrap()
        .record_timing("custom".to_string(), Duration::from_millis(66), None);
    Html("<h1>Hello, World!</h1>")
}
