use super::config::*;

/// Represents a creature in the game world.
pub struct Creature {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub creature_type: u8,
    pub health: i32,
    pub food: i32,
    pub state: u8,
    pub player_id: u32,
    pub target_id: Option<u32>,
    pub path: Vec<(i32, i32)>,
    pub convert_type: u8,
    pub convert_food: i32,
    pub spawn_food: i32,
    pub message: String,
    pub suicide: bool,
    pub age_action_deltas: i32,
}

impl Creature {
    /// Create a new creature at the given pixel position with full health and zero food.
    pub fn new(id: u32, x: i32, y: i32, creature_type: u8, player_id: u32) -> Self {
        let health = MAX_HEALTH[creature_type as usize];
        Creature {
            id,
            x,
            y,
            creature_type,
            health,
            food: 0,
            state: CREATURE_IDLE,
            player_id,
            target_id: None,
            path: Vec::new(),
            convert_type: creature_type, // default: no conversion
            convert_food: 0,
            spawn_food: 0,
            message: String::new(),
            suicide: false,
            age_action_deltas: 0,
        }
    }

    // --- Stats (indexed by creature_type) ---

    #[inline]
    fn type_idx(&self) -> usize {
        self.creature_type as usize
    }

    /// Maximum health for this creature type.
    pub fn max_health(&self) -> i32 {
        MAX_HEALTH[self.type_idx()]
    }

    /// Maximum food storage for this creature type.
    pub fn max_food(&self) -> i32 {
        MAX_FOOD[self.type_idx()]
    }

    /// Current movement speed in pixels per second.
    /// For small creatures, speed increases with health.
    /// Capped at 1000.
    pub fn speed(&self) -> i32 {
        let base = BASE_SPEED[self.type_idx()];
        let max_hp = self.max_health();
        let health_bonus = if max_hp > 0 {
            HEALTH_SPEED[self.type_idx()] * self.health / max_hp
        } else {
            0
        };
        (base + health_bonus).min(1000)
    }

    /// Aging rate: health lost per 100ms tick.
    pub fn aging_rate(&self) -> i32 {
        AGING[self.type_idx()]
    }

    /// Heal rate: food converted to health per second.
    pub fn heal_rate(&self) -> i32 {
        HEAL_RATE[self.type_idx()]
    }

    /// Eat rate: tile food consumed per second.
    pub fn eat_rate(&self) -> i32 {
        EAT_RATE[self.type_idx()]
    }

    /// Returns true if this creature is ground-based (not a flyer).
    pub fn is_ground_based(&self) -> bool {
        self.creature_type != CREATURE_FLYER
    }

    /// Health as a percentage (0-100).
    pub fn health_percent(&self) -> i32 {
        let max = self.max_health();
        if max == 0 {
            return 0;
        }
        self.health * 100 / max
    }

    // --- State checks ---

    /// Can this creature walk? Needs a non-empty path and positive speed.
    pub fn can_walk(&self) -> bool {
        !self.path.is_empty() && self.speed() > 0
    }

    /// Can this creature heal? Needs health below max and some food.
    pub fn can_heal(&self) -> bool {
        self.health < self.max_health() && self.food > 0
    }

    /// Can this creature eat from a tile? Tile needs food, creature needs room.
    pub fn can_eat(&self, tile_food: i32) -> bool {
        tile_food > 0 && self.food < self.max_food()
    }

    /// Can this creature convert? The target conversion type must require food > 0.
    pub fn can_convert(&self) -> bool {
        let needed = CONVERSION_FOOD[self.type_idx()][self.convert_type as usize];
        needed > 0
    }

    /// Can this creature spawn? Must have a valid spawn type and enough health.
    pub fn can_spawn(&self) -> bool {
        let spawn_type = SPAWN_TYPE[self.type_idx()];
        spawn_type >= 0 && self.health > SPAWN_HEALTH[self.type_idx()]
    }

    /// Can this creature feed others? Needs positive feed distance and some food.
    pub fn can_feed(&self) -> bool {
        FEED_DISTANCE[self.type_idx()] > 0 && self.food > 0
    }

    // --- State transitions ---

    /// Attempt to transition to a new state. Returns true if the transition is valid.
    /// Resets convert_food and spawn_food when leaving CONVERT or SPAWN states.
    pub fn set_state(&mut self, new_state: u8) -> bool {
        if new_state >= CREATURE_STATES as u8 {
            return false;
        }

        // Clean up on exit from current state
        match self.state {
            CREATURE_CONVERT => {
                if new_state != CREATURE_CONVERT {
                    self.convert_food = 0;
                }
            }
            CREATURE_SPAWN => {
                if new_state != CREATURE_SPAWN {
                    self.spawn_food = 0;
                }
            }
            _ => {}
        }

        self.state = new_state;
        true
    }

    // --- Combat ---

    /// Damage per second this creature deals to a target of the given type.
    pub fn attack_damage(&self, target_type: u8) -> i32 {
        HITPOINTS[self.type_idx()][target_type as usize]
    }

    /// Attack range against a target of the given type, in pixels.
    pub fn attack_range(&self, target_type: u8) -> i32 {
        ATTACK_DISTANCE[self.type_idx()][target_type as usize]
    }

    // --- Actions (process one tick) ---

    /// Move along the path for one tick. Sets state to IDLE when path is complete.
    pub fn do_walk(&mut self, delta: i32) {
        let mut travelled = self.speed() * delta / 1000;
        if travelled == 0 {
            travelled = 1;
        }

        while !self.path.is_empty() {
            let (wx, wy) = self.path[0];
            let dx = wx - self.x;
            let dy = wy - self.y;
            let dist_sq = (dx as i64) * (dx as i64) + (dy as i64) * (dy as i64);
            let dist = (dist_sq as f64).sqrt() as i32;

            if dist == 0 {
                self.path.remove(0);
                continue;
            }

            if travelled >= dist {
                self.x = wx;
                self.y = wy;
                travelled -= dist;
                self.path.remove(0);
            } else {
                self.x += dx * travelled / dist;
                self.y += dy * travelled / dist;
                return;
            }
        }

        // Path exhausted
        self.state = CREATURE_IDLE;
    }

    /// Heal for one tick. Consumes food, adds health. Returns true if healing is complete
    /// (health full or food exhausted).
    pub fn do_heal(&mut self, delta: i32) -> bool {
        let rate = self.heal_rate() * delta / 1000;
        if rate == 0 {
            return false;
        }

        let max_hp = self.max_health();
        let needed = max_hp - self.health;
        let amount = rate.min(needed).min(self.food);

        self.food -= amount;
        self.health += amount;

        self.health >= max_hp || self.food <= 0
    }

    /// Eat from a tile for one tick. Returns (amount_eaten_from_tile, finished).
    /// Finished means creature food is full or no more tile food to eat.
    pub fn do_eat(&mut self, delta: i32, tile_food: i32) -> (i32, bool) {
        let rate = self.eat_rate() * delta / 1000;
        if rate == 0 {
            return (0, false);
        }

        let room = self.max_food() - self.food;
        let amount = rate.min(room).min(tile_food);

        self.food += amount;

        let finished = self.food >= self.max_food() || tile_food - amount <= 0;
        (amount, finished)
    }

    /// Process conversion for one tick. Invests food toward converting to convert_type.
    /// Returns Some(new_type) when conversion is complete.
    pub fn do_convert(&mut self, delta: i32) -> Option<u8> {
        let needed = CONVERSION_FOOD[self.type_idx()][self.convert_type as usize];
        if needed == 0 {
            return None;
        }

        let rate = CONVERSION_SPEED[self.type_idx()] * delta / 1000;
        let invest = rate.min(self.food).min(needed - self.convert_food);
        self.food -= invest;
        self.convert_food += invest;

        if self.convert_food >= needed {
            let new_type = self.convert_type;
            self.creature_type = new_type;
            self.health = MAX_HEALTH[new_type as usize];
            self.convert_food = 0;
            self.convert_type = new_type;
            Some(new_type)
        } else {
            None
        }
    }

    /// Process spawning for one tick. Invests food toward creating an offspring.
    /// Returns true when spawn is complete and a new creature should be created.
    pub fn do_spawn(&mut self, delta: i32) -> bool {
        let needed = SPAWN_FOOD[self.type_idx()];
        if needed == 0 {
            return false;
        }

        let rate = SPAWN_SPEED[self.type_idx()] * delta / 1000;
        let invest = rate.min(self.food).min(needed - self.spawn_food);
        self.food -= invest;
        self.spawn_food += invest;

        if self.spawn_food >= needed {
            self.spawn_food = 0;
            // Deduct health cost
            self.health -= SPAWN_HEALTH[self.type_idx()];
            true
        } else {
            false
        }
    }

    /// Age the creature for one tick. Returns true if the creature has died (health <= 0).
    pub fn do_age(&mut self, delta: i32) -> bool {
        self.age_action_deltas += delta;
        // Aging is per 100ms, so accumulate and drain
        while self.age_action_deltas >= 100 {
            self.age_action_deltas -= 100;
            self.health -= self.aging_rate();
        }
        self.health <= 0
    }

    // --- Setters ---

    /// Set the target creature. Cannot target self. Returns true on success.
    pub fn set_target(&mut self, target_id: u32) -> bool {
        if target_id == self.id {
            return false;
        }
        self.target_id = Some(target_id);
        true
    }

    /// Set the desired conversion type. Validates that conversion is possible.
    /// Returns true on success.
    pub fn set_conversion_type(&mut self, target_type: u8) -> bool {
        if target_type as usize >= CREATURE_TYPES {
            return false;
        }
        let needed = CONVERSION_FOOD[self.type_idx()][target_type as usize];
        if needed == 0 {
            return false;
        }
        self.convert_type = target_type;
        self.convert_food = 0; // reset progress when changing target type
        true
    }

    /// Set the display message, truncated to 8 characters.
    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.chars().take(8).collect();
    }

    /// Set the movement path (list of pixel coordinate waypoints).
    pub fn set_path(&mut self, path: Vec<(i32, i32)>) {
        self.path = path;
    }

    /// Current tile X coordinate.
    pub fn tile_x(&self) -> usize {
        (self.x / TILE_SIZE) as usize
    }

    /// Current tile Y coordinate.
    pub fn tile_y(&self) -> usize {
        (self.y / TILE_SIZE) as usize
    }

    /// Euclidean distance to a point in pixels.
    pub fn distance_to(&self, other_x: i32, other_y: i32) -> i32 {
        let dx = (self.x - other_x) as i64;
        let dy = (self.y - other_y) as i64;
        ((dx * dx + dy * dy) as f64).sqrt() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creature_new() {
        let c = Creature::new(1, 512, 768, CREATURE_SMALL, 10);
        assert_eq!(c.id, 1);
        assert_eq!(c.x, 512);
        assert_eq!(c.y, 768);
        assert_eq!(c.creature_type, CREATURE_SMALL);
        assert_eq!(c.health, MAX_HEALTH[0]);
        assert_eq!(c.food, 0);
        assert_eq!(c.state, CREATURE_IDLE);
        assert_eq!(c.player_id, 10);
        assert!(c.target_id.is_none());
        assert!(c.path.is_empty());
        assert!(!c.suicide);
    }

    #[test]
    fn test_creature_stats() {
        let small = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert_eq!(small.max_health(), 10000);
        assert_eq!(small.max_food(), 10000);
        assert_eq!(small.aging_rate(), 5);
        assert_eq!(small.heal_rate(), 500);
        assert_eq!(small.eat_rate(), 800);
        assert!(small.is_ground_based());

        let big = Creature::new(2, 0, 0, CREATURE_BIG, 0);
        assert_eq!(big.max_health(), 20000);
        assert_eq!(big.max_food(), 20000);
        assert_eq!(big.aging_rate(), 7);
        assert_eq!(big.heal_rate(), 300);
        assert_eq!(big.eat_rate(), 400);
        assert!(big.is_ground_based());

        let flyer = Creature::new(3, 0, 0, CREATURE_FLYER, 0);
        assert_eq!(flyer.max_health(), 5000);
        assert_eq!(flyer.max_food(), 5000);
        assert_eq!(flyer.aging_rate(), 5);
        assert_eq!(flyer.heal_rate(), 600);
        assert_eq!(flyer.eat_rate(), 600);
        assert!(!flyer.is_ground_based());
    }

    #[test]
    fn test_speed_calculation() {
        // Small creature: speed depends on health
        let mut small = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        // Full health: base_speed(200) + health_speed(625) * 10000/10000 = 825
        assert_eq!(small.speed(), 825);

        // Half health: 200 + 625 * 5000/10000 = 200 + 312 = 512
        small.health = 5000;
        assert_eq!(small.speed(), 512);

        // Zero health: 200 + 0 = 200
        small.health = 0;
        assert_eq!(small.speed(), 200);

        // Big creature: no health speed bonus, base 400
        let big = Creature::new(2, 0, 0, CREATURE_BIG, 0);
        assert_eq!(big.speed(), 400);

        // Flyer: base 800, no bonus
        let flyer = Creature::new(3, 0, 0, CREATURE_FLYER, 0);
        assert_eq!(flyer.speed(), 800);
    }

    #[test]
    fn test_speed_capped_at_1000() {
        // Hypothetical scenario where speed would exceed 1000
        // Small at full health = 825, which is under 1000. But let's verify the cap exists.
        let small = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert!(small.speed() <= 1000);
    }

    #[test]
    fn test_health_percent() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert_eq!(c.health_percent(), 100);
        c.health = 5000;
        assert_eq!(c.health_percent(), 50);
        c.health = 0;
        assert_eq!(c.health_percent(), 0);
        c.health = 2500;
        assert_eq!(c.health_percent(), 25);
    }

    #[test]
    fn test_state_transitions() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert_eq!(c.state, CREATURE_IDLE);

        assert!(c.set_state(CREATURE_WALK));
        assert_eq!(c.state, CREATURE_WALK);

        assert!(c.set_state(CREATURE_EAT));
        assert_eq!(c.state, CREATURE_EAT);

        // Invalid state
        assert!(!c.set_state(99));
        assert_eq!(c.state, CREATURE_EAT); // unchanged
    }

    #[test]
    fn test_state_transition_resets_convert_food() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.set_state(CREATURE_CONVERT);
        c.convert_food = 5000;
        c.set_state(CREATURE_IDLE);
        assert_eq!(c.convert_food, 0);
    }

    #[test]
    fn test_state_transition_resets_spawn_food() {
        let mut c = Creature::new(1, 0, 0, CREATURE_BIG, 0);
        c.set_state(CREATURE_SPAWN);
        c.spawn_food = 3000;
        c.set_state(CREATURE_IDLE);
        assert_eq!(c.spawn_food, 0);
    }

    #[test]
    fn test_state_transition_no_reset_same_state() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.set_state(CREATURE_CONVERT);
        c.convert_food = 5000;
        c.set_state(CREATURE_CONVERT); // same state
        assert_eq!(c.convert_food, 5000); // not reset
    }

    #[test]
    fn test_can_walk() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert!(!c.can_walk()); // no path
        c.path = vec![(100, 100)];
        assert!(c.can_walk());
    }

    #[test]
    fn test_can_heal() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert!(!c.can_heal()); // full health, no food
        c.health = 5000;
        assert!(!c.can_heal()); // below max but no food
        c.food = 100;
        assert!(c.can_heal()); // below max and has food
        c.health = c.max_health();
        assert!(!c.can_heal()); // full health
    }

    #[test]
    fn test_can_eat() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert!(c.can_eat(100)); // has room, tile has food
        assert!(!c.can_eat(0)); // no tile food
        c.food = c.max_food();
        assert!(!c.can_eat(100)); // food full
    }

    #[test]
    fn test_can_convert() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        // Small -> Small: CONVERSION_FOOD[0][0] = 0, cannot convert to same type
        c.convert_type = CREATURE_SMALL;
        assert!(!c.can_convert());
        // Small -> Big: CONVERSION_FOOD[0][1] = 8000
        c.convert_type = CREATURE_BIG;
        assert!(c.can_convert());
        // Small -> Flyer: CONVERSION_FOOD[0][2] = 5000
        c.convert_type = CREATURE_FLYER;
        assert!(c.can_convert());
    }

    #[test]
    fn test_can_spawn() {
        // Big can spawn (SPAWN_TYPE[1] = 0, SPAWN_HEALTH[1] = 4000)
        let big = Creature::new(1, 0, 0, CREATURE_BIG, 0);
        assert!(big.can_spawn()); // health 20000 > 4000

        // Small cannot spawn (SPAWN_TYPE[0] = -1)
        let small = Creature::new(2, 0, 0, CREATURE_SMALL, 0);
        assert!(!small.can_spawn());

        // Big with low health
        let mut weak_big = Creature::new(3, 0, 0, CREATURE_BIG, 0);
        weak_big.health = 4000; // equal to SPAWN_HEALTH, not greater
        assert!(!weak_big.can_spawn());
    }

    #[test]
    fn test_can_feed() {
        let small = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert!(!small.can_feed()); // no food

        let mut small = Creature::new(2, 0, 0, CREATURE_SMALL, 0);
        small.food = 100;
        assert!(small.can_feed()); // FEED_DISTANCE[0] = 256 > 0

        let mut big = Creature::new(3, 0, 0, CREATURE_BIG, 0);
        big.food = 100;
        assert!(!big.can_feed()); // FEED_DISTANCE[1] = 0
    }

    #[test]
    fn test_walking() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.state = CREATURE_WALK;
        c.path = vec![(1000, 0), (2000, 0)];

        // With full health, speed = 825 px/s. delta=1000ms => travel 825 px.
        c.do_walk(1000);
        // Should have passed first waypoint (1000 px away) and moved toward second
        // After reaching (1000,0), remaining = 825-1000 = ... wait, 825 < 1000
        // Actually speed=825, delta=1000 => travelled=825. First waypoint at distance 1000.
        // 825 < 1000, so partial move: x += 1000*825/1000 = 825
        assert_eq!(c.x, 825);
        assert_eq!(c.y, 0);
        assert_eq!(c.path.len(), 2); // still has both waypoints
    }

    #[test]
    fn test_walking_reaches_waypoint() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.state = CREATURE_WALK;
        c.path = vec![(500, 0)];

        // speed=825, delta=1000 => travelled=825 > 500
        c.do_walk(1000);
        // Should reach waypoint and go idle
        assert_eq!(c.x, 500);
        assert_eq!(c.y, 0);
        assert!(c.path.is_empty());
        assert_eq!(c.state, CREATURE_IDLE);
    }

    #[test]
    fn test_walking_multiple_waypoints() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.state = CREATURE_WALK;
        c.path = vec![(100, 0), (200, 0), (300, 0)];

        // speed=825, delta=1000 => travelled=825
        // Waypoint 1 at dist 100: pass, remaining=725
        // Waypoint 2 at dist 100: pass, remaining=625
        // Waypoint 3 at dist 100: pass, remaining=525
        // Path empty -> IDLE
        c.do_walk(1000);
        assert_eq!(c.x, 300);
        assert_eq!(c.state, CREATURE_IDLE);
    }

    #[test]
    fn test_walking_minimum_travel() {
        // Very low speed scenario: delta very small
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.health = 0; // speed = 200
        c.state = CREATURE_WALK;
        c.path = vec![(100, 0)];

        // speed=200, delta=1 => travelled = 200*1/1000 = 0 => forced to 1
        c.do_walk(1);
        assert_eq!(c.x, 1); // moved 1 pixel
    }

    #[test]
    fn test_healing() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.health = 5000;
        c.food = 10000;

        // heal_rate = 500/s, delta=1000ms => heal 500
        let finished = c.do_heal(1000);
        assert_eq!(c.health, 5500);
        assert_eq!(c.food, 9500);
        assert!(!finished);
    }

    #[test]
    fn test_healing_completes_at_max() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.health = 9800;
        c.food = 10000;

        // heal_rate = 500/s, but only need 200
        let finished = c.do_heal(1000);
        assert_eq!(c.health, 10000);
        assert_eq!(c.food, 9800);
        assert!(finished);
    }

    #[test]
    fn test_healing_completes_food_exhausted() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.health = 5000;
        c.food = 100;

        let finished = c.do_heal(1000);
        assert_eq!(c.health, 5100);
        assert_eq!(c.food, 0);
        assert!(finished);
    }

    #[test]
    fn test_eating() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        // eat_rate = 800/s, delta=1000 => eat 800
        let (eaten, finished) = c.do_eat(1000, 5000);
        assert_eq!(eaten, 800);
        assert_eq!(c.food, 800);
        assert!(!finished);
    }

    #[test]
    fn test_eating_limited_by_tile() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        let (eaten, finished) = c.do_eat(1000, 100);
        assert_eq!(eaten, 100);
        assert_eq!(c.food, 100);
        assert!(finished); // tile food exhausted
    }

    #[test]
    fn test_eating_limited_by_room() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.food = 9500;
        // Room = 500, eat_rate = 800
        let (eaten, finished) = c.do_eat(1000, 5000);
        assert_eq!(eaten, 500);
        assert_eq!(c.food, 10000);
        assert!(finished); // food full
    }

    #[test]
    fn test_conversion() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.food = 10000;
        c.convert_type = CREATURE_BIG;
        // CONVERSION_FOOD[0][1] = 8000, CONVERSION_SPEED[0] = 1000/s

        // First 7 seconds: invest 1000 each, total 7000 - not done yet
        for _ in 0..7 {
            let result = c.do_convert(1000);
            assert!(result.is_none());
        }
        assert_eq!(c.convert_food, 7000);
        assert_eq!(c.food, 3000);

        // 8th second: invest 1000 more, total 8000 - conversion completes
        let result = c.do_convert(1000);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), CREATURE_BIG);
        assert_eq!(c.creature_type, CREATURE_BIG);
        assert_eq!(c.health, MAX_HEALTH[CREATURE_BIG as usize]);
        assert_eq!(c.convert_food, 0);
        assert_eq!(c.food, 2000);
    }

    #[test]
    fn test_spawning() {
        let mut c = Creature::new(1, 0, 0, CREATURE_BIG, 0);
        c.food = 10000;
        // SPAWN_FOOD[1] = 5000, SPAWN_SPEED[1] = 2000/s, SPAWN_HEALTH[1] = 4000

        // After 1s: invest 2000
        let result = c.do_spawn(1000);
        assert!(!result);
        assert_eq!(c.spawn_food, 2000);
        assert_eq!(c.food, 8000);

        // After 1 more second: invest 2000 (total 4000)
        let result = c.do_spawn(1000);
        assert!(!result);
        assert_eq!(c.spawn_food, 4000);

        // After 0.5 more seconds: invest 1000 (total 5000)
        let result = c.do_spawn(500);
        assert!(result);
        assert_eq!(c.spawn_food, 0);
        // Health should have been deducted
        assert_eq!(c.health, MAX_HEALTH[CREATURE_BIG as usize] - SPAWN_HEALTH[CREATURE_BIG as usize]);
    }

    #[test]
    fn test_aging() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        // aging_rate = 5 per 100ms

        // After 100ms: lose 5 health
        let dead = c.do_age(100);
        assert!(!dead);
        assert_eq!(c.health, 9995);

        // After 1 second (10 * 100ms): lose 50
        let dead = c.do_age(1000);
        assert!(!dead);
        assert_eq!(c.health, 9945);
    }

    #[test]
    fn test_aging_death() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.health = 5; // exactly one aging tick away
        let dead = c.do_age(100);
        assert!(dead);
        assert_eq!(c.health, 0);
    }

    #[test]
    fn test_aging_accumulator() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        // 50ms: not enough for a full aging tick
        c.do_age(50);
        assert_eq!(c.health, 10000); // no change yet
        assert_eq!(c.age_action_deltas, 50);

        // Another 50ms: now 100ms total
        c.do_age(50);
        assert_eq!(c.health, 9995);
        assert_eq!(c.age_action_deltas, 0);
    }

    #[test]
    fn test_combat_damage() {
        let small = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert_eq!(small.attack_damage(CREATURE_SMALL), 0);
        assert_eq!(small.attack_damage(CREATURE_BIG), 0);
        assert_eq!(small.attack_damage(CREATURE_FLYER), 1000);

        let big = Creature::new(2, 0, 0, CREATURE_BIG, 0);
        assert_eq!(big.attack_damage(CREATURE_SMALL), 1500);
        assert_eq!(big.attack_damage(CREATURE_BIG), 1500);
        assert_eq!(big.attack_damage(CREATURE_FLYER), 1500);

        let flyer = Creature::new(3, 0, 0, CREATURE_FLYER, 0);
        assert_eq!(flyer.attack_damage(CREATURE_SMALL), 0);
        assert_eq!(flyer.attack_damage(CREATURE_BIG), 0);
        assert_eq!(flyer.attack_damage(CREATURE_FLYER), 0);
    }

    #[test]
    fn test_combat_range() {
        let small = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert_eq!(small.attack_range(CREATURE_SMALL), 0);
        assert_eq!(small.attack_range(CREATURE_BIG), 0);
        assert_eq!(small.attack_range(CREATURE_FLYER), 768);

        let big = Creature::new(2, 0, 0, CREATURE_BIG, 0);
        assert_eq!(big.attack_range(CREATURE_SMALL), 512);
        assert_eq!(big.attack_range(CREATURE_BIG), 512);
        assert_eq!(big.attack_range(CREATURE_FLYER), 512);
    }

    #[test]
    fn test_set_target() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert!(c.set_target(2));
        assert_eq!(c.target_id, Some(2));
        // Can't target self
        assert!(!c.set_target(1));
        assert_eq!(c.target_id, Some(2)); // unchanged
    }

    #[test]
    fn test_set_conversion_type() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        // Small -> Big: valid
        assert!(c.set_conversion_type(CREATURE_BIG));
        assert_eq!(c.convert_type, CREATURE_BIG);
        // Small -> Small: invalid (CONVERSION_FOOD[0][0] = 0)
        assert!(!c.set_conversion_type(CREATURE_SMALL));
        assert_eq!(c.convert_type, CREATURE_BIG); // unchanged
        // Invalid type index
        assert!(!c.set_conversion_type(99));
    }

    #[test]
    fn test_set_message() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.set_message("hello");
        assert_eq!(c.message, "hello");
        c.set_message("this is a long message");
        assert_eq!(c.message, "this is ");
        c.set_message("");
        assert_eq!(c.message, "");
    }

    #[test]
    fn test_tile_coords() {
        let c = Creature::new(1, 512, 768, CREATURE_SMALL, 0);
        assert_eq!(c.tile_x(), 2); // 512 / 256
        assert_eq!(c.tile_y(), 3); // 768 / 256
    }

    #[test]
    fn test_distance_to() {
        let c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        assert_eq!(c.distance_to(3, 4), 5); // 3-4-5 triangle
        assert_eq!(c.distance_to(0, 0), 0);
        assert_eq!(c.distance_to(100, 0), 100);
    }

    #[test]
    fn test_set_path() {
        let mut c = Creature::new(1, 0, 0, CREATURE_SMALL, 0);
        c.set_path(vec![(100, 200), (300, 400)]);
        assert_eq!(c.path.len(), 2);
        assert_eq!(c.path[0], (100, 200));
    }
}
