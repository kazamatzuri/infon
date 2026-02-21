// Game server: manages a running game instance and broadcasts state to WebSocket clients.

use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tokio::sync::broadcast;

use super::config::*;
use super::game::{Game, GameSnapshot, PlayerSnapshot, WorldSnapshot};
use super::world::World;

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
}

/// A player entry for starting a game: (name, code, api_type).
pub struct PlayerEntry {
    pub name: String,
    pub code: String,
    pub api_type: String,
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

                // Add players and spawn initial creatures
                let mut player_ids = Vec::new();
                for entry in &players {
                    match game.add_player(&entry.name, &entry.code, &entry.api_type) {
                        Ok(pid) => {
                            player_ids.push(pid);
                        }
                        Err(e) => {
                            tracing::error!("Failed to add player '{}': {}", entry.name, e);
                        }
                    }
                }

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
                    let _ = tx.send(json);
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
                        // If send fails, there are no receivers -- that is fine, keep running
                        let _ = tx.send(json);
                    }

                    // Check win condition: only one player has creatures left
                    if let Some(w) = game.check_winner() {
                        tracing::info!(player_id = w, "Player won — last one standing");
                        winner = Some(w);
                        break;
                    }

                    std::thread::sleep(std::time::Duration::from_millis(100));
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
                    final_scores: final_snap.players,
                };
                if let Ok(json) = serde_json::to_string(&end_msg) {
                    let _ = tx.send(json);
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

    /// Create a default 20x20 world with walkable interior and some food.
    pub fn default_world() -> World {
        let size = 20;
        let mut world = World::new(size, size);

        // Make interior walkable
        for y in 1..size - 1 {
            for x in 1..size - 1 {
                world.set_type(x, y, TILE_PLAIN);
            }
        }

        // Scatter food on several tiles
        let food_positions = [
            (3, 3),
            (3, 16),
            (16, 3),
            (16, 16),
            (10, 10),
            (5, 10),
            (15, 10),
            (10, 5),
            (10, 15),
        ];
        for (fx, fy) in food_positions {
            world.add_food(fx, fy, 5000);
        }

        world
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
        assert_eq!(world.width, 20);
        assert_eq!(world.height, 20);
        assert!(world.is_walkable(5, 5));
        assert!(!world.is_walkable(0, 0));
        assert!(world.get_food(10, 10) > 0);
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
