// Simple FIFO game queue for pending match requests.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::api::{build_game_completion_callback, resolve_map};
use crate::db::Database;
use crate::engine::server::{GameServer, PlayerEntry};
use crate::metrics;

/// A pending match in the queue.
#[derive(Debug, Clone)]
pub struct QueueEntry {
    pub match_id: i64,
    pub players: Vec<QueuePlayer>,
    pub map: Option<String>,
    pub headless: bool,
}

/// A player in a queued match.
#[derive(Debug, Clone)]
pub struct QueuePlayer {
    pub bot_version_id: i64,
    pub name: String,
}

/// Status of the game queue.
#[derive(Debug, Clone, Serialize)]
pub struct QueueStatus {
    pub depth: usize,
    pub estimated_wait_seconds: f64,
}

/// Thread-safe FIFO game queue.
#[derive(Debug, Clone)]
pub struct GameQueue {
    inner: Arc<Mutex<VecDeque<QueueEntry>>>,
}

impl GameQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Add a match to the back of the queue.
    pub fn enqueue(&self, entry: QueueEntry) {
        let mut queue = self.inner.lock().unwrap();
        queue.push_back(entry);
        metrics::GAME_QUEUE_DEPTH.set(queue.len() as i64);
    }

    /// Remove and return the next match from the front of the queue.
    pub fn dequeue(&self) -> Option<QueueEntry> {
        let mut queue = self.inner.lock().unwrap();
        let result = queue.pop_front();
        metrics::GAME_QUEUE_DEPTH.set(queue.len() as i64);
        result
    }

    /// Peek at the next entry without removing it.
    pub fn peek(&self) -> Option<QueueEntry> {
        let queue = self.inner.lock().unwrap();
        queue.front().cloned()
    }

    /// Get the current queue depth.
    pub fn depth(&self) -> usize {
        let queue = self.inner.lock().unwrap();
        queue.len()
    }

    /// Get the queue status with estimated wait time.
    /// Assumes ~10 minutes (600 seconds) per game as average.
    pub fn status(&self) -> QueueStatus {
        let depth = self.depth();
        let avg_game_seconds = 600.0;
        QueueStatus {
            depth,
            estimated_wait_seconds: depth as f64 * avg_game_seconds,
        }
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        let queue = self.inner.lock().unwrap();
        queue.is_empty()
    }
}

impl Default for GameQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawn a background task that polls the queue and starts games when the server is idle.
pub fn spawn_queue_worker(
    db: Arc<Database>,
    game_server: Arc<GameServer>,
    game_queue: GameQueue,
    maps_dir: PathBuf,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Only process if no game is running
            if game_server.is_running() {
                continue;
            }

            let entry = match game_queue.dequeue() {
                Some(e) => e,
                None => continue,
            };

            // Load bot code for each player
            let mut players = Vec::new();
            let mut version_ids = Vec::new();
            let mut ok = true;
            for qp in &entry.players {
                match db.get_bot_version_by_id(qp.bot_version_id).await {
                    Ok(Some(v)) => {
                        players.push(PlayerEntry {
                            name: qp.name.clone(),
                            code: v.code,
                        });
                        version_ids.push(qp.bot_version_id);
                    }
                    _ => {
                        tracing::error!(
                            "Queue worker: bot version {} not found",
                            qp.bot_version_id
                        );
                        ok = false;
                        break;
                    }
                }
            }

            if !ok {
                continue;
            }

            // Resolve map
            let world = match resolve_map(&maps_dir, &entry.map) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!("Queue worker: invalid map: {e}");
                    continue;
                }
            };

            // Determine format from number of players
            let format = if entry.players.len() == 2 {
                "1v1".to_string()
            } else {
                "ffa".to_string()
            };

            // Build completion callback
            let on_complete = build_game_completion_callback(
                db.clone(),
                entry.match_id,
                version_ids.clone(),
                format,
                game_queue.clone(),
            );

            let max_ticks = if entry.headless { Some(6000) } else { None };
            if let Err(e) = game_server.start_game_with_callback(
                world,
                players,
                max_ticks,
                Some(entry.match_id),
                version_ids,
                entry.headless,
                Some(on_complete),
            ) {
                tracing::error!("Queue worker: failed to start game: {e}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_enqueue_dequeue() {
        let queue = GameQueue::new();

        assert!(queue.is_empty());
        assert_eq!(queue.depth(), 0);
        assert!(queue.dequeue().is_none());

        queue.enqueue(QueueEntry {
            match_id: 1,
            players: vec![QueuePlayer {
                bot_version_id: 10,
                name: "Bot A".into(),
            }],
            map: None,
            headless: false,
        });

        assert!(!queue.is_empty());
        assert_eq!(queue.depth(), 1);

        queue.enqueue(QueueEntry {
            match_id: 2,
            players: vec![],
            map: Some("desert".into()),
            headless: true,
        });

        assert_eq!(queue.depth(), 2);

        // FIFO: first dequeue should be match_id 1
        let first = queue.dequeue().unwrap();
        assert_eq!(first.match_id, 1);
        assert!(!first.headless);

        let second = queue.dequeue().unwrap();
        assert_eq!(second.match_id, 2);
        assert!(second.headless);

        assert!(queue.is_empty());
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn test_queue_peek() {
        let queue = GameQueue::new();

        assert!(queue.peek().is_none());

        queue.enqueue(QueueEntry {
            match_id: 42,
            players: vec![],
            map: None,
            headless: false,
        });

        // Peek should return the entry without removing it
        let peeked = queue.peek().unwrap();
        assert_eq!(peeked.match_id, 42);
        assert_eq!(queue.depth(), 1); // Still there

        // Dequeue should return the same entry
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.match_id, 42);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_queue_status() {
        let queue = GameQueue::new();

        let status = queue.status();
        assert_eq!(status.depth, 0);
        assert_eq!(status.estimated_wait_seconds, 0.0);

        queue.enqueue(QueueEntry {
            match_id: 1,
            players: vec![],
            map: None,
            headless: false,
        });
        queue.enqueue(QueueEntry {
            match_id: 2,
            players: vec![],
            map: None,
            headless: false,
        });

        let status = queue.status();
        assert_eq!(status.depth, 2);
        assert_eq!(status.estimated_wait_seconds, 1200.0); // 2 * 600
    }

    #[test]
    fn test_queue_fifo_order() {
        let queue = GameQueue::new();

        for i in 1..=5 {
            queue.enqueue(QueueEntry {
                match_id: i,
                players: vec![],
                map: None,
                headless: false,
            });
        }

        for expected_id in 1..=5 {
            let entry = queue.dequeue().unwrap();
            assert_eq!(entry.match_id, expected_id);
        }
    }
}
