// Game server: manages a running game instance and broadcasts state to WebSocket clients.

use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tokio::sync::broadcast;

use crate::replay::ReplayRecorder;

use super::config::*;
use super::game::{Game, GameSnapshot, PlayerSnapshot, WorldSnapshot};
use super::world::{RandomMapParams, World};

/// Result of a completed game, passed to the on_complete callback.
pub struct GameResult {
    pub match_id: Option<i64>,
    pub winner_player_index: Option<usize>,
    pub player_scores: Vec<PlayerScore>,
    pub replay_data: Vec<u8>,
    pub tick_count: i32,
}

/// Score data for one player in a completed game.
pub struct PlayerScore {
    pub player_index: usize,
    pub bot_version_id: i64,
    pub score: i32,
    pub creatures_spawned: i32,
    pub creatures_killed: i32,
    pub creatures_lost: i32,
}

/// Metadata about an available map file.
#[derive(Debug, Clone, Serialize)]
pub struct MapInfo {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub description: String,
}

/// Scan a directory for `*.json` map files and return their metadata, sorted by name.
pub fn list_maps(maps_dir: &Path) -> Vec<MapInfo> {
    let mut maps = Vec::new();

    let entries = match std::fs::read_dir(maps_dir) {
        Ok(e) => e,
        Err(_) => return maps,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // Parse just enough to get width/height
        #[derive(serde::Deserialize)]
        struct MapMeta {
            width: usize,
            height: usize,
        }
        if let Ok(meta) = serde_json::from_str::<MapMeta>(&contents) {
            maps.push(MapInfo {
                name: stem,
                width: meta.width,
                height: meta.height,
                description: format!("{}x{} map", meta.width, meta.height),
            });
        }
    }

    maps.sort_by(|a, b| a.name.cmp(&b.name));
    maps
}

/// Load a map by name from the given directory. Returns a World or an error message.
pub fn load_map(maps_dir: &Path, name: &str) -> Result<World, String> {
    let path = maps_dir.join(format!("{}.json", name));
    let contents = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read map '{}': {}", name, e))?;
    World::from_json(&contents)
}

/// Messages sent from the game loop to WebSocket clients.
#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type")]
pub enum GameMessage {
    /// Initial world state (tiles, dimensions, koth position).
    #[serde(rename = "world")]
    WorldInit(WorldSnapshot),
    /// Per-tick game state snapshot.
    #[serde(rename = "snapshot")]
    Snapshot(GameSnapshot),
    /// Game has ended.
    #[serde(rename = "game_end")]
    GameEnd {
        winner: Option<u32>,
        final_scores: Vec<PlayerSnapshot>,
    },
    /// A player failed to load (e.g. Lua syntax error).
    #[serde(rename = "player_load_error")]
    PlayerLoadError {
        player_name: String,
        error: String,
    },
}

/// A player entry for starting a game.
pub struct PlayerEntry {
    pub name: String,
    pub code: String,
}

/// Manages a single game instance, running the game loop on a dedicated thread
/// and broadcasting snapshots to WebSocket subscribers via a broadcast channel.
pub struct GameServer {
    broadcast_tx: broadcast::Sender<String>,
    running: Arc<AtomicBool>,
    /// Cached world JSON so late-joining WS clients get the world state.
    world_json: Arc<Mutex<Option<String>>>,
}

impl GameServer {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            broadcast_tx: tx,
            running: Arc::new(AtomicBool::new(false)),
            world_json: Arc::new(Mutex::new(None)),
        }
    }

    /// Subscribe to game messages. Returns a receiver that yields JSON strings.
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.broadcast_tx.subscribe()
    }

    /// Get the cached world JSON for late-joining clients.
    pub fn world_json(&self) -> Option<String> {
        self.world_json.lock().unwrap().clone()
    }

    /// Whether a game is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Stop the currently running game (if any).
    pub fn stop_game(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Start a game with the given world and players.
    /// The game loop runs on a dedicated OS thread (Game is !Send due to Rc<RefCell<>>).
    /// Each tick broadcasts a GameSnapshot as JSON to all subscribers.
    /// The game runs for `max_ticks` ticks (default 6000 = 10 minutes at 100ms/tick).
    pub fn start_game(
        &self,
        world: World,
        players: Vec<PlayerEntry>,
        max_ticks: Option<u64>,
    ) -> Result<(), String> {
        self.start_game_with_callback(world, players, max_ticks, None, vec![], false, None)
    }

    /// Start a game with a completion callback for Elo updates, replay saving, etc.
    ///
    /// - `match_id`: optional DB match ID to include in the GameResult
    /// - `bot_version_ids`: one per player, same order as `players` vec
    /// - `headless`: if true, skip the 100ms per-tick sleep (fast mode)
    /// - `on_complete`: called on the game thread when the game finishes
    pub fn start_game_with_callback(
        &self,
        world: World,
        players: Vec<PlayerEntry>,
        max_ticks: Option<u64>,
        match_id: Option<i64>,
        bot_version_ids: Vec<i64>,
        headless: bool,
        on_complete: Option<Box<dyn FnOnce(GameResult) + Send + 'static>>,
    ) -> Result<(), String> {
        if self.is_running() {
            return Err("A game is already running".into());
        }

        let tx = self.broadcast_tx.clone();
        let running = self.running.clone();
        let world_json = self.world_json.clone();
        let max_ticks = max_ticks.unwrap_or(6000);

        running.store(true, Ordering::Relaxed);

        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                let mut game = Game::new(world);
                let mut recorder = ReplayRecorder::new();

                // Add players and spawn initial creatures
                let mut player_ids = Vec::new();
                for entry in &players {
                    match game.add_player(&entry.name, &entry.code) {
                        Ok(pid) => {
                            player_ids.push(pid);
                        }
                        Err(e) => {
                            tracing::error!("Failed to add player '{}': {}", entry.name, e);
                            let err_msg = GameMessage::PlayerLoadError {
                                player_name: entry.name.clone(),
                                error: e.to_string(),
                            };
                            if let Ok(json) = serde_json::to_string(&err_msg) {
                                let _ = tx.send(json);
                            }
                        }
                    }
                }

                // Auto-generate food spawners if the map has none
                game.ensure_food_spawners();

                // Place initial food from spawners
                game.seed_initial_food();

                // Spawn initial creatures for each player on random walkable tiles
                for &pid in &player_ids {
                    let initial_creatures = 2;
                    for _ in 0..initial_creatures {
                        let tile = game.world.borrow().find_plain_tile();
                        if let Some((tx_pos, ty_pos)) = tile {
                            let cx = World::tile_center(tx_pos);
                            let cy = World::tile_center(ty_pos);
                            game.spawn_creature(pid, cx, cy, CREATURE_SMALL);
                        }
                    }
                }

                // Send initial world snapshot and cache it for late joiners
                let world_snap = game.world_snapshot();
                let world_msg = GameMessage::WorldInit(world_snap);
                if let Ok(json) = serde_json::to_string(&world_msg) {
                    *world_json.lock().unwrap() = Some(json.clone());
                    let _ = tx.send(json.clone());
                    recorder.record_message(&json);
                }

                // Game loop
                let mut tick_count: u64 = 0;
                let mut winner: Option<u32> = None;
                while running.load(Ordering::Relaxed) && tick_count < max_ticks {
                    game.tick();
                    tick_count += 1;

                    let snapshot = game.snapshot();
                    let msg = GameMessage::Snapshot(snapshot);
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let _ = tx.send(json.clone());
                        recorder.record_message(&json);
                    }

                    // Check win condition: only one player has creatures left
                    if let Some(w) = game.check_winner() {
                        tracing::info!(player_id = w, "Player won — last one standing");
                        winner = Some(w);
                        break;
                    }

                    if !headless {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }

                // Game ended -- send final scores
                let final_snap = game.snapshot();
                let winner = winner.or_else(|| {
                    final_snap
                        .players
                        .iter()
                        .max_by_key(|p| p.score)
                        .map(|p| p.id)
                });

                let end_msg = GameMessage::GameEnd {
                    winner,
                    final_scores: final_snap.players.clone(),
                };
                if let Ok(json) = serde_json::to_string(&end_msg) {
                    let _ = tx.send(json.clone());
                    recorder.record_message(&json);
                }

                // Build GameResult and invoke callback
                if let Some(callback) = on_complete {
                    // Determine winner player index (index into players vec)
                    let winner_player_index = winner
                        .and_then(|winner_id| player_ids.iter().position(|&pid| pid == winner_id));

                    // Build player scores from the final snapshot
                    let player_scores: Vec<PlayerScore> = final_snap
                        .players
                        .iter()
                        .enumerate()
                        .map(|(i, ps)| {
                            let pid = player_ids.get(i).copied().unwrap_or(0);
                            let stats = game.player_stats(pid);
                            PlayerScore {
                                player_index: i,
                                bot_version_id: bot_version_ids.get(i).copied().unwrap_or(0),
                                score: ps.score,
                                creatures_spawned: stats.creatures_spawned,
                                creatures_killed: stats.creatures_killed,
                                creatures_lost: stats.creatures_lost,
                            }
                        })
                        .collect();

                    let game_result = GameResult {
                        match_id,
                        winner_player_index,
                        player_scores,
                        replay_data: recorder.finish(),
                        tick_count: tick_count as i32,
                    };

                    callback(game_result);
                }
            }));

            if let Err(panic_info) = result {
                let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                tracing::error!("Game thread panicked: {}", msg);
            }

            *world_json.lock().unwrap() = None;
            running.store(false, Ordering::Relaxed);
        });

        Ok(())
    }

    /// Create a default world using random map generation.
    pub fn default_world() -> World {
        World::generate_random(RandomMapParams::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_server_new() {
        let server = GameServer::new();
        assert!(!server.is_running());
    }

    #[test]
    fn test_default_world() {
        let world = GameServer::default_world();
        assert_eq!(world.width, 30);
        assert_eq!(world.height, 30);
        // Border should always be solid
        assert!(!world.is_walkable(0, 0));
        assert!(!world.is_walkable(29, 29));
        // Should have some walkable tiles
        assert!(world.find_plain_tile().is_some());
        // Should have food spawners
        assert!(!world.food_spawners.is_empty());
    }

    #[test]
    fn test_game_message_serialization() {
        let snap = GameSnapshot {
            game_time: 100,
            creatures: vec![],
            players: vec![PlayerSnapshot {
                id: 1,
                name: "test".to_string(),
                score: 42,
                color: 0,
                num_creatures: 3,
                output: vec![],
            }],
            king_player_id: Some(1),
        };
        let msg = GameMessage::Snapshot(snap);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"snapshot\""));
        assert!(json.contains("\"game_time\":100"));

        let end_msg = GameMessage::GameEnd {
            winner: Some(1),
            final_scores: vec![],
        };
        let json = serde_json::to_string(&end_msg).unwrap();
        assert!(json.contains("\"type\":\"game_end\""));
    }

    #[test]
    fn test_start_game_while_running() {
        let server = GameServer::new();
        let world = GameServer::default_world();
        // Start a game
        let result = server.start_game(world, vec![], Some(5));
        assert!(result.is_ok());

        // Brief sleep to let thread start
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Try to start another while first is running
        let world2 = GameServer::default_world();
        let result2 = server.start_game(world2, vec![], Some(5));
        assert!(result2.is_err());

        server.stop_game();
        // Wait for thread to finish
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
