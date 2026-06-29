mod auth;
mod config;
mod error;
mod routes;
mod state;

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{config::Config, state::AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aether_api=debug,tower_http=info,sqlx=warn,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;

    tracing::info!("exécution des migrations…");
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState {
        pool,
        config: config.clone(),
    };

    let app = routes::router(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("aether-api en écoute sur http://{}", config.bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
