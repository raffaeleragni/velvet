use askama_axum::IntoResponse;
use axum::{routing::get, Extension, Router};
use axum_prometheus::PrometheusMetricLayer;
use axum_server::tls_rustls::RustlsConfig;
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

#[derive(Default)]
pub struct App {
    router: Router,
}

impl App {
    pub fn new() -> Self {
        // May not know if app is constructed before databse, so trigger dotenvs in both situations
        dotenvy::dotenv().ok();
        logger();
        Self::default()
    }

    pub async fn start(self) {
        start(self.router).await.unwrap()
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

    pub fn enable_compression(self) -> Self {
        let mut app = self;
        let compression_layer: CompressionLayer = CompressionLayer::new()
            .br(true)
            .deflate(true)
            .gzip(true)
            .zstd(true);
        app.router = app.router.layer(compression_layer);
        app
    }
}

async fn start(app: Router) -> anyhow::Result<()> {
    let bind = env::var("SERVER_BIND").unwrap_or("0.0.0.0".into());
    let port = env::var("SERVER_PORT")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from_str(format!("{bind}:{port}").as_str()).unwrap();
    let _guard = sentry();
    let app = prometheus(app);
    let app = app.route("/status/liveness", get(|| async { "".into_response() }));
    let app = app
        .layer(NewSentryLayer::new_from_top())
        .layer(SentryHttpLayer::with_transaction());

    if env::var("TLS").is_ok() {
        let pem_cert = env::var("TLS_PEM_CERT")?;
        let pem_key = env::var("TLS_PEM_KEY")?;
        info!("Starting server on {bind}:{port} with TLS ON");
        let tls_config = RustlsConfig::from_pem_file(pem_cert, pem_key)
            .await
            .unwrap();
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await?
    } else {
        info!("Starting server on {bind}:{port}");
        axum::serve(TcpListener::bind(addr).await?, app).await?;
    }
    Ok(())
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
