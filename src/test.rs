use crate::{ServerTimingExtension, ServerTimingLayer};
use axum::{
    http::{HeaderMap, HeaderValue},
    routing::get,
    Extension, Router,
};
use std::time::Duration;

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
        assert!((100f32..300f32).contains(&val_num), "{val_num}");
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

#[tokio::test]
async fn extension_works() {
    let name = "service";
    let app = Router::new()
        .route(
            "/",
            get(|x: Extension<ServerTimingExtension>| async move {
                let timing = x.0.clone();

                // lock and unlock in one statement, so the mutex is not kept through the async call
                timing.lock().unwrap().record("step1".to_string(), None);
                tokio::time::sleep(Duration::from_millis(100)).await;

                // second call, and again lock the mutex since it was released before the async call
                timing.lock().unwrap().record("step2".to_string(), None);
                "".to_string()
            }),
        )
        .layer(ServerTimingLayer::new(name));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3004").await.unwrap();
    tokio::spawn(async { axum::serve(listener, app.into_make_service()).await });
    //test request
    let resp = reqwest::get("http://localhost:3004/").await.unwrap();
    let hdr = resp.headers().get("server-timing");
    println!("{hdr:?}");
    assert!(hdr.is_some());
}
