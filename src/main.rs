mod app;

// Local modules
use app::config::AppConfig;

// Crates
use airtable_api::{Airtable, Record};
use askama::Template;
use axum::{
    error_handling::HandleErrorLayer,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{str, time::Duration};
use tokio::signal;
use tower::{BoxError, ServiceBuilder};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct HtmlTemplate<T>(T);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: i32,
    pub order_placed: String,
    pub product_name: String,
    pub price: f64,
    pub first_name: String,
    pub last_name: String,
    pub address: String,
    pub email: String,
    pub order_status: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title: String,
    records: Vec<airtable_api::Record<Order>>,
}

#[derive(Debug, Clone)]
struct AppState {
    app_config: AppConfig,
}

// Traits ///////////////////////////////////////////

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

// Route Handlers ////////////////////////////////

async fn handle_root(State(app_state): State<AppState>) -> impl IntoResponse {
    tracing::debug!("{:#?}", app_state.app_config);
    // Initialize the Airtable client.
    let airtable = Airtable::new(
        app_state.app_config.airtable_api_key,
        app_state.app_config.airtable_id,
        "",
    );
    let records: Vec<Record<Order>> = airtable
        .list_records(
            "ORDERS",
            "Grid view",
            vec![
                "order_id",
                "order_placed",
                "product_name",
                "price",
                "first_name",
                "last_name",
                "address",
                "email",
                "order_status",
            ],
        )
        .await
        .unwrap();
    let template = IndexTemplate {
        title: "Vorboss".to_string(),
        records,
    };
    HtmlTemplate(template)
}

// Main /////////////////////////////

const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const REV: &str = env!("REV");
const BRANCH: &str = env!("BRANCH");
const BUILD_USER: &str = env!("BUILD_USER");
const RUST_VERSION: &str = env!("RUST_VERSION");

#[tokio::main]
async fn main() {
    match envy::from_env::<AppConfig>() {
        Ok(app_config) => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::new(&app_config.log_level))
                .with(tracing_subscriber::fmt::layer())
                .init();
            tracing::info!("CARGO_PKG_NAME: {}", CARGO_PKG_NAME);
            tracing::info!("CARGO_PKG_VERSION: {}", CARGO_PKG_VERSION);
            tracing::info!("REV: {}", REV);
            tracing::info!("BRANCH: {}", BRANCH);
            tracing::info!("BUILD_USER: {}", BUILD_USER);
            tracing::info!("RUST_VERSION: {}", RUST_VERSION);
            tracing::debug!("APP_NAME: {:#?}", app_config.app_name);
            tracing::debug!("APP_ENVIRONMENT: {:#?}", app_config.app_environment);
            tracing::debug!("LOG_LEVEL: {:#?}", app_config.log_level);

            // Start the http server
            let app = Router::new()
                .route("/", get(handle_root))
                .layer(
                    ServiceBuilder::new()
                        .layer(HandleErrorLayer::new(|error: BoxError| async move {
                            if error.is::<tower::timeout::error::Elapsed>() {
                                Ok(StatusCode::REQUEST_TIMEOUT)
                            } else {
                                Err((
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    format!("Unhandled internal error: {}", error),
                                ))
                            }
                        }))
                        .timeout(Duration::from_secs(10))
                        .layer(TraceLayer::new_for_http())
                        .layer(CompressionLayer::new())
                        .into_inner(),
                )
                .with_state(AppState {
                    app_config: app_config.clone(),
                });
            let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
            tracing::debug!("listening on {}", addr);
            if app_config.app_environment == "production" {
                axum::Server::bind(&addr)
                    .serve(app.into_make_service())
                    .with_graceful_shutdown(shutdown_signal())
                    .await
                    .unwrap();
            } else {
                axum::Server::bind(&addr)
                    .serve(app.into_make_service())
                    .await
                    .unwrap();
            }
        }
        Err(error) => panic!("{:#?}", error),
    }
}

// Graceful shutdown //////////////////////////////////

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    //#[cfg(not(unix))]
    //let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::debug!("signal received, starting graceful shutdown");
}
