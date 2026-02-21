// HTTP API routes (bot CRUD, game control, etc.)

pub mod ws;

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::db::Database;
use crate::engine::server::{GameServer, PlayerEntry};

// ── Request types ─────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateBotRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateBotVersionRequest {
    pub code: String,
    pub api_type: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTournamentRequest {
    pub name: String,
    pub map: Option<String>,
}

#[derive(Deserialize)]
pub struct AddTournamentEntryRequest {
    pub bot_version_id: i64,
    pub slot_name: Option<String>,
}

#[derive(Deserialize)]
pub struct StartGameRequest {
    pub players: Vec<StartGamePlayer>,
}

#[derive(Deserialize)]
pub struct StartGamePlayer {
    pub bot_version_id: i64,
    pub name: Option<String>,
}

// ── Shared application state ─────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub game_server: Arc<GameServer>,
}

// ── Error helper ──────────────────────────────────────────────────────

fn json_error(status: StatusCode, msg: &str) -> impl IntoResponse {
    (status, Json(json!({ "error": msg })))
}

fn internal_error(e: sqlx::Error) -> impl IntoResponse {
    tracing::error!("Database error: {e}");
    json_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error",
    )
}

// ── Router ────────────────────────────────────────────────────────────

pub fn router(db: Arc<Database>, game_server: Arc<GameServer>) -> Router {
    let state = AppState { db, game_server };

    Router::new()
        // Bots
        .route("/api/bots", get(list_bots).post(create_bot))
        .route(
            "/api/bots/{id}",
            get(get_bot).put(update_bot).delete(delete_bot),
        )
        // Bot versions
        .route(
            "/api/bots/{id}/versions",
            get(list_bot_versions).post(create_bot_version),
        )
        .route(
            "/api/bots/{bot_id}/versions/{version_id}",
            get(get_bot_version),
        )
        // Tournaments
        .route(
            "/api/tournaments",
            get(list_tournaments).post(create_tournament),
        )
        .route("/api/tournaments/{id}", get(get_tournament))
        // Tournament entries
        .route(
            "/api/tournaments/{id}/entries",
            get(list_tournament_entries).post(add_tournament_entry),
        )
        .route(
            "/api/tournaments/{id}/entries/{entry_id}",
            delete(remove_tournament_entry),
        )
        // Tournament results
        .route(
            "/api/tournaments/{id}/results",
            get(get_tournament_results),
        )
        // Tournament run
        .route("/api/tournaments/{id}/run", post(run_tournament))
        // Game control
        .route("/api/game/start", post(start_game))
        .route("/api/game/status", get(game_status))
        .route("/api/game/stop", post(stop_game))
        // WebSocket
        .route("/ws/game", get(ws::ws_game))
        .with_state(state)
}

// ── Bot handlers ──────────────────────────────────────────────────────

async fn list_bots(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.list_bots().await {
        Ok(bots) => (StatusCode::OK, Json(json!(bots))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn create_bot(
    State(state): State<AppState>,
    Json(req): Json<CreateBotRequest>,
) -> impl IntoResponse {
    if req.name.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "name is required").into_response();
    }
    let description = req.description.unwrap_or_default();
    match state.db.create_bot(&req.name, &description).await {
        Ok(bot) => (StatusCode::CREATED, Json(json!(bot))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn get_bot(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.db.get_bot(id).await {
        Ok(Some(bot)) => (StatusCode::OK, Json(json!(bot))).into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, "Bot not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn update_bot(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateBotRequest>,
) -> impl IntoResponse {
    if req.name.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "name is required").into_response();
    }
    let description = req.description.unwrap_or_default();
    match state.db.update_bot(id, &req.name, &description).await {
        Ok(Some(bot)) => (StatusCode::OK, Json(json!(bot))).into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, "Bot not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn delete_bot(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.db.delete_bot(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => json_error(StatusCode::NOT_FOUND, "Bot not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

// ── Bot version handlers ──────────────────────────────────────────────

async fn list_bot_versions(
    State(state): State<AppState>,
    Path(bot_id): Path<i64>,
) -> impl IntoResponse {
    // Check bot exists
    match state.db.get_bot(bot_id).await {
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "Bot not found").into_response(),
        Err(e) => return internal_error(e).into_response(),
        Ok(Some(_)) => {}
    }
    match state.db.list_bot_versions(bot_id).await {
        Ok(versions) => (StatusCode::OK, Json(json!(versions))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn create_bot_version(
    State(state): State<AppState>,
    Path(bot_id): Path<i64>,
    Json(req): Json<CreateBotVersionRequest>,
) -> impl IntoResponse {
    // Check bot exists
    match state.db.get_bot(bot_id).await {
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "Bot not found").into_response(),
        Err(e) => return internal_error(e).into_response(),
        Ok(Some(_)) => {}
    }
    let api_type = req.api_type.unwrap_or_else(|| "oo".to_string());
    if api_type != "oo" && api_type != "state" {
        return json_error(StatusCode::BAD_REQUEST, "api_type must be 'oo' or 'state'")
            .into_response();
    }
    match state.db.create_bot_version(bot_id, &req.code, &api_type).await {
        Ok(version) => (StatusCode::CREATED, Json(json!(version))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn get_bot_version(
    State(state): State<AppState>,
    Path((bot_id, version_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    match state.db.get_bot_version(bot_id, version_id).await {
        Ok(Some(version)) => (StatusCode::OK, Json(json!(version))).into_response(),
        Ok(None) => {
            json_error(StatusCode::NOT_FOUND, "Bot version not found").into_response()
        }
        Err(e) => internal_error(e).into_response(),
    }
}

// ── Tournament handlers ───────────────────────────────────────────────

async fn list_tournaments(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.list_tournaments().await {
        Ok(tournaments) => (StatusCode::OK, Json(json!(tournaments))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn create_tournament(
    State(state): State<AppState>,
    Json(req): Json<CreateTournamentRequest>,
) -> impl IntoResponse {
    if req.name.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "name is required").into_response();
    }
    let map = req.map.unwrap_or_else(|| "default".to_string());
    match state.db.create_tournament(&req.name, &map).await {
        Ok(tournament) => (StatusCode::CREATED, Json(json!(tournament))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn get_tournament(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.db.get_tournament(id).await {
        Ok(Some(tournament)) => (StatusCode::OK, Json(json!(tournament))).into_response(),
        Ok(None) => {
            json_error(StatusCode::NOT_FOUND, "Tournament not found").into_response()
        }
        Err(e) => internal_error(e).into_response(),
    }
}

// ── Tournament entry handlers ─────────────────────────────────────────

async fn list_tournament_entries(
    State(state): State<AppState>,
    Path(tournament_id): Path<i64>,
) -> impl IntoResponse {
    match state.db.get_tournament(tournament_id).await {
        Ok(None) => {
            return json_error(StatusCode::NOT_FOUND, "Tournament not found").into_response()
        }
        Err(e) => return internal_error(e).into_response(),
        Ok(Some(_)) => {}
    }
    match state.db.list_tournament_entries(tournament_id).await {
        Ok(entries) => (StatusCode::OK, Json(json!(entries))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn add_tournament_entry(
    State(state): State<AppState>,
    Path(tournament_id): Path<i64>,
    Json(req): Json<AddTournamentEntryRequest>,
) -> impl IntoResponse {
    match state.db.get_tournament(tournament_id).await {
        Ok(None) => {
            return json_error(StatusCode::NOT_FOUND, "Tournament not found").into_response()
        }
        Err(e) => return internal_error(e).into_response(),
        Ok(Some(_)) => {}
    }
    let slot_name = req.slot_name.unwrap_or_default();
    match state.db
        .add_tournament_entry(tournament_id, req.bot_version_id, &slot_name)
        .await
    {
        Ok(entry) => (StatusCode::CREATED, Json(json!(entry))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn remove_tournament_entry(
    State(state): State<AppState>,
    Path((_tournament_id, entry_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    match state.db.remove_tournament_entry(entry_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => {
            json_error(StatusCode::NOT_FOUND, "Tournament entry not found").into_response()
        }
        Err(e) => internal_error(e).into_response(),
    }
}

// ── Tournament result handlers ────────────────────────────────────────

async fn get_tournament_results(
    State(state): State<AppState>,
    Path(tournament_id): Path<i64>,
) -> impl IntoResponse {
    match state.db.get_tournament(tournament_id).await {
        Ok(None) => {
            return json_error(StatusCode::NOT_FOUND, "Tournament not found").into_response()
        }
        Err(e) => return internal_error(e).into_response(),
        Ok(Some(_)) => {}
    }
    match state.db.get_tournament_results(tournament_id).await {
        Ok(results) => (StatusCode::OK, Json(json!(results))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

// ── Tournament run handler ───────────────────────────────────────────

async fn run_tournament(
    State(state): State<AppState>,
    Path(tournament_id): Path<i64>,
) -> impl IntoResponse {
    // Check tournament exists
    let tournament = match state.db.get_tournament(tournament_id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return json_error(StatusCode::NOT_FOUND, "Tournament not found").into_response()
        }
        Err(e) => return internal_error(e).into_response(),
    };

    if state.game_server.is_running() {
        return json_error(StatusCode::CONFLICT, "A game is already running").into_response();
    }

    // Load tournament entries
    let entries = match state.db.list_tournament_entries(tournament_id).await {
        Ok(e) => e,
        Err(e) => return internal_error(e).into_response(),
    };

    if entries.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "Tournament has no entries").into_response();
    }

    // Load bot code for each entry
    let mut players = Vec::new();
    for entry in &entries {
        let version = match state
            .db
            .get_bot_version_by_id(entry.bot_version_id)
            .await
        {
            Ok(Some(v)) => v,
            Ok(None) => {
                return json_error(
                    StatusCode::BAD_REQUEST,
                    &format!(
                        "Bot version {} not found for entry {}",
                        entry.bot_version_id, entry.id
                    ),
                )
                .into_response();
            }
            Err(e) => return internal_error(e).into_response(),
        };

        let name = if entry.slot_name.is_empty() {
            format!("Player {}", entry.id)
        } else {
            entry.slot_name.clone()
        };

        players.push(PlayerEntry {
            name,
            code: version.code,
            api_type: version.api_type,
        });
    }

    // Create world (use default for MVP)
    let world = GameServer::default_world();

    // Update tournament status
    let _ = state
        .db
        .update_tournament_status(tournament_id, "running")
        .await;

    // Start the game
    match state.game_server.start_game(world, players, None) {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "status": "running",
                "tournament_id": tournament_id,
                "tournament_name": tournament.name,
                "message": "Game started. Connect to /ws/game for live updates."
            })),
        )
            .into_response(),
        Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, &e).into_response(),
    }
}

// ── Game control handlers ────────────────────────────────────────────

async fn start_game(
    State(state): State<AppState>,
    Json(req): Json<StartGameRequest>,
) -> impl IntoResponse {
    if state.game_server.is_running() {
        return json_error(StatusCode::CONFLICT, "A game is already running").into_response();
    }

    if req.players.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "At least one player is required")
            .into_response();
    }

    let mut players = Vec::new();
    for (i, p) in req.players.iter().enumerate() {
        let version = match state.db.get_bot_version_by_id(p.bot_version_id).await {
            Ok(Some(v)) => v,
            Ok(None) => {
                return json_error(
                    StatusCode::NOT_FOUND,
                    &format!("Bot version {} not found", p.bot_version_id),
                )
                .into_response();
            }
            Err(e) => return internal_error(e).into_response(),
        };

        let name = p
            .name
            .clone()
            .unwrap_or_else(|| format!("Player {}", i + 1));

        players.push(PlayerEntry {
            name,
            code: version.code,
            api_type: version.api_type,
        });
    }

    let world = GameServer::default_world();

    match state.game_server.start_game(world, players, None) {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "status": "running",
                "message": "Game started. Connect to /ws/game for live updates."
            })),
        )
            .into_response(),
        Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, &e).into_response(),
    }
}

async fn game_status(State(state): State<AppState>) -> impl IntoResponse {
    let running = state.game_server.is_running();
    (
        StatusCode::OK,
        Json(json!({
            "running": running,
        })),
    )
        .into_response()
}

async fn stop_game(State(state): State<AppState>) -> impl IntoResponse {
    if !state.game_server.is_running() {
        return json_error(StatusCode::BAD_REQUEST, "No game is running").into_response();
    }
    state.game_server.stop_game();
    (StatusCode::OK, Json(json!({ "status": "stopping" }))).into_response()
}
