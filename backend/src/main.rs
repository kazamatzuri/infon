mod api;
mod db;
mod engine;

use std::sync::Arc;
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "infon-backend" }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db = db::Database::new("sqlite:infon.db?mode=rwc")
        .await
        .expect("Failed to initialize database");
    let db = Arc::new(db);

    let app = Router::new()
        .route("/health", get(health_check))
        .merge(api::router(db))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Infon backend listening on port 3000");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
