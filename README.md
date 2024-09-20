# Velvet
(original repo: https://github.com/raffaeleragni/velvet)

![crates.io](https://img.shields.io/crates/v/velvet_web)

A layer of republish and small functions to remove some boilerplate on web stacks.

For a reference/example of a project using it: https://github.com/raffaeleragni/veltes

## Stack used

The askama templates, the static RustEmbed will all be compiled in the binary and not require them at runtime in the file system.

The sqlx migrations are not embedded, and will be needed at file system at runtime.

Items of the stack:
  - WEB: Axum
  - DB: sqlx(postgres,sqlite,mysql)
  - Templating: Askama (folder templates/)
  - Telemetry: sentry supported
  - Metrics: prometheus under /metrics/prometheus

## Base route setup

```rust
use velvet_web::prelude::*;

#[tokio::main]
async fn main() {
    App::new().route("/", get(index)).start().await;
}

async fn index() -> impl IntoResponse {
    "Hello World"
}
```

## Add a database

```rust
use velvet::prelude::*;

#[tokio::main]
async fn main() {
    let db = sqlite().await;
    App::new().route("/", get(index).inject(db).start().await;
}

async fn index(Extension(db): Extension<Pool<Sqlite>>) -> AppResult<impl IntoResponse> {
    let res = sqlx::query!("pragma integrity_check").fetch_one(&db).await?;
    Ok(res.integrity_check.unwrap_or("Bad check".to_string()))
}
```

## Support for static files

```rust
use velvet::prelude::*;

#[tokio::main]
async fn main() {
    #[derive(RustEmbed)]
    #[folder = "statics"]
    struct S;

    App::new().statics::<S>().start().await;
}
```

## Default routes already implemented

  - Status (no-op): http GET /status/liveness
  - Metrics: http GET /metrics/prometheus

## ENV vars

  - SERVER_BIND: [default] default (0.0.0.0) bind network for which to listen on
  - SERVER_PORT: [number] (default 8080) port for which to listen on
  - DATABASE_URL: postgres://user:pass@host:port/database (if database used) or sqlite::memory:...
  - DATABASE_MAX_CONNECTIONS: [number] (default 1)
  - STRUCTURED_LOGGING: true|false (default false)
  - SENTRY_URL: url inclusive of key for sending telemetry to sentry

## To setup TLS use env vars:

  - TLS=true (or any string)
  - TLS_PEM_CERT=cert.pem
  - TLS_PEM_KEY=key.pem
