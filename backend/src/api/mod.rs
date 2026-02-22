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

// ── Shared application state ─────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub game_server: Arc<GameServer>,
    pub maps_dir: PathBuf,
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

pub fn router(db: Arc<Database>, game_server: Arc<GameServer>) -> Router {
    let maps_dir = PathBuf::from("../data/maps");
    let state = AppState {
        db,
        game_server,
        maps_dir,
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
        .route("/api/matches/{id}", get(get_match))
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
