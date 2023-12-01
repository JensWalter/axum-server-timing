use axum::{
    http::{HeaderMap, HeaderValue},
    routing::get,
    Router,
};
use std::time::Duration;

use crate::ServerTimingLayer;

#[test]
fn service_name() {
    let name = "svc1";
    let obj = ServerTimingLayer::new(name);
    assert_eq!(obj.app, name);
}

#[test]
fn service_desc() {
    let name = "svc1";
    let desc = "desc1";
    let obj = ServerTimingLayer::new(name).with_description(desc);
    assert_eq!(obj.app, name);
    assert_eq!(obj.description, Some(desc));
}

#[tokio::test]
async fn header_exists_on_response() {
    let name = "svc1";
    let app = Router::new()
        .route("/", get(|| async move { "" }))
        .layer(ServerTimingLayer::new(name));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tokio::spawn(async { axum::serve(listener, app.into_make_service()).await });
    //test request
    let resp = reqwest::get("http://localhost:3001/").await.unwrap();
    let hdr = resp.headers().get("server-timing");
    assert!(hdr.is_some());
}

#[tokio::test]
async fn header_value() {
    let name = "svc1";
    let app = Router::new()
        .route(
            "/",
            get(|| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                ""
            }),
        )
        .layer(ServerTimingLayer::new(name));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
    tokio::spawn(async { axum::serve(listener, app.into_make_service()).await });

    //test request
    let resp = reqwest::get("http://localhost:3002/").await.unwrap();
    if let Some(hdr) = resp.headers().get("server-timing") {
        let val = &hdr.to_str().unwrap()[9..];
        let val_num: f32 = val.parse().unwrap();
        assert!(val_num > 100_f32);
    } else {
        panic!("no header found");
    }
}

#[tokio::test]
async fn support_existing_header() {
    let name = "svc1";
    let app = Router::new()
        .route(
            "/",
            get(|| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let mut hdr = HeaderMap::new();
                hdr.insert("server-timing", HeaderValue::from_static("inner;dur=23"));
                (hdr, "")
            }),
        )
        .layer(ServerTimingLayer::new(name));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003").await.unwrap();
    tokio::spawn(async { axum::serve(listener, app.into_make_service()).await });

    //test request
    let resp = reqwest::get("http://localhost:3003/").await.unwrap();
    let hdr = resp.headers().get("server-timing").unwrap();
    let hdr_str = hdr.to_str().unwrap();
    assert!(hdr_str.contains("svc1"));
    assert!(hdr_str.contains("inner"));
    println!("{hdr:?}");
}
