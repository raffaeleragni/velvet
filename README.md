# Velvet
(original repo: https://github.com/raffaeleragni/velvet)

![crates.io](https://img.shields.io/crates/v/velvet_web)

![build](https://github.com/raffaeleragni/velvet/actions/workflows/build.yml/badge.svg)

A repackage and republish of a combination of crates to create a specific web stack in a consistent and single point of view.
This is not meant to be a library with any specific purpose, only a short handing of boilerplate for the common setup and structure of this web stack.

For a reference/example of a project using it: https://github.com/raffaeleragni/veltes

## Stack used

The templates and static files will be compiled in the binary and those directories and won't be required at runtime.

Items of the stack:
  - WEB: Axum
  - DB: sqlx(postgres,sqlite,mysql)
  - Templating: Askama (folder templates/)
  - Telemetry: sentry supported

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

fn index(Extension(db): Extension<Pool<Sqlite>>) -> impl IntoResponse {
    let result = query_as!(String, "select 1").fetch_one(&db).await?;
    result
}

#[tokio::main]
async fn main() {
    let db = sqlite().await;
    App::new().route("/", get(index).inject(db).start().await;
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
