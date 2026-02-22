mod api;
mod auth;
mod db;
mod elo;
mod engine;
mod llms_txt;
mod queue;
mod rate_limit;
mod replay;
mod tournament;

use axum::{
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
        // Auth routes (no auth required)
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/profile", put(auth::update_profile))
        .with_state(db.clone())
        .merge(api::router(db, game_server, rate_limiter, game_queue))
        .layer(CorsLayer::permissive())
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
