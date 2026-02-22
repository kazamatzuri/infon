// HTTP API routes (bot CRUD, game control, etc.)

pub mod ws;

use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use std::path::PathBuf;

use crate::auth::{AuthUser, OptionalAuthUser};
use crate::db::Database;
use crate::engine::server::{self, GameServer, PlayerEntry};
use crate::engine::world::World;
use crate::queue::GameQueue;
use crate::rate_limit::{RateLimitType, RateLimiter};

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
pub struct UpdateTournamentRequest {
    pub name: Option<String>,
    pub map: Option<String>,
    pub format: Option<String>,
    pub config: Option<String>,
}

#[derive(Deserialize)]
pub struct AddTournamentEntryRequest {
    pub bot_version_id: i64,
    pub slot_name: Option<String>,
}

#[derive(Deserialize)]
pub struct StartGameRequest {
    pub players: Vec<StartGamePlayer>,
    pub map: Option<String>,
}

#[derive(Deserialize)]
pub struct StartGamePlayer {
    pub bot_version_id: i64,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct SetActiveVersionRequest {
    pub version_id: i64,
}

#[derive(Deserialize)]
pub struct UpdateBotVersionRequest {
    pub is_archived: Option<bool>,
}

#[derive(Deserialize)]
pub struct ChallengeRequest {
    pub bot_version_id: i64,
    pub opponent_bot_version_id: i64,
    pub format: Option<String>,
    pub headless: Option<bool>,
    pub map: Option<String>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateTeamRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateTeamVersionRequest {
    pub bot_version_a: i64,
    pub bot_version_b: i64,
}

#[derive(Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub scopes: Option<String>,
}

// ── Shared application state ─────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub game_server: Arc<GameServer>,
    pub maps_dir: PathBuf,
    pub rate_limiter: RateLimiter,
    pub game_queue: GameQueue,
}

// ── Error helper ──────────────────────────────────────────────────────

fn json_error(status: StatusCode, msg: &str) -> impl IntoResponse {
    (status, Json(json!({ "error": msg })))
}

fn internal_error(e: sqlx::Error) -> impl IntoResponse {
    tracing::error!("Database error: {e}");
    json_error(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
}

// ── Router ────────────────────────────────────────────────────────────

pub fn router(
    db: Arc<Database>,
    game_server: Arc<GameServer>,
    rate_limiter: RateLimiter,
    game_queue: GameQueue,
) -> Router {
    let maps_dir = PathBuf::from("../data/maps");
    let state = AppState {
        db,
        game_server,
        maps_dir,
        rate_limiter,
        game_queue,
    };

    Router::new()
        // Maps
        .route("/api/maps", get(list_maps))
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
            get(get_bot_version).put(update_bot_version),
        )
        .route("/api/bots/{id}/active-version", put(set_active_version))
        .route("/api/bots/{id}/stats", get(get_bot_stats))
        // Matches
        .route("/api/matches", get(list_matches))
        .route("/api/matches/challenge", post(create_challenge))
        .route("/api/matches/{id}", get(get_match))
        .route("/api/matches/{id}/replay", get(get_match_replay))
        // Queue
        .route("/api/queue/status", get(queue_status))
        // Tournaments
        .route(
            "/api/tournaments",
            get(list_tournaments).post(create_tournament),
        )
        .route(
            "/api/tournaments/{id}",
            get(get_tournament).put(update_tournament),
        )
        .route(
            "/api/tournaments/{id}/standings",
            get(get_tournament_standings),
        )
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
        .route("/api/tournaments/{id}/results", get(get_tournament_results))
        // Tournament run
        .route("/api/tournaments/{id}/run", post(run_tournament))
        // Leaderboards
        .route("/api/leaderboards/1v1", get(leaderboard_1v1))
        .route("/api/leaderboards/ffa", get(leaderboard_ffa))
        .route("/api/leaderboards/2v2", get(leaderboard_2v2))
        // Teams
        .route("/api/teams", get(list_teams).post(create_team))
        .route(
            "/api/teams/{id}",
            get(get_team).put(update_team).delete(delete_team),
        )
        .route(
            "/api/teams/{id}/versions",
            get(list_team_versions).post(create_team_version),
        )
        // Game control
        .route("/api/game/start", post(start_game))
        .route("/api/game/status", get(game_status))
        .route("/api/game/stop", post(stop_game))
        // API keys
        .route("/api/api-keys", get(list_api_keys).post(create_api_key))
        .route("/api/api-keys/{id}", delete(delete_api_key))
        // Documentation
        .route("/api/docs/lua-api", get(get_lua_api_docs))
        .route("/llms.txt", get(get_llms_txt))
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
    auth: OptionalAuthUser,
    Json(req): Json<CreateBotRequest>,
) -> impl IntoResponse {
    if req.name.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "name is required").into_response();
    }
    let description = req.description.unwrap_or_default();
    let owner_id = auth.0.map(|c| c.sub);
    match state.db.create_bot(&req.name, &description, owner_id).await {
        Ok(bot) => (StatusCode::CREATED, Json(json!(bot))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn get_bot(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
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

async fn delete_bot(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
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
    match state
        .db
        .create_bot_version(bot_id, &req.code, &api_type)
        .await
    {
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
        Ok(None) => json_error(StatusCode::NOT_FOUND, "Bot version not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

// ── Bot version management handlers ──────────────────────────────────

async fn update_bot_version(
    State(state): State<AppState>,
    Path((_bot_id, version_id)): Path<(i64, i64)>,
    Json(req): Json<UpdateBotVersionRequest>,
) -> impl IntoResponse {
    if let Some(archived) = req.is_archived {
        match state.db.archive_version(version_id, archived).await {
            Ok(true) => {}
            Ok(false) => {
                return json_error(StatusCode::NOT_FOUND, "Version not found").into_response()
            }
            Err(e) => return internal_error(e).into_response(),
        }
    }
    match state.db.get_bot_version_by_id(version_id).await {
        Ok(Some(v)) => (StatusCode::OK, Json(json!(v))).into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, "Version not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn set_active_version(
    State(state): State<AppState>,
    Path(bot_id): Path<i64>,
    Json(req): Json<SetActiveVersionRequest>,
) -> impl IntoResponse {
    // Verify the version belongs to this bot
    match state.db.get_bot_version(bot_id, req.version_id).await {
        Ok(None) => {
            return json_error(StatusCode::NOT_FOUND, "Version not found for this bot")
                .into_response()
        }
        Err(e) => return internal_error(e).into_response(),
        Ok(Some(_)) => {}
    }
    match state.db.set_active_version(bot_id, req.version_id).await {
        Ok(true) => match state.db.get_bot(bot_id).await {
            Ok(Some(bot)) => (StatusCode::OK, Json(json!(bot))).into_response(),
            _ => StatusCode::OK.into_response(),
        },
        Ok(false) => json_error(StatusCode::NOT_FOUND, "Bot not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn get_bot_stats(
    State(state): State<AppState>,
    Path(bot_id): Path<i64>,
) -> impl IntoResponse {
    match state.db.list_bot_versions(bot_id).await {
        Ok(versions) => {
            let stats: Vec<serde_json::Value> = versions
                .iter()
                .map(|v| {
                    json!({
                        "version_id": v.id,
                        "version": v.version,
                        "elo_rating": v.elo_rating,
                        "elo_1v1": v.elo_1v1,
                        "elo_peak": v.elo_peak,
                        "games_played": v.games_played,
                        "wins": v.wins,
                        "losses": v.losses,
                        "draws": v.draws,
                        "win_rate": if v.games_played > 0 {
                            v.wins as f64 / v.games_played as f64
                        } else {
                            0.0
                        },
                        "ffa_placement_points": v.ffa_placement_points,
                        "ffa_games": v.ffa_games,
                        "avg_ffa_placement": if v.ffa_games > 0 {
                            v.ffa_placement_points as f64 / v.ffa_games as f64
                        } else {
                            0.0
                        },
                        "creatures_spawned": v.creatures_spawned,
                        "creatures_killed": v.creatures_killed,
                        "creatures_lost": v.creatures_lost,
                        "total_score": v.total_score,
                        "avg_score": if v.games_played > 0 {
                            v.total_score as f64 / v.games_played as f64
                        } else {
                            0.0
                        },
                        "is_archived": v.is_archived,
                    })
                })
                .collect();
            (StatusCode::OK, Json(json!(stats))).into_response()
        }
        Err(e) => internal_error(e).into_response(),
    }
}

async fn get_match(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    let m = match state.db.get_match(id).await {
        Ok(Some(m)) => m,
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "Match not found").into_response(),
        Err(e) => return internal_error(e).into_response(),
    };
    let participants = match state.db.get_match_participants(id).await {
        Ok(p) => p,
        Err(e) => return internal_error(e).into_response(),
    };
    (
        StatusCode::OK,
        Json(json!({
            "match": m,
            "participants": participants,
        })),
    )
        .into_response()
}

// ── Replay handler ────────────────────────────────────────────────────

async fn get_match_replay(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    // Check match exists
    match state.db.get_match(id).await {
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "Match not found").into_response(),
        Err(e) => return internal_error(e).into_response(),
        Ok(Some(_)) => {}
    }

    let replay = match state.db.get_replay(id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return json_error(StatusCode::NOT_FOUND, "Replay not found for this match")
                .into_response()
        }
        Err(e) => return internal_error(e).into_response(),
    };

    // Decompress the replay data
    let json_str = match crate::replay::decompress_replay(&replay.data) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to decompress replay: {e}");
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to decompress replay",
            )
            .into_response();
        }
    };

    // Parse the messages array from the decompressed JSON
    let messages: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to parse replay JSON: {e}");
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse replay data",
            )
            .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(json!({
            "match_id": replay.match_id,
            "tick_count": replay.tick_count,
            "messages": messages,
        })),
    )
        .into_response()
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

async fn get_tournament(State(state): State<AppState>, Path(id): Path<i64>) -> impl IntoResponse {
    match state.db.get_tournament(id).await {
        Ok(Some(tournament)) => (StatusCode::OK, Json(json!(tournament))).into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, "Tournament not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn update_tournament(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateTournamentRequest>,
) -> impl IntoResponse {
    // Validate format if provided
    if let Some(ref fmt) = req.format {
        if crate::tournament::TournamentFormat::from_str_name(fmt).is_none() {
            return json_error(
                StatusCode::BAD_REQUEST,
                "Invalid format. Use 'round_robin', 'single_elimination', or 'swiss_N'",
            )
            .into_response();
        }
    }
    match state
        .db
        .update_tournament(
            id,
            req.name.as_deref(),
            req.map.as_deref(),
            req.format.as_deref(),
            req.config.as_deref(),
        )
        .await
    {
        Ok(Some(tournament)) => (StatusCode::OK, Json(json!(tournament))).into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, "Tournament not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn get_tournament_standings(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.db.get_tournament(id).await {
        Ok(None) => {
            return json_error(StatusCode::NOT_FOUND, "Tournament not found").into_response()
        }
        Err(e) => return internal_error(e).into_response(),
        Ok(Some(_)) => {}
    }
    match state.db.get_tournament_standings(id).await {
        Ok(standings) => (StatusCode::OK, Json(json!(standings))).into_response(),
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
    match state
        .db
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
        let version = match state.db.get_bot_version_by_id(entry.bot_version_id).await {
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

    // Create world from tournament map setting
    let map_name = if tournament.map == "default" {
        None
    } else {
        Some(tournament.map.clone())
    };
    let world = match resolve_map(&state.maps_dir, &map_name) {
        Ok(w) => w,
        Err(e) => {
            return json_error(StatusCode::BAD_REQUEST, &format!("Invalid map: {}", e))
                .into_response()
        }
    };

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

// ── Map handlers ─────────────────────────────────────────────────────

async fn list_maps(State(state): State<AppState>) -> impl IntoResponse {
    let mut maps = server::list_maps(&state.maps_dir);
    // Prepend a "Random" pseudo-entry
    maps.insert(
        0,
        server::MapInfo {
            name: "random".to_string(),
            width: 30,
            height: 30,
            description: "Randomly generated map".to_string(),
        },
    );
    (StatusCode::OK, Json(json!(maps))).into_response()
}

/// Resolve an optional map name to a World.
fn resolve_map(maps_dir: &std::path::Path, map: &Option<String>) -> Result<World, String> {
    use crate::engine::world::RandomMapParams;
    match map.as_deref() {
        None | Some("random") => Ok(World::generate_random(RandomMapParams::default())),
        Some(name) => server::load_map(maps_dir, name),
    }
}

// ── Leaderboard handlers ─────────────────────────────────────────────

async fn leaderboard_1v1(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0).max(0);
    match state.db.leaderboard_1v1(limit, offset).await {
        Ok(entries) => (StatusCode::OK, Json(json!(entries))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn leaderboard_ffa(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0).max(0);
    match state.db.leaderboard_ffa(limit, offset).await {
        Ok(entries) => (StatusCode::OK, Json(json!(entries))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn leaderboard_2v2(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0).max(0);
    match state.db.leaderboard_2v2(limit, offset).await {
        Ok(entries) => (StatusCode::OK, Json(json!(entries))).into_response(),
        Err(e) => internal_error(e).into_response(),
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

    let world = match resolve_map(&state.maps_dir, &req.map) {
        Ok(w) => w,
        Err(e) => {
            return json_error(StatusCode::BAD_REQUEST, &format!("Invalid map: {}", e))
                .into_response()
        }
    };

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

// ── Match list handler ────────────────────────────────────────────────

async fn list_matches(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).min(100);
    match state.db.list_recent_matches(limit).await {
        Ok(matches) => (StatusCode::OK, Json(json!(matches))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

// ── Challenge handler ────────────────────────────────────────────────

async fn create_challenge(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ChallengeRequest>,
) -> impl IntoResponse {
    let user_id = auth.0.sub;
    let headless = req.headless.unwrap_or(false);
    let format = req.format.clone().unwrap_or_else(|| "1v1".to_string());

    if format != "1v1" && format != "ffa" {
        return json_error(StatusCode::BAD_REQUEST, "format must be '1v1' or 'ffa'")
            .into_response();
    }

    // Validate both bot versions exist
    let version_a = match state.db.get_bot_version_by_id(req.bot_version_id).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return json_error(
                StatusCode::NOT_FOUND,
                &format!("Bot version {} not found", req.bot_version_id),
            )
            .into_response();
        }
        Err(e) => return internal_error(e).into_response(),
    };

    let version_b = match state
        .db
        .get_bot_version_by_id(req.opponent_bot_version_id)
        .await
    {
        Ok(Some(v)) => v,
        Ok(None) => {
            return json_error(
                StatusCode::NOT_FOUND,
                &format!("Bot version {} not found", req.opponent_bot_version_id),
            )
            .into_response();
        }
        Err(e) => return internal_error(e).into_response(),
    };

    // Check rate limits
    let limit_type = if headless {
        RateLimitType::HeadlessChallenges
    } else {
        RateLimitType::LiveChallenges
    };
    if let Err(e) = state.rate_limiter.check_limit(user_id, limit_type) {
        return json_error(StatusCode::TOO_MANY_REQUESTS, &e.to_string()).into_response();
    }

    // For live games, also check concurrent game limit
    if !headless {
        if let Err(e) = state
            .rate_limiter
            .check_limit(user_id, RateLimitType::LiveGames)
        {
            return json_error(StatusCode::TOO_MANY_REQUESTS, &e.to_string()).into_response();
        }
    }

    // Create match record in DB
    let map_name = req.map.clone().unwrap_or_else(|| "random".to_string());
    let m = match state.db.create_match(&format, &map_name).await {
        Ok(m) => m,
        Err(e) => return internal_error(e).into_response(),
    };

    // Add participants
    if let Err(e) = state
        .db
        .add_match_participant(m.id, req.bot_version_id, 0)
        .await
    {
        return internal_error(e).into_response();
    }
    if let Err(e) = state
        .db
        .add_match_participant(m.id, req.opponent_bot_version_id, 1)
        .await
    {
        return internal_error(e).into_response();
    }

    if headless {
        // TODO: Run headless game in background thread
        // For now, just queue it
        state.game_queue.enqueue(crate::queue::QueueEntry {
            match_id: m.id,
            players: vec![
                crate::queue::QueuePlayer {
                    bot_version_id: req.bot_version_id,
                    name: format!("Player 1 (v{})", version_a.version),
                },
                crate::queue::QueuePlayer {
                    bot_version_id: req.opponent_bot_version_id,
                    name: format!("Player 2 (v{})", version_b.version),
                },
            ],
            map: req.map.clone(),
            headless: true,
        });

        return (
            StatusCode::CREATED,
            Json(json!({
                "match_id": m.id,
                "status": "queued",
                "message": "Headless challenge queued."
            })),
        )
            .into_response();
    }

    // Live game: start via GameServer
    if state.game_server.is_running() {
        // Queue it instead of rejecting
        state.game_queue.enqueue(crate::queue::QueueEntry {
            match_id: m.id,
            players: vec![
                crate::queue::QueuePlayer {
                    bot_version_id: req.bot_version_id,
                    name: format!("Player 1 (v{})", version_a.version),
                },
                crate::queue::QueuePlayer {
                    bot_version_id: req.opponent_bot_version_id,
                    name: format!("Player 2 (v{})", version_b.version),
                },
            ],
            map: req.map.clone(),
            headless: false,
        });

        return (
            StatusCode::CREATED,
            Json(json!({
                "match_id": m.id,
                "status": "queued",
                "message": "A game is already running. Challenge queued.",
                "queue_depth": state.game_queue.depth(),
            })),
        )
            .into_response();
    }

    // Resolve map
    let world = match resolve_map(&state.maps_dir, &req.map) {
        Ok(w) => w,
        Err(e) => {
            return json_error(StatusCode::BAD_REQUEST, &format!("Invalid map: {}", e))
                .into_response();
        }
    };

    // Get bot names from the DB
    let name_a = match state.db.get_bot(version_a.bot_id).await {
        Ok(Some(b)) => format!("{} v{}", b.name, version_a.version),
        _ => format!("Player 1 (v{})", version_a.version),
    };
    let name_b = match state.db.get_bot(version_b.bot_id).await {
        Ok(Some(b)) => format!("{} v{}", b.name, version_b.version),
        _ => format!("Player 2 (v{})", version_b.version),
    };

    let players = vec![
        PlayerEntry {
            name: name_a,
            code: version_a.code,
            api_type: version_a.api_type,
        },
        PlayerEntry {
            name: name_b,
            code: version_b.code,
            api_type: version_b.api_type,
        },
    ];

    match state.game_server.start_game(world, players, None) {
        Ok(()) => (
            StatusCode::CREATED,
            Json(json!({
                "match_id": m.id,
                "status": "running",
                "message": "Game started. Connect to /ws/game for live updates."
            })),
        )
            .into_response(),
        Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, &e).into_response(),
    }
}

// ── Queue status handler ─────────────────────────────────────────────

async fn queue_status(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.game_queue.status();
    (StatusCode::OK, Json(json!(status))).into_response()
}

// ── Team handlers ──────────────────────────────────────────────────────

async fn list_teams(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    match state.db.list_teams_by_owner(auth.0.sub).await {
        Ok(teams) => (StatusCode::OK, Json(json!(teams))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn create_team(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateTeamRequest>,
) -> impl IntoResponse {
    if req.name.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "name is required").into_response();
    }
    match state.db.create_team(auth.0.sub, &req.name).await {
        Ok(team) => (StatusCode::CREATED, Json(json!(team))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn get_team(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.db.get_team(id).await {
        Ok(Some(team)) => {
            if team.owner_id != auth.0.sub {
                return json_error(StatusCode::FORBIDDEN, "Not your team").into_response();
            }
            (StatusCode::OK, Json(json!(team))).into_response()
        }
        Ok(None) => json_error(StatusCode::NOT_FOUND, "Team not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn update_team(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateTeamRequest>,
) -> impl IntoResponse {
    if req.name.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "name is required").into_response();
    }
    // Check ownership
    match state.db.get_team(id).await {
        Ok(Some(team)) => {
            if team.owner_id != auth.0.sub {
                return json_error(StatusCode::FORBIDDEN, "Not your team").into_response();
            }
        }
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "Team not found").into_response(),
        Err(e) => return internal_error(e).into_response(),
    }
    match state.db.update_team_name(id, &req.name).await {
        Ok(Some(team)) => (StatusCode::OK, Json(json!(team))).into_response(),
        Ok(None) => json_error(StatusCode::NOT_FOUND, "Team not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn delete_team(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    // Check ownership
    match state.db.get_team(id).await {
        Ok(Some(team)) => {
            if team.owner_id != auth.0.sub {
                return json_error(StatusCode::FORBIDDEN, "Not your team").into_response();
            }
        }
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "Team not found").into_response(),
        Err(e) => return internal_error(e).into_response(),
    }
    match state.db.delete_team(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => json_error(StatusCode::NOT_FOUND, "Team not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

// ── Team version handlers ──────────────────────────────────────────────

async fn list_team_versions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(team_id): Path<i64>,
) -> impl IntoResponse {
    // Check team exists and ownership
    match state.db.get_team(team_id).await {
        Ok(Some(team)) => {
            if team.owner_id != auth.0.sub {
                return json_error(StatusCode::FORBIDDEN, "Not your team").into_response();
            }
        }
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "Team not found").into_response(),
        Err(e) => return internal_error(e).into_response(),
    }
    match state.db.list_team_versions(team_id).await {
        Ok(versions) => (StatusCode::OK, Json(json!(versions))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn create_team_version(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(team_id): Path<i64>,
    Json(req): Json<CreateTeamVersionRequest>,
) -> impl IntoResponse {
    // Check team exists and ownership
    match state.db.get_team(team_id).await {
        Ok(Some(team)) => {
            if team.owner_id != auth.0.sub {
                return json_error(StatusCode::FORBIDDEN, "Not your team").into_response();
            }
        }
        Ok(None) => return json_error(StatusCode::NOT_FOUND, "Team not found").into_response(),
        Err(e) => return internal_error(e).into_response(),
    }
    // Verify both bot versions exist
    match state.db.get_bot_version_by_id(req.bot_version_a).await {
        Ok(None) => {
            return json_error(
                StatusCode::BAD_REQUEST,
                &format!("Bot version {} not found", req.bot_version_a),
            )
            .into_response()
        }
        Err(e) => return internal_error(e).into_response(),
        Ok(Some(_)) => {}
    }
    match state.db.get_bot_version_by_id(req.bot_version_b).await {
        Ok(None) => {
            return json_error(
                StatusCode::BAD_REQUEST,
                &format!("Bot version {} not found", req.bot_version_b),
            )
            .into_response()
        }
        Err(e) => return internal_error(e).into_response(),
        Ok(Some(_)) => {}
    }
    match state
        .db
        .create_team_version(team_id, req.bot_version_a, req.bot_version_b)
        .await
    {
        Ok(version) => (StatusCode::CREATED, Json(json!(version))).into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

// ── API Key handlers ──────────────────────────────────────────────────

async fn list_api_keys(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    match state.db.list_api_tokens(auth.0.sub).await {
        Ok(tokens) => {
            // Don't expose the hash to the client
            let keys: Vec<serde_json::Value> = tokens
                .iter()
                .map(|t| {
                    json!({
                        "id": t.id,
                        "name": t.name,
                        "scopes": t.scopes,
                        "last_used_at": t.last_used_at,
                        "created_at": t.created_at,
                    })
                })
                .collect();
            (StatusCode::OK, Json(json!(keys))).into_response()
        }
        Err(e) => internal_error(e).into_response(),
    }
}

async fn create_api_key(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiKeyRequest>,
) -> impl IntoResponse {
    if req.name.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "name is required").into_response();
    }

    let scopes = req
        .scopes
        .unwrap_or_else(|| "bots:read,matches:read,leaderboard:read".to_string());

    // Generate random token
    let raw_token = format!("infon_{}", hex::encode(generate_random_bytes()));

    // Hash it for storage
    let token_hash = hash_token(&raw_token);

    match state
        .db
        .create_api_token(auth.0.sub, &req.name, &token_hash, &scopes)
        .await
    {
        Ok(token_record) => (
            StatusCode::CREATED,
            Json(json!({
                "id": token_record.id,
                "name": token_record.name,
                "token": raw_token,
                "scopes": token_record.scopes,
                "created_at": token_record.created_at,
            })),
        )
            .into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

async fn delete_api_key(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.db.delete_api_token(id, auth.0.sub).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => json_error(StatusCode::NOT_FOUND, "API key not found").into_response(),
        Err(e) => internal_error(e).into_response(),
    }
}

fn generate_random_bytes() -> [u8; 32] {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

// ── Documentation handlers ────────────────────────────────────────────

async fn get_lua_api_docs() -> impl IntoResponse {
    let content = include_str!("../../../docs/lua-api-reference.md");
    (
        StatusCode::OK,
        [("content-type", "text/markdown; charset=utf-8")],
        content,
    )
        .into_response()
}

async fn get_llms_txt() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        crate::llms_txt::LLMS_TXT,
    )
        .into_response()
}
