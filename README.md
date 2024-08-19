# Velvet
(original repo: https://github.com/raffaeleragni/velvet)

A repackage and republish of a combination of crates to create a specific web stack in a consistent and single point of view.
This is not meant to be a library with any specific purpose, only a short handing of boilerplate for the common setup and structure of this web stack.

For a reference/example of a project using it: https://github.com/raffaeleragni/veltes

## Stack used

The templates and static files will be compiled in the binary and those directories and won't be required at runtime.

Items of the stack:
  - WEB: Axum
  - DB: sqlx(postgres)
  - Templating: Askama (folder templates/)
  - Telemetry: sentry supported

## Base route setup

```rust
use velvet::prelude::*;

fn index() -> &'static str {
    "Hello world"
}

#[tokio::main]
async fn main() {
    App::new()
        .router(Router::new().route("/", get(index)))
        .start()
        .await;
}

```

## Add a database

```rust
use velvet::prelude::*;

fn index(Extension(db): Extension<Pool<Postgres>>) -> &'static str {
    let result = query_as!(String, "select 1").fetch_one(&db).await?;
    result
}

#[tokio::main]
async fn main() {
    let db = database().await;

    App::new()
        .router(Router::new().route("/", get(index))
        .inject(db)
        .start()
        .await;
}
```

## Support for static files

Example:

```rust
use velvet::prelude::*;

#[tokio::main]
async fn main() {
    #[derive(RustEmbed)]
    #[folder = "statics"]
    struct S;

    App::new()
        .statics::<S>()
        .start()
        .await;
}
```

## Where to find...

  - Status (no-op): http GET /status/liveness
  - Metrics: http GET /metrics/prometheus

## ENV vars

  - SERVER_BIND: ip for which to listen on
  - SERVER_PORT: [number] port for which to listen on
  - DATABASE_URL: postgres://user:pass@host:port/database (if database used)
  - DATABASE_MAX_CONNECTIONS: [number] (default 1)
  - STRUCTURED_LOGGING: true|false (default false)
  - SENTRY_URL: url inclusive of key for sending telemetry to sentry

## To setup TLS use env vars:

  - TLS=true (or any string)
  - TLS_PEM_CERT=cert.pem
  - TLS_PEM_KEY=key.pem
