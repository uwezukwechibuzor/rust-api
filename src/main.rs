mod db;
mod errors;
mod handlers;
mod models;

use axum::{
    routing::get,
    Router,
};
use handlers::{
    AppState,
    create_user, delete_user, get_user, health_check, list_users, update_user,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load .env
    dotenvy::dotenv().ok();

    // Init tracing (structured logs)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Connect to Postgres
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    tracing::info!("Connected to PostgreSQL");

    let state = AppState { db: pool };

    // Build the router
    let app = Router::new()
        .route("/health",            get(health_check))
        .route("/users",             get(list_users).post(create_user))
        .route("/users/:id",         get(get_user).put(update_user).delete(delete_user))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}