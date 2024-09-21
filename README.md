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

Adding a `.env` file with `DATABASE_URL=sqlite::memory:`.

```rust
use velvet::prelude::*;

#[tokio::main]
async fn main() {
    let db = sqlite().await;
    App::new().route("/", get(index)).inject(db).start().await;
}

async fn index(Extension(db): Extension<Pool<Sqlite>>) -> AppResult<impl IntoResponse> {
    let res = sqlx::query!("pragma integrity_check").fetch_one(&db).await?;
    Ok(res.integrity_check.unwrap_or("Bad check".to_string()))
}
```

## Use an HTTP Client

```rust
use velvet_web::prelude::*;

#[tokio::main]
async fn main() {
    App::new().route("/", get(index)).inject(client()).start().await;
}

async fn index(Extension(client): Extension<Client>) -> AppResult<impl IntoResponse> {
    Ok(client.get("https://en.wikipedia.org").send().await?.text().await?)
}
```

## Check JWT token (from bearer or cookies)

Adding a `.env` file with `JWT_SECRET=secret`.

```rust
use velvet_web::prelude::*;

#[derive(Deserialize)]
struct Claims {
    role: String,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    JWT::Secret.setup().await?;
    let router = Router::new()
        .route("/", get(index))
        .authorized_bearer_claims(|claims: Claims| Ok(claims.role == "admin"));
    App::new().router(router).start().await;
    Ok(())
}

async fn index() -> AppResult<impl IntoResponse> {
    Ok("Hello World")
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

## Add custom metrics

Metrics available at `/metrics/prometheus`. The custom metrics will be visible as soon as the first use happens, but used within the App, not before.

Note: needs to add crate `axum_prometheus`.

```rust
use velvet_web::prelude::*;

#[tokio::main]
async fn main() {
    App::new().route("/", get(index)).start().await;
}

async fn index() -> AppResult<impl IntoResponse> {
    axum_prometheus::metrics::counter!("counter").increment(1);
    Ok("Hello World")
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
