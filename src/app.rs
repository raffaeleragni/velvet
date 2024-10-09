use askama_axum::IntoResponse;
use axum::{
    routing::{get, MethodRouter},
    Extension, Router,
};
use axum_prometheus::PrometheusMetricLayer;
use axum_server::tls_rustls::RustlsConfig;
use axum_test::{transport_layer::IntoTransportLayer, TestServer};
use rust_embed::RustEmbed;
use sentry_tower::{NewSentryLayer, SentryHttpLayer};
use std::{env, net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tracing::info;
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{
        self,
        format::{Format, JsonFields},
    },
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::errors::AppResult;

#[cfg(feature = "sqlite")]
type DB = sqlx::Pool<sqlx::Sqlite>;

#[cfg(feature = "mysql")]
type DB = sqlx::Pool<sqlx::Mysql>;

#[cfg(feature = "postgres")]
type DB = sqlx::Pool<sqlx::Postgres>;

#[derive(Default)]
pub struct App {
    router: Router,
}

pub struct BuiltApp {
    app: App,
    addr: SocketAddr,
    tls: Option<RustlsConfig>,
}

impl App {
    pub fn new() -> Self {
        // May not know if app is constructed before database, so trigger dotenvs in both situations
        dotenvy::dotenv().ok();
        logger();
        App::default()
    }

    pub async fn start(self) -> AppResult<()> {
        self.build().await?.start().await
    }

    pub fn router(self, router: Router) -> Self {
        Self {
            router: self.router.merge(router),
        }
    }

    pub fn inject<T: Clone + Send + Sync + 'static>(self, t: T) -> Self {
        Self {
            router: self.router.layer(Extension(t)),
        }
    }

    pub fn statics<T: RustEmbed>(self) -> Self {
        let mut app = self;
        for file in T::iter() {
            let file = file.as_ref();
            let bytes = T::get(file).unwrap().data.to_vec();
            let mime = mime_guess::from_path(file).first_raw().unwrap_or("");
            app = Self {
                router: app.router.route(
                    format!("/{}", file).as_str(),
                    get(|| async { ([("Content-Type", mime.to_owned())], bytes).into_response() }),
                ),
            };
        }
        app
    }

    pub fn route(self, path: &str, method_router: MethodRouter<()>) -> Self {
        let mut app = self;
        app.router = app.router.route(path, method_router);
        app
    }

    pub async fn as_test_server(self) -> TestServer {
        TestServer::new(self.build().await.unwrap()).unwrap()
    }

    #[cfg(feature = "login")]
    pub async fn login_flow(self, db: &DB) -> Self {
        crate::auth::login::default_flow::add_default_flow(db, self).await
    }

    async fn build(self) -> AppResult<BuiltApp> {
        let _guard = sentry();
        let compression_layer: CompressionLayer = CompressionLayer::new()
            .br(true)
            .deflate(true)
            .gzip(true)
            .zstd(true);
        let mut app = self;
        app.router = app
            .router
            .route("/status/liveness", get(|| async { "".into_response() }));
        app.router = prometheus(app.router);
        app.router = app
            .router
            .layer(NewSentryLayer::new_from_top())
            .layer(SentryHttpLayer::with_transaction())
            .layer(compression_layer);

        let bind = env::var("SERVER_BIND").unwrap_or("0.0.0.0".into());
        let port = env::var("SERVER_PORT")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(8080);
        let addr = SocketAddr::from_str(format!("{bind}:{port}").as_str()).unwrap();
        if env::var("TLS").is_ok() {
            let pem_cert = env::var("TLS_PEM_CERT")?;
            let pem_key = env::var("TLS_PEM_KEY")?;
            let tls_config = RustlsConfig::from_pem_file(pem_cert, pem_key)
                .await
                .unwrap();
            info!("Starting server on {bind}:{port} with TLS ON");
            Ok(BuiltApp {
                app,
                addr,
                tls: Some(tls_config),
            })
        } else {
            info!("Starting server on {bind}:{port}");
            Ok(BuiltApp {
                app,
                addr,
                tls: None,
            })
        }
    }
}

impl BuiltApp {
    pub async fn start(self) -> AppResult<()> {
        match self.tls {
            Some(tls_config) => {
                axum_server::bind_rustls(self.addr, tls_config)
                    .serve(self.app.router.into_make_service())
                    .await?
            }
            None => axum::serve(TcpListener::bind(self.addr).await?, self.app.router).await?,
        }
        Ok(())
    }
}

impl IntoTransportLayer for App {
    fn into_http_transport_layer(
        self,
        builder: axum_test::transport_layer::TransportLayerBuilder,
    ) -> anyhow::Result<Box<dyn axum_test::transport_layer::TransportLayer>> {
        self.router.into_http_transport_layer(builder)
    }

    fn into_mock_transport_layer(
        self,
    ) -> anyhow::Result<Box<dyn axum_test::transport_layer::TransportLayer>> {
        self.router.into_mock_transport_layer()
    }
}

impl IntoTransportLayer for BuiltApp {
    fn into_http_transport_layer(
        self,
        builder: axum_test::transport_layer::TransportLayerBuilder,
    ) -> anyhow::Result<Box<dyn axum_test::transport_layer::TransportLayer>> {
        self.app.router.into_http_transport_layer(builder)
    }

    fn into_mock_transport_layer(
        self,
    ) -> anyhow::Result<Box<dyn axum_test::transport_layer::TransportLayer>> {
        self.app.router.into_mock_transport_layer()
    }
}

fn sentry() -> Option<sentry::ClientInitGuard> {
    if let Ok(url) = env::var("SENTRY_URL") {
        return Some(sentry::init((
            url,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                traces_sample_rate: 1.0,
                ..Default::default()
            },
        )));
    }
    None
}

pub(crate) fn logger() {
    let enabled: bool = env::var("STRUCTURED_LOGGING")
        .map(|s| s.parse::<bool>().unwrap_or(false))
        .unwrap_or(false);
    if enabled {
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .event_format(Format::default().json())
                    .fmt_fields(JsonFields::new()),
            )
            .with(EnvFilter::from_default_env())
            .try_init()
            .ok();
    } else {
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .try_init()
            .ok();
    };
}

fn prometheus(app: Router) -> Router {
    let (metric_gatherer, metric_printer) = PrometheusMetricLayer::pair();
    app.route(
        "/metrics/prometheus",
        get(|| async move { metric_printer.render() }),
    )
    .layer(metric_gatherer)
}
