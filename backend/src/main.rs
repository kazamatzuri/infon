mod api;
mod auth;
mod db;
mod elo;
mod engine;

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use engine::server::GameServer;

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "infon-backend" }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:infon.db?mode=rwc".to_string());
    let db = db::Database::new(&db_url)
        .await
        .expect("Failed to initialize database");
    let db = Arc::new(db);

    let game_server = Arc::new(GameServer::new());

    let app = Router::new()
        .route("/health", get(health_check))
        // Auth routes (no auth required)
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .with_state(db.clone())
        .merge(api::router(db, game_server))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Infon backend listening on port 3000");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
