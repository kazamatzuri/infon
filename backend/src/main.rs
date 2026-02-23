#![allow(dead_code)]

mod api;
mod auth;
mod db;
mod elo;
mod engine;
mod llms_txt;
mod metrics;
mod queue;
mod rate_limit;
mod replay;
mod tournament;

use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use engine::server::GameServer;
use queue::GameQueue;
use rate_limit::RateLimiter;

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "infon-backend" }))
}

async fn metrics_handler() -> impl IntoResponse {
    let body = metrics::gather_metrics();
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
}

/// Axum middleware that records per-request metrics (count and duration).
async fn metrics_middleware(req: Request<Body>, next: Next) -> Response {
    let method = req.method().to_string();
    let path = metrics::normalize_path(req.uri().path());

    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let elapsed = start.elapsed().as_secs_f64();

    let status = response.status().as_u16().to_string();

    metrics::API_REQUESTS_TOTAL
        .with_label_values(&[&method, &path, &status])
        .inc();
    metrics::API_REQUEST_DURATION_SECONDS
        .with_label_values(&[&path])
        .observe(elapsed);

    response
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    metrics::register_metrics();

    let db_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:infon.db?mode=rwc".to_string());
    let db = db::Database::new(&db_url)
        .await
        .expect("Failed to initialize database");
    let db = Arc::new(db);

    let game_server = Arc::new(GameServer::new());
    let rate_limiter = RateLimiter::new();
    let game_queue = GameQueue::new();

    // Spawn background queue worker to process pending games
    crate::queue::spawn_queue_worker(
        db.clone(),
        game_server.clone(),
        game_queue.clone(),
        std::path::PathBuf::from("../data/maps"),
    );

    // Inject Arc<Database> into request extensions so auth extractors can
    // look up API tokens without needing access to AppState directly.
    let db_for_ext = db.clone();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        // Auth routes (no auth required)
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/profile", put(auth::update_profile))
        .with_state(db.clone())
        .merge(api::router(db, game_server, rate_limiter, game_queue))
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn(metrics_middleware))
        .layer(axum::middleware::from_fn(
            move |mut req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
                let db = db_for_ext.clone();
                async move {
                    req.extensions_mut().insert(db);
                    next.run(req).await
                }
            },
        ));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Infon backend listening on port 3000");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
