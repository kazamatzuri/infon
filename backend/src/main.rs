#![allow(dead_code)]

mod api;
mod auth;
mod config;
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

use config::Config;
use engine::server::GameServer;
use queue::GameQueue;
use rate_limit::RateLimiter;

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "infon-backend" }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg = Config::load();

    // Set local mode flag globally so auth extractors can check it
    config::set_local_mode(cfg.local_mode);

    if cfg.local_mode {
        tracing::info!("==========================================================");
        tracing::info!("  Running in LOCAL MODE - no authentication required");
        tracing::info!("  Rate limiting is disabled");
        tracing::info!("==========================================================");
    }

    let db = db::Database::new(&cfg.database_url)
        .await
        .expect("Failed to initialize database");
    let db = Arc::new(db);

    // In local mode, ensure a default user exists for auto-login
    if cfg.local_mode {
        ensure_local_user(&db).await;
    }

    let game_server = Arc::new(GameServer::new());
    let rate_limiter = RateLimiter::new();
    let game_queue = GameQueue::new();

    // Spawn background queue worker to process pending games
    crate::queue::spawn_queue_worker(
        db.clone(),
        game_server.clone(),
        game_queue.clone(),
        cfg.maps_dir.clone(),
    );

    // Inject Arc<Database> into request extensions so auth extractors can
    // look up API tokens without needing access to AppState directly.
    let db_for_ext = db.clone();

    let mut app = Router::new()
        .route("/health", get(health_check))
        // Auth routes (no auth required)
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/local", post(auth::local_login))
        .route("/api/auth/local-mode", get(auth::local_mode_status))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/profile", put(auth::update_profile))
        .with_state(db.clone())
        .merge(api::router(
            db,
            game_server,
            rate_limiter,
            game_queue,
            cfg.maps_dir.clone(),
        ))
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

    // Serve static frontend files if a static directory is configured
    if let Some(ref static_dir) = cfg.static_dir {
        if static_dir.exists() {
            tracing::info!("Serving static files from: {}", static_dir.display());
            // Serve static files, with SPA fallback to index.html for client-side routing
            let serve_dir = tower_http::services::ServeDir::new(static_dir)
                .not_found_service(tower_http::services::ServeFile::new(
                    static_dir.join("index.html"),
                ));
            app = app.fallback_service(serve_dir);
        } else {
            tracing::warn!(
                "Static directory not found: {} - frontend will not be served",
                static_dir.display()
            );
        }
    }

    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", addr));

    tracing::info!("Infon backend listening on port {}", cfg.port);
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

/// Ensure the default "local" user exists in the database for local mode.
async fn ensure_local_user(db: &db::Database) {
    match db
        .get_user_by_username(config::LOCAL_USERNAME)
        .await
    {
        Ok(Some(_)) => {
            tracing::info!("Local user already exists");
        }
        Ok(None) => {
            // Create a local user with a placeholder password hash
            let password_hash = auth::hash_password("local-mode-password")
                .unwrap_or_else(|_| "not-a-real-hash".to_string());
            match db
                .create_user(
                    config::LOCAL_USERNAME,
                    "local@localhost",
                    &password_hash,
                    "Local Player",
                )
                .await
            {
                Ok(user) => {
                    tracing::info!("Created local user (id={})", user.id);
                }
                Err(e) => {
                    tracing::warn!("Failed to create local user: {e}");
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to check for local user: {e}");
        }
    }
}
