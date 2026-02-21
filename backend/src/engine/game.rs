use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde::Serialize;

use super::config::*;
use super::creature::Creature;
use super::lua_api::{self, LuaGameState};
use super::player::Player;
use super::world::World;

/// Events that get passed to player_think Lua function.
#[derive(Clone, Debug)]
pub enum GameEvent {
    CreatureSpawned { id: u32, parent: i32 },
    CreatureKilled { id: u32, killer: i32 },
    CreatureAttacked { id: u32, attacker: u32 },
    PlayerCreated { player_id: u32 },
}

/// Snapshot of a creature for rendering / API consumers.
#[derive(Clone, Debug, Serialize)]
pub struct CreatureSnapshot {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub creature_type: u8,
    pub health: i32,
    pub max_health: i32,
    pub food: i32,
    pub state: u8,
    pub player_id: u32,
    pub message: String,
    pub target_id: Option<u32>,
}

/// Snapshot of a player for rendering / API consumers.
#[derive(Clone, Debug, Serialize)]
pub struct PlayerSnapshot {
    pub id: u32,
    pub name: String,
    pub score: i32,
    pub color: u8,
    pub num_creatures: i32,
    pub output: Vec<String>,
}

/// Snapshot of a tile for rendering / API consumers.
#[derive(Clone, Debug, Serialize)]
pub struct TileSnapshot {
    pub x: usize,
    pub y: usize,
    pub food: i32,
    pub tile_type: u8,
    pub gfx: u8,
}

/// Full world snapshot sent on initial connection.
#[derive(Clone, Debug, Serialize)]
pub struct WorldSnapshot {
    pub width: usize,
    pub height: usize,
    pub koth_x: usize,
    pub koth_y: usize,
    pub tiles: Vec<TileSnapshot>,
}

/// Snapshot of the full game state for one tick.
#[derive(Clone, Debug, Serialize)]
pub struct GameSnapshot {
    pub game_time: i64,
    pub creatures: Vec<CreatureSnapshot>,
    pub players: Vec<PlayerSnapshot>,
    pub king_player_id: Option<u32>,
}

/// Top-level game state and tick loop.
pub struct Game {
    pub world: Rc<RefCell<World>>,
    pub creatures: Rc<RefCell<HashMap<u32, Creature>>>,
    pub players: HashMap<u32, Player>,
    pub game_time: i64,
    pub next_creature_id: u32,
    pub next_player_id: u32,
    pub king_player_id: Option<u32>,
    pub tick_delta: i32,
    pub player_scores: Rc<RefCell<HashMap<u32, i32>>>,
    pub player_names: Rc<RefCell<HashMap<u32, String>>>,
    /// Pending events per player (player_id -> events)
    pending_events: HashMap<u32, Vec<GameEvent>>,
}

impl Game {
    pub fn new(world: World) -> Self {
        Game {
            world: Rc::new(RefCell::new(world)),
            creatures: Rc::new(RefCell::new(HashMap::new())),
            players: HashMap::new(),
            game_time: 0,
            next_creature_id: 1,
            next_player_id: 1,
            king_player_id: None,
            tick_delta: 100,
            player_scores: Rc::new(RefCell::new(HashMap::new())),
            player_names: Rc::new(RefCell::new(HashMap::new())),
            pending_events: HashMap::new(),
        }
    }

    /// Add a player with the given bot code and API type ("oo" or "state").
    /// Returns the player ID on success.
    pub fn add_player(&mut self, name: &str, code: &str, api_type: &str) -> Result<u32, String> {
        let player_id = self.next_player_id;
        self.next_player_id += 1;

        let player = Player::new(player_id, name, api_type)?;

        // Set game state so top-level bot code can call API functions
        // (e.g. world_size(), get_koth_pos() during script initialization)
        let print_output = Rc::new(RefCell::new(Vec::new()));
        let gs = Rc::new(RefCell::new(lua_api::LuaGameState {
            world: self.world.clone(),
            creatures: self.creatures.clone(),
            game_time: self.game_time,
            player_id,
            player_scores: self.player_scores.clone(),
            player_names: self.player_names.clone(),
            king_player_id: self.king_player_id,
            print_output: print_output.clone(),
        }));
        lua_api::set_game_state(&player.lua, gs);

        let load_result = player.load_code(code);

        lua_api::clear_game_state(&player.lua);

        // Collect any print output from loading
        {
            let output = print_output.borrow();
            // Can't mutate player yet since we need to insert it first
            // We'll store output after insert
            drop(output);
        }

        load_result?;

        self.players.insert(player_id, player);

        // Collect load-time print output
        {
            let output = print_output.borrow();
            if let Some(p) = self.players.get_mut(&player_id) {
                p.output.extend(output.iter().cloned());
            }
        }

        self.player_scores.borrow_mut().insert(player_id, 0);
        self.player_names
            .borrow_mut()
            .insert(player_id, name.to_string());
        self.pending_events.insert(player_id, Vec::new());

        // Queue PLAYER_CREATED event
        self.pending_events
            .entry(player_id)
            .or_default()
            .push(GameEvent::PlayerCreated { player_id });

        Ok(player_id)
    }

    /// Remove a player and all their creatures.
    pub fn remove_player(&mut self, player_id: u32) {
        self.players.remove(&player_id);
        self.player_scores.borrow_mut().remove(&player_id);
        self.player_names.borrow_mut().remove(&player_id);
        self.pending_events.remove(&player_id);

        // Kill all creatures belonging to this player
        let to_kill: Vec<u32> = self
            .creatures
            .borrow()
            .iter()
            .filter(|(_, c)| c.player_id == player_id)
            .map(|(id, _)| *id)
            .collect();
        for id in to_kill {
            self.creatures.borrow_mut().remove(&id);
        }
    }

    /// Spawn a creature for a player at the given pixel position.
    /// Returns the creature ID or None if spawn fails.
    pub fn spawn_creature(
        &mut self,
        player_id: u32,
        x: i32,
        y: i32,
        creature_type: u8,
    ) -> Option<u32> {
        if !self.players.contains_key(&player_id) {
            return None;
        }

        let id = self.next_creature_id;
        self.next_creature_id += 1;

        let creature = Creature::new(id, x, y, creature_type, player_id);
        self.creatures.borrow_mut().insert(id, creature);

        // Update player creature count
        if let Some(player) = self.players.get_mut(&player_id) {
            player.num_creatures += 1;
        }

        // Queue spawn event for the owning player
        self.pending_events
            .entry(player_id)
            .or_default()
            .push(GameEvent::CreatureSpawned { id, parent: -1 });

        Some(id)
    }

    /// Spawn a creature as an offspring of another creature.
    fn spawn_offspring(&mut self, parent_id: u32, player_id: u32, x: i32, y: i32, creature_type: u8) -> Option<u32> {
        let id = self.next_creature_id;
        self.next_creature_id += 1;

        let creature = Creature::new(id, x, y, creature_type, player_id);
        self.creatures.borrow_mut().insert(id, creature);

        if let Some(player) = self.players.get_mut(&player_id) {
            player.num_creatures += 1;
        }

        self.pending_events
            .entry(player_id)
            .or_default()
            .push(GameEvent::CreatureSpawned {
                id,
                parent: parent_id as i32,
            });

        Some(id)
    }

    /// Kill a creature. Queues kill event.
    pub fn kill_creature(&mut self, creature_id: u32, killer_id: Option<u32>) {
        let creature = self.creatures.borrow().get(&creature_id).map(|c| (c.player_id, c.id));
        if let Some((player_id, _)) = creature {
            self.pending_events
                .entry(player_id)
                .or_default()
                .push(GameEvent::CreatureKilled {
                    id: creature_id,
                    killer: killer_id.map(|k| k as i32).unwrap_or(-1),
                });

            // Drop food on tile
            let creatures = self.creatures.borrow();
            if let Some(c) = creatures.get(&creature_id) {
                let tx = c.tile_x();
                let ty = c.tile_y();
                let food = c.food;
                drop(creatures);
                if food > 0 {
                    self.world.borrow_mut().add_food(tx, ty, food);
                }
            } else {
                drop(creatures);
            }

            self.creatures.borrow_mut().remove(&creature_id);

            if let Some(player) = self.players.get_mut(&player_id) {
                player.num_creatures -= 1;
            }
        }
    }

    /// Run one game tick.
    pub fn tick(&mut self) {
        let delta = self.tick_delta;

        // 1. Run each player's think (Lua execution)
        self.process_player_think();

        // 2. Process all creatures (movement, combat, aging, etc.)
        self.process_creatures(delta);

        // 3. King of the Hill scoring
        self.process_koth();

        // 4. Advance game time
        self.game_time += delta as i64;
    }

    /// Run each player's Lua think function.
    fn process_player_think(&mut self) {
        let player_ids: Vec<u32> = self.players.keys().copied().collect();

        for pid in player_ids {
            // Take pending events for this player
            let events = self.pending_events.get_mut(&pid).map(|e| std::mem::take(e)).unwrap_or_default();

            // Build the Lua events table
            let player = match self.players.get(&pid) {
                Some(p) => p,
                None => continue,
            };

            let print_output = Rc::new(RefCell::new(Vec::new()));

            let gs = Rc::new(RefCell::new(LuaGameState {
                world: self.world.clone(),
                creatures: self.creatures.clone(),
                game_time: self.game_time,
                player_id: pid,
                player_scores: self.player_scores.clone(),
                player_names: self.player_names.clone(),
                king_player_id: self.king_player_id,
                print_output: print_output.clone(),
            }));

            lua_api::set_game_state(&player.lua, gs);

            // Set instruction count limit to prevent infinite loops
            let _ = player.lua.set_hook(
                mlua::HookTriggers::new().every_nth_instruction(LUA_MAX_INSTRUCTIONS),
                |_lua, _debug| {
                    Err(mlua::Error::RuntimeError("lua vm cycles exceeded".into()))
                },
            );

            // Build the events table in Lua
            let result = (|| -> mlua::Result<()> {
                let lua = &player.lua;
                let events_table = lua.create_table()?;

                for (i, event) in events.iter().enumerate() {
                    let evt = lua.create_table()?;
                    match event {
                        GameEvent::CreatureSpawned { id, parent } => {
                            evt.set("type", 0i32)?; // CREATURE_SPAWNED
                            evt.set("id", *id)?;
                            evt.set("parent", *parent)?;
                        }
                        GameEvent::CreatureKilled { id, killer } => {
                            evt.set("type", 1i32)?; // CREATURE_KILLED
                            evt.set("id", *id)?;
                            evt.set("killer", *killer)?;
                        }
                        GameEvent::CreatureAttacked { id, attacker } => {
                            evt.set("type", 2i32)?; // CREATURE_ATTACKED
                            evt.set("id", *id)?;
                            evt.set("attacker", *attacker)?;
                        }
                        GameEvent::PlayerCreated { player_id: _ } => {
                            evt.set("type", 3i32)?; // PLAYER_CREATED
                        }
                    }
                    events_table.set(i + 1, evt)?; // Lua tables are 1-indexed
                }

                // Call player_think(events)
                let player_think: mlua::Function = lua.globals().get("player_think")?;
                let _: () = player_think.call(events_table)?;

                Ok(())
            })();

            // Remove instruction hook after execution
            player.lua.remove_hook();

            if let Err(e) = result {
                // Log the error but don't crash the game
                tracing::warn!(player_id = pid, "Lua error in player_think: {e}");
                let player = self.players.get_mut(&pid).unwrap();
                player
                    .output
                    .push(format!("Lua error: {e}"));
            }

            lua_api::clear_game_state(&self.players.get(&pid).unwrap().lua);

            // Collect print output
            if let Some(player) = self.players.get_mut(&pid) {
                let output = print_output.borrow();
                player.output.extend(output.iter().cloned());
            }
        }
    }

    /// Process all creatures for one tick: suicides, aging, state actions.
    fn process_creatures(&mut self, delta: i32) {
        // Collect creature IDs to process
        let creature_ids: Vec<u32> = self.creatures.borrow().keys().copied().collect();

        // Handle suicides first
        let suicides: Vec<u32> = creature_ids
            .iter()
            .filter(|id| {
                self.creatures
                    .borrow()
                    .get(id)
                    .map(|c| c.suicide)
                    .unwrap_or(false)
            })
            .copied()
            .collect();
        for id in suicides {
            self.kill_creature(id, Some(id));
        }

        // Re-collect after removals
        let creature_ids: Vec<u32> = self.creatures.borrow().keys().copied().collect();

        // Age all creatures, collect deaths
        let mut deaths = Vec::new();
        for &id in &creature_ids {
            let died = {
                let mut creatures = self.creatures.borrow_mut();
                if let Some(creature) = creatures.get_mut(&id) {
                    creature.do_age(delta)
                } else {
                    false
                }
            };
            if died {
                deaths.push(id);
            }
        }
        for id in deaths {
            self.kill_creature(id, None);
        }

        // Re-collect after deaths
        let creature_ids: Vec<u32> = self.creatures.borrow().keys().copied().collect();

        // Process state actions
        let mut new_spawns: Vec<(u32, u32, i32, i32, u8)> = Vec::new(); // (parent_id, player_id, x, y, type)
        let mut attack_events: Vec<(u32, u32, u32)> = Vec::new(); // (target_id, target_player, attacker_id)
        let mut kills_from_combat: Vec<(u32, u32)> = Vec::new(); // (creature_id, killer_id)

        for &id in &creature_ids {
            let mut creatures = self.creatures.borrow_mut();
            let creature = match creatures.get_mut(&id) {
                Some(c) => c,
                None => continue,
            };

            match creature.state {
                CREATURE_WALK => {
                    creature.do_walk(delta);
                }
                CREATURE_HEAL => {
                    let finished = creature.do_heal(delta);
                    if finished {
                        creature.set_state(CREATURE_IDLE);
                    }
                }
                CREATURE_EAT => {
                    let tx = creature.tile_x();
                    let ty = creature.tile_y();
                    let tile_food = self.world.borrow().get_food(tx, ty);
                    let (eaten, finished) = creature.do_eat(delta, tile_food);
                    if eaten > 0 {
                        self.world.borrow_mut().eat_food(tx, ty, eaten);
                    }
                    if finished {
                        creature.set_state(CREATURE_IDLE);
                    }
                }
                CREATURE_ATTACK => {
                    let target_id = match creature.target_id {
                        Some(tid) => tid,
                        None => {
                            creature.set_state(CREATURE_IDLE);
                            continue;
                        }
                    };

                    // Get target info
                    let attacker_type = creature.creature_type;
                    let attacker_x = creature.x;
                    let attacker_y = creature.y;
                    let attacker_id = creature.id;

                    let target = match creatures.get(&target_id) {
                        Some(t) => t,
                        None => {
                            let c = creatures.get_mut(&id).unwrap();
                            c.set_state(CREATURE_IDLE);
                            continue;
                        }
                    };

                    let target_type = target.creature_type;
                    let target_x = target.x;
                    let target_y = target.y;
                    let target_player = target.player_id;

                    let range = ATTACK_DISTANCE[attacker_type as usize][target_type as usize];
                    let damage_per_sec = HITPOINTS[attacker_type as usize][target_type as usize];

                    // Check range
                    let dx = (attacker_x - target_x) as i64;
                    let dy = (attacker_y - target_y) as i64;
                    let dist = ((dx * dx + dy * dy) as f64).sqrt() as i32;

                    if range == 0 || damage_per_sec == 0 || dist > range {
                        let c = creatures.get_mut(&id).unwrap();
                        c.set_state(CREATURE_IDLE);
                        continue;
                    }

                    // Apply damage
                    let damage = damage_per_sec * delta / 1000;
                    let target = creatures.get_mut(&target_id).unwrap();
                    target.health -= damage;

                    attack_events.push((target_id, target_player, attacker_id));

                    if target.health <= 0 {
                        kills_from_combat.push((target_id, attacker_id));
                        let c = creatures.get_mut(&id).unwrap();
                        c.set_state(CREATURE_IDLE);
                    }
                }
                CREATURE_CONVERT => {
                    let result = creature.do_convert(delta);
                    if result.is_some() {
                        creature.set_state(CREATURE_IDLE);
                    }
                }
                CREATURE_SPAWN => {
                    let player_id = creature.player_id;
                    let cx = creature.x;
                    let cy = creature.y;
                    let spawn_type_val = SPAWN_TYPE[creature.creature_type as usize];
                    let completed = creature.do_spawn(delta);
                    if completed && spawn_type_val >= 0 {
                        new_spawns.push((id, player_id, cx, cy, spawn_type_val as u8));
                        creature.set_state(CREATURE_IDLE);
                    }
                }
                CREATURE_FEED => {
                    let target_id = match creature.target_id {
                        Some(tid) => tid,
                        None => {
                            creature.set_state(CREATURE_IDLE);
                            continue;
                        }
                    };

                    let feeder_type = creature.creature_type;
                    let feeder_x = creature.x;
                    let feeder_y = creature.y;
                    let feeder_food = creature.food;

                    let feed_dist = FEED_DISTANCE[feeder_type as usize];
                    let feed_spd = FEED_SPEED[feeder_type as usize];

                    if feed_dist == 0 || feed_spd == 0 || feeder_food <= 0 {
                        creature.set_state(CREATURE_IDLE);
                        continue;
                    }

                    let target = match creatures.get(&target_id) {
                        Some(t) => t,
                        None => {
                            let c = creatures.get_mut(&id).unwrap();
                            c.set_state(CREATURE_IDLE);
                            continue;
                        }
                    };

                    let target_x = target.x;
                    let target_y = target.y;
                    let target_food = target.food;
                    let target_max_food = target.max_food();

                    let dx = (feeder_x - target_x) as i64;
                    let dy = (feeder_y - target_y) as i64;
                    let dist = ((dx * dx + dy * dy) as f64).sqrt() as i32;

                    if dist > feed_dist {
                        let c = creatures.get_mut(&id).unwrap();
                        c.set_state(CREATURE_IDLE);
                        continue;
                    }

                    let rate = feed_spd * delta / 1000;
                    let target_room = target_max_food - target_food;
                    let amount = rate.min(feeder_food).min(target_room);

                    if amount <= 0 {
                        let c = creatures.get_mut(&id).unwrap();
                        c.set_state(CREATURE_IDLE);
                        continue;
                    }

                    let feeder = creatures.get_mut(&id).unwrap();
                    feeder.food -= amount;
                    let feeder_food_left = feeder.food;

                    let target = creatures.get_mut(&target_id).unwrap();
                    target.food += amount;

                    if feeder_food_left <= 0 {
                        let c = creatures.get_mut(&id).unwrap();
                        c.set_state(CREATURE_IDLE);
                    }
                }
                CREATURE_IDLE | _ => {
                    // Idle: do nothing
                }
            }
        }

        // Queue attack events
        for (target_id, target_player, attacker_id) in attack_events {
            self.pending_events
                .entry(target_player)
                .or_default()
                .push(GameEvent::CreatureAttacked {
                    id: target_id,
                    attacker: attacker_id,
                });
        }

        // Process kills from combat
        for (creature_id, killer_id) in kills_from_combat {
            self.kill_creature(creature_id, Some(killer_id));
        }

        // Process new spawns
        for (parent_id, player_id, x, y, spawn_type) in new_spawns {
            self.spawn_offspring(parent_id, player_id, x, y, spawn_type);
        }
    }

    /// King of the Hill scoring: player with creature idle on koth tile gets points.
    fn process_koth(&mut self) {
        let world = self.world.borrow();
        let koth_x = world.koth_x;
        let koth_y = world.koth_y;
        drop(world);

        let creatures = self.creatures.borrow();
        let mut koth_player: Option<u32> = None;
        let mut multiple_players = false;

        for creature in creatures.values() {
            if creature.tile_x() == koth_x && creature.tile_y() == koth_y {
                match koth_player {
                    None => koth_player = Some(creature.player_id),
                    Some(pid) if pid != creature.player_id => {
                        multiple_players = true;
                        break;
                    }
                    _ => {}
                }
            }
        }

        if multiple_players {
            self.king_player_id = None;
        } else if let Some(pid) = koth_player {
            self.king_player_id = Some(pid);
            self.player_scores
                .borrow_mut()
                .entry(pid)
                .and_modify(|s| *s += 1)
                .or_insert(1);
            if let Some(player) = self.players.get_mut(&pid) {
                player.score += 1;
            }
        }
    }

    /// Check if only one player has creatures remaining (win condition).
    /// Returns Some(player_id) if exactly one player has creatures, None otherwise.
    /// Also returns None if no players have creatures at all.
    pub fn check_winner(&self) -> Option<u32> {
        let creatures = self.creatures.borrow();
        let mut player_with_creatures: Option<u32> = None;
        for c in creatures.values() {
            match player_with_creatures {
                None => player_with_creatures = Some(c.player_id),
                Some(pid) if pid != c.player_id => return None, // multiple players alive
                _ => {}
            }
        }
        // Only return a winner if there are actually creatures (and more than 1 player in the game)
        if player_with_creatures.is_some() && self.players.len() > 1 {
            player_with_creatures
        } else {
            None
        }
    }

    /// Create a snapshot of the game state for rendering.
    pub fn snapshot(&mut self) -> GameSnapshot {
        let creatures = self.creatures.borrow();
        let creature_snapshots: Vec<CreatureSnapshot> = creatures
            .values()
            .map(|c| CreatureSnapshot {
                id: c.id,
                x: c.x,
                y: c.y,
                creature_type: c.creature_type,
                health: c.health,
                max_health: c.max_health(),
                food: c.food,
                state: c.state,
                player_id: c.player_id,
                message: c.message.clone(),
                target_id: c.target_id,
            })
            .collect();

        let player_snapshots: Vec<PlayerSnapshot> = self
            .players
            .values_mut()
            .map(|p| PlayerSnapshot {
                id: p.id,
                name: p.name.clone(),
                score: p.score,
                color: p.color,
                num_creatures: p.num_creatures,
                output: std::mem::take(&mut p.output),
            })
            .collect();

        GameSnapshot {
            game_time: self.game_time,
            creatures: creature_snapshots,
            players: player_snapshots,
            king_player_id: self.king_player_id,
        }
    }

    /// Create a snapshot of the world (tiles) for initial WebSocket handshake.
    pub fn world_snapshot(&self) -> WorldSnapshot {
        let world = self.world.borrow();
        let mut tiles = Vec::with_capacity(world.width * world.height);
        for y in 0..world.height {
            for x in 0..world.width {
                tiles.push(TileSnapshot {
                    x,
                    y,
                    food: world.get_food(x, y),
                    tile_type: world.get_type(x, y),
                    gfx: world.get_gfx(x, y),
                });
            }
        }
        WorldSnapshot {
            width: world.width,
            height: world.height,
            koth_x: world.koth_x,
            koth_y: world.koth_y,
            tiles,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a simple open world for testing.
    fn make_test_world() -> World {
        let mut w = World::new(10, 10);
        for y in 1..9 {
            for x in 1..9 {
                w.set_type(x, y, TILE_PLAIN);
            }
        }
        // Add some food
        w.add_food(3, 3, 5000);
        w
    }

    #[test]
    fn test_new_game() {
        let world = make_test_world();
        let game = Game::new(world);
        assert_eq!(game.game_time, 0);
        assert!(game.players.is_empty());
        assert!(game.creatures.borrow().is_empty());
    }

    #[test]
    fn test_add_player() {
        let world = make_test_world();
        let mut game = Game::new(world);

        let pid = game.add_player("TestBot", "", "oo");
        assert!(pid.is_ok());
        let pid = pid.unwrap();
        assert_eq!(pid, 1);
        assert!(game.players.contains_key(&pid));
        assert_eq!(game.players.get(&pid).unwrap().name, "TestBot");
    }

    #[test]
    fn test_spawn_creature() {
        let world = make_test_world();
        let mut game = Game::new(world);
        let pid = game.add_player("TestBot", "", "oo").unwrap();

        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        let cid = game.spawn_creature(pid, cx, cy, CREATURE_SMALL);
        assert!(cid.is_some());
        let cid = cid.unwrap();

        let creatures = game.creatures.borrow();
        let creature = creatures.get(&cid).unwrap();
        assert_eq!(creature.x, cx);
        assert_eq!(creature.y, cy);
        assert_eq!(creature.creature_type, CREATURE_SMALL);
        assert_eq!(creature.player_id, pid);
    }

    #[test]
    fn test_basic_tick() {
        let world = make_test_world();
        let mut game = Game::new(world);
        let pid = game.add_player("TestBot", "", "oo").unwrap();

        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        let cid = game.spawn_creature(pid, cx, cy, CREATURE_SMALL).unwrap();

        let health_before = game.creatures.borrow().get(&cid).unwrap().health;

        // Run one tick
        game.tick();

        // After tick, creature should have aged
        let health_after = game.creatures.borrow().get(&cid).unwrap().health;
        assert!(health_after < health_before);
        assert_eq!(game.game_time, 100);
    }

    #[test]
    fn test_creature_eating() {
        let world = make_test_world();
        let mut game = Game::new(world);
        // Bot code that sets eating state
        let code = r#"
            function Creature:main()
                self:begin_eating()
                self:wait_for_next_round()
            end
        "#;
        let pid = game.add_player("EatBot", code, "oo").unwrap();

        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        let cid = game.spawn_creature(pid, cx, cy, CREATURE_SMALL).unwrap();

        // Run several ticks (first tick processes spawn event and starts coroutine,
        // subsequent ticks resume coroutine which sets eating)
        for _ in 0..5 {
            game.tick();
        }

        let creatures = game.creatures.borrow();
        let creature = creatures.get(&cid).unwrap();
        // Creature should have eaten some food
        assert!(creature.food > 0, "Creature food should be > 0 after eating, got {}", creature.food);
    }

    #[test]
    fn test_creature_walking() {
        let world = make_test_world();
        let mut game = Game::new(world);

        let target_x = World::tile_center(6);
        let target_y = World::tile_center(3);
        let code = format!(
            r#"
            function Creature:main()
                self:set_path({target_x}, {target_y})
                self:begin_walk_path()
                while self:is_walking() do
                    self:wait_for_next_round()
                end
            end
            "#
        );
        let pid = game.add_player("WalkBot", &code, "oo").unwrap();

        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        let cid = game.spawn_creature(pid, cx, cy, CREATURE_SMALL).unwrap();

        // Run several ticks
        for _ in 0..20 {
            game.tick();
        }

        let creatures = game.creatures.borrow();
        let creature = creatures.get(&cid).unwrap();
        // Creature should have moved closer to target
        let start_dist = ((cx - target_x).abs() + (cy - target_y).abs()) as i32;
        let end_dist = ((creature.x - target_x).abs() + (creature.y - target_y).abs()) as i32;
        assert!(end_dist < start_dist, "Creature should have moved closer to target");
    }

    #[test]
    fn test_combat() {
        let world = make_test_world();
        let mut game = Game::new(world);

        // Player 1: big creature that attacks
        let pid1 = game.add_player("Attacker", "", "oo").unwrap();
        // Player 2: small creature as target
        let pid2 = game.add_player("Target", "", "oo").unwrap();

        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        let attacker_id = game.spawn_creature(pid1, cx, cy, CREATURE_BIG).unwrap();
        let target_id = game.spawn_creature(pid2, cx + 100, cy, CREATURE_SMALL).unwrap();

        // Manually set the attacker to attack the target
        {
            let mut creatures = game.creatures.borrow_mut();
            let attacker = creatures.get_mut(&attacker_id).unwrap();
            attacker.set_target(target_id);
            attacker.set_state(CREATURE_ATTACK);
        }

        let target_health_before = game.creatures.borrow().get(&target_id).unwrap().health;

        // Run one tick (skip player_think by not having bot code do anything)
        game.tick();

        // Target should have taken damage
        let creatures = game.creatures.borrow();
        if let Some(target) = creatures.get(&target_id) {
            assert!(target.health < target_health_before, "Target should have taken damage");
        }
        // (target might also be dead if damage is high enough)
    }

    #[test]
    fn test_koth() {
        let world = make_test_world();
        let mut game = Game::new(world);
        let pid = game.add_player("KothBot", "", "oo").unwrap();

        // Spawn creature on the koth tile
        let koth_x = World::tile_center(game.world.borrow().koth_x);
        let koth_y = World::tile_center(game.world.borrow().koth_y);
        game.spawn_creature(pid, koth_x, koth_y, CREATURE_SMALL);

        // Run a tick
        game.tick();

        // Player should have scored
        let score = *game.player_scores.borrow().get(&pid).unwrap_or(&0);
        assert!(score > 0, "Player should have scored from koth, got {score}");
        assert_eq!(game.king_player_id, Some(pid));
    }

    #[test]
    fn test_remove_player() {
        let world = make_test_world();
        let mut game = Game::new(world);
        let pid = game.add_player("TestBot", "", "oo").unwrap();
        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        game.spawn_creature(pid, cx, cy, CREATURE_SMALL);

        assert_eq!(game.creatures.borrow().len(), 1);
        game.remove_player(pid);
        assert!(!game.players.contains_key(&pid));
        assert_eq!(game.creatures.borrow().len(), 0);
    }

    #[test]
    fn test_snapshot() {
        let world = make_test_world();
        let mut game = Game::new(world);
        let pid = game.add_player("TestBot", "", "oo").unwrap();
        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        game.spawn_creature(pid, cx, cy, CREATURE_SMALL);

        let snap = game.snapshot();
        assert_eq!(snap.game_time, 0);
        assert_eq!(snap.creatures.len(), 1);
        assert_eq!(snap.creatures[0].x, cx);
    }

    #[test]
    fn test_check_winner_no_winner() {
        let world = make_test_world();
        let mut game = Game::new(world);
        let pid1 = game.add_player("Bot1", "", "oo").unwrap();
        let pid2 = game.add_player("Bot2", "", "oo").unwrap();

        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        game.spawn_creature(pid1, cx, cy, CREATURE_SMALL);
        game.spawn_creature(pid2, cx + 256, cy, CREATURE_SMALL);

        // Both players have creatures — no winner
        assert_eq!(game.check_winner(), None);
    }

    #[test]
    fn test_check_winner_one_player_left() {
        let world = make_test_world();
        let mut game = Game::new(world);
        let pid1 = game.add_player("Bot1", "", "oo").unwrap();
        let _pid2 = game.add_player("Bot2", "", "oo").unwrap();

        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        game.spawn_creature(pid1, cx, cy, CREATURE_SMALL);
        // pid2 has no creatures

        assert_eq!(game.check_winner(), Some(pid1));
    }

    #[test]
    fn test_check_winner_single_player_no_win() {
        let world = make_test_world();
        let mut game = Game::new(world);
        let pid1 = game.add_player("Bot1", "", "oo").unwrap();

        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        game.spawn_creature(pid1, cx, cy, CREATURE_SMALL);

        // Only 1 player in game — no win condition
        assert_eq!(game.check_winner(), None);
    }

    #[test]
    fn test_instruction_limit_infinite_loop() {
        let world = make_test_world();
        let mut game = Game::new(world);

        // Bot with an infinite loop in main()
        let code = r#"
            function Creature:main()
                while true do end
            end
        "#;
        let pid = game.add_player("InfiniteBot", code, "oo").unwrap();
        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        game.spawn_creature(pid, cx, cy, CREATURE_SMALL);

        // This should NOT hang — the instruction limit should catch it
        game.tick();
        assert_eq!(game.game_time, 100);

        // Error should appear in player output
        let snap = game.snapshot();
        let player = snap.players.iter().find(|p| p.id == pid).unwrap();
        let has_error = player.output.iter().any(|line| line.contains("cycles exceeded"));
        assert!(has_error, "Expected 'cycles exceeded' error in output, got: {:?}", player.output);
    }

    #[test]
    fn test_instruction_limit_loop_in_onspawned() {
        let world = make_test_world();
        let mut game = Game::new(world);

        // Bot with an infinite loop at load time / onSpawned
        let code = r#"
            function Creature:onSpawned()
                while true do end
            end
            function Creature:main()
                self:wait_for_next_round()
            end
        "#;
        let pid = game.add_player("SpawnLoopBot", code, "oo").unwrap();
        let cx = World::tile_center(3);
        let cy = World::tile_center(3);
        game.spawn_creature(pid, cx, cy, CREATURE_SMALL);

        // Should not hang
        game.tick();
        assert_eq!(game.game_time, 100);
    }
}
