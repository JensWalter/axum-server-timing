# axum-server-timing
[![Latest Version](https://img.shields.io/crates/v/axum-server-timing.svg)](https://crates.io/crates/axum-server-timing)

An axum layer to inject the server-timing HTTP header into the response.

For refence on the header please see [developer.mozilla.org](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Server-Timing).

## Examples

Using the layer to inject the Http-Timing Header.
```rust
    let app = Router::new()
        .route("/", get(handler))
        .layer(axum_server_timing::ServerTimingLayer::new("HelloService"));
```

### Output
```
HTTP/1.1 200 OK
content-type: text/html; charset=utf-8
content-length: 22
server-timing: HelloService;dur=102
date: Wed, 19 Apr 2023 15:25:40 GMT

<h1>Hello, World!</h1>
```

Using the layer to inject the Http-Timing Header with description.
```rust
    let app = Router::new()
        .route("/", get(handler))
        .layer(
        axum_server_timing::ServerTimingLayer::new("HelloService")
            .with_description("whatever")
        );
```

### Output
```
HTTP/1.1 200 OK
content-type: text/html; charset=utf-8
content-length: 22
server-timing: HelloService;desc="whatever";dur=102
date: Wed, 19 Apr 2023 15:25:40 GMT

<h1>Hello, World!</h1>
```