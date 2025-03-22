# axum-server-timing
[![Latest Version](https://img.shields.io/crates/v/axum-server-timing.svg)](https://crates.io/crates/axum-server-timing)

An axum layer to inject the Server-Timing HTTP header into the response.

For a reference on the header please see [developer.mozilla.org](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Server-Timing).

## Examples

### Using for Request Timing

Using the layer to inject the Server-Timing Header.
```rust
    let app = Router::new()
        .route("/", get(handler))
        .layer(axum_server_timing::ServerTimingLayer::new("HelloService"));
```

**Output**
```http
HTTP/1.1 200 OK
content-type: text/html; charset=utf-8
content-length: 22
server-timing: HelloService;dur=102
date: Wed, 19 Apr 2023 15:25:40 GMT

<h1>Hello, World!</h1>
```

### Using for Request Timing with Description

Using the layer to inject the Server-Timing Header with description.
```rust
    let app = Router::new()
        .route("/", get(handler))
        .layer(
        axum_server_timing::ServerTimingLayer::new("HelloService")
            .with_description("whatever")
        );
```

**Output**
```http
HTTP/1.1 200 OK
content-type: text/html; charset=utf-8
content-length: 22
server-timing: HelloService;desc="whatever";dur=102
date: Wed, 19 Apr 2023 15:25:40 GMT

<h1>Hello, World!</h1>
```

### Recording exection Steps

```rust
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
```

**Output**
```http
HTTP/1.1 200 OK
content-type: text/html; charset=utf-8
server-timing: HelloService;desc="whatever";dur=102.43, after-sleep;dur=102.40, custom;dur=66.00
content-length: 22
date: Sat, 22 Mar 2025 20:32:45 GMT

<h1>Hello, World!</h1>
```