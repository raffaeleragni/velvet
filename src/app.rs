use askama_axum::IntoResponse;
use axum::{
    routing::{get, MethodRouter},
    Extension, Router,
};
use axum_prometheus::{metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayer};
use axum_server::tls_rustls::RustlsConfig;
use axum_test::{transport_layer::IntoTransportLayer, TestServer};
use rust_embed::RustEmbed;
use sentry_tower::{NewSentryLayer, SentryHttpLayer};
use std::{env, net::SocketAddr, str::FromStr, sync::LazyLock};
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

#[cfg(feature = "login")]
#[cfg(feature = "sqlite")]
type DB = sqlx::Pool<sqlx::Sqlite>;

#[cfg(feature = "login")]
#[cfg(feature = "mysql")]
type DB = sqlx::Pool<sqlx::Mysql>;

#[cfg(feature = "login")]
#[cfg(feature = "postgres")]
type DB = sqlx::Pool<sqlx::Postgres>;

/// An application.
/// This is handling the main application setup and execution entry point.
///
/// Quickstart with:
/// ```rust
/// use velvet_web::prelude::*;
///
/// #[tokio::main]
/// async fn main() {
///     App::new().start().await;
/// }
/// ```
#[derive(Default)]
pub struct App {
    router: Router,
}

impl App {
    /// Creates a new application.
    /// Takes care of:
    ///   - initializing/reading .env file
    ///   - initializing the logger
    ///
    /// Structured logging (json) will be enabled if in .env: STRUCTURED_LOGGING=true
    pub fn new() -> Self {
        // May not know if app is constructed before database, so trigger dotenvs in both situations
        dotenvy::dotenv().ok();
        logger();
        App::default()
    }

    /// Starts the server.
    ///
    /// Initializes the listening port, TLS(optional), prometheus endpoint, sentry(optional).
    ///
    /// Listening details can be changed in .env with:
    ///   - SERVER_BIND: listening address
    ///   - SERVER_PORT: listening port
    ///
    /// TLS can be setup by pointing these two .env vars to the respective .pem files:
    ///   - TLS=true
    ///   - TLS_PEM_CERT=cert.pem
    ///   - TLS_PEM_KEY=key.pem
    ///
    /// To use sentry, setup the .env var SENTRY_URL.
    pub async fn start(self) -> AppResult<()> {
        self.build().await?.start().await
    }

    /// Append the set of routes to the current application routes.
    pub fn router(self, router: Router) -> Self {
        Self {
            router: self.router.merge(router),
        }
    }

    /// Injects a new extension into the application.
    /// This instance will be available (via clone) when using the Extension<T> extractor for this
    /// type T.
    pub fn inject<T: Clone + Send + Sync + 'static>(self, t: T) -> Self {
        Self {
            router: self.router.layer(Extension(t)),
        }
    }

    /// Serve static files by path from root, from a RustEmbed setup.
    /// RustEmbed will build the contents of the files directly in the binary of the application,
    /// without requiring them to be deployed along.
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

    /// Append a new single route to the application
    pub fn route(self, path: &str, method_router: MethodRouter<()>) -> Self {
        let mut app = self;
        app.router = app.router.route(path, method_router);
        app
    }

    /// Returns the application as a test harness.
    pub async fn as_test_server(self) -> TestServer {
        TestServer::new(self.build().await.unwrap()).unwrap()
    }

    #[cfg(feature = "login")]
    /// Setup the login flow
    /// Registration is handled without email confirmation.
    /// Required for setup .env:
    ///  - JWT_SECRET=<secret>
    pub async fn login_flow(self, db: &DB) -> Self {
        use crate::auth::login::default_flow::LoginConfig;
        crate::auth::login::default_flow::add_default_flow(db, LoginConfig::default(), self).await
    }

    #[cfg(feature = "login")]
    /// Setup the login flow with registration requiring mail confirmation.
    /// Required for setup is the mail environment variables in .env, for example:
    ///  - JWT_SECRET=<secret>
    ///  - MAIL_FROM=test@test.com
    ///  - MAIL_HOST=localhost
    ///  - MAIL_PORT=2525
    ///  - MAIL_USERNAME=user
    ///  - MAIL_PASSWORD=password
    ///  - MAIL_ACCEPT_INVALID_CERTS=true
    pub async fn login_flow_with_mail(self, db: &DB) -> Self {
        use crate::auth::login::default_flow::LoginConfig;
        crate::auth::login::default_flow::add_mail_flow(db, LoginConfig::default(), self).await
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

/// An instance of an application ready to run.
/// Cannot be changed once built, only ran.
pub struct BuiltApp {
    app: App,
    addr: SocketAddr,
    tls: Option<RustlsConfig>,
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

#[cfg(feature = "login")]
#[allow(async_fn_in_trait)]
/// Utility for testing the application with a cookie token
pub trait TestLoginAsCookie {
    async fn login_as(self, username: &str) -> Self;
}

#[cfg(feature = "login")]
impl TestLoginAsCookie for TestServer {
    async fn login_as(self, username: &str) -> Self {
        use crate::auth::jwt::token_from_claims;
        use crate::prelude::JWT;
        use axum_extra::extract::cookie::Cookie;
        use serde::Serialize;
        use std::time::{SystemTime, UNIX_EPOCH};

        #[derive(Serialize)]
        struct Claims {
            exp: u64,
            username: String,
        }
        JWT::Secret.setup().await.unwrap();
        let token = token_from_claims(&Claims {
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 3600 * 24,
            username: username.to_string(),
        })
        .unwrap();
        let mut server = self;
        server.add_cookie(Cookie::new("token", token));
        server
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

/// Setup the logger, this is already called internally on App::new().
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

// axum-prometheus can be initialized only once and would otherwise cause problems for
// simulated envoronments that recreate the app, such as tests, so need to keep a static
static METRICS: LazyLock<(PrometheusMetricLayer, PrometheusHandle)> =
    LazyLock::new(PrometheusMetricLayer::pair);

fn prometheus(app: Router) -> Router {
    app.route(
        "/metrics/prometheus",
        get(|| async move { METRICS.to_owned().1.render() }),
    )
    .layer(METRICS.to_owned().0)
}
