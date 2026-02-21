// Creature types
pub const CREATURE_SMALL: u8 = 0;
pub const CREATURE_BIG: u8 = 1;
pub const CREATURE_FLYER: u8 = 2;

pub const CREATURE_TYPES: usize = 4;
pub const MAX_CREATURES: usize = 256;

// Creature states
pub const CREATURE_IDLE: u8 = 0;
pub const CREATURE_WALK: u8 = 1;
pub const CREATURE_HEAL: u8 = 2;
pub const CREATURE_EAT: u8 = 3;
pub const CREATURE_ATTACK: u8 = 4;
pub const CREATURE_CONVERT: u8 = 5;
pub const CREATURE_SPAWN: u8 = 6;
pub const CREATURE_FEED: u8 = 7;

pub const CREATURE_STATES: usize = 8;

// World constants
pub const TILE_SOLID: u8 = 0;
pub const TILE_PLAIN: u8 = 1;
pub const MAX_TILE_FOOD: i32 = 9999;
pub const TILE_SIZE: i32 = 256; // pixels per tile

// Tile GFX types
pub const TILE_GFX_SOLID: u8 = 0;
pub const TILE_GFX_PLAIN: u8 = 1;
pub const TILE_GFX_BORDER: u8 = 2;
pub const TILE_GFX_SNOW_SOLID: u8 = 3;
pub const TILE_GFX_SNOW_PLAIN: u8 = 4;
pub const TILE_GFX_SNOW_BORDER: u8 = 5;
pub const TILE_GFX_WATER: u8 = 6;
pub const TILE_GFX_LAVA: u8 = 7;
pub const TILE_GFX_NONE: u8 = 8;
pub const TILE_GFX_KOTH: u8 = 9;
pub const TILE_GFX_DESERT: u8 = 10;

// Max health per type [small, big, flyer, unused]
pub const MAX_HEALTH: [i32; CREATURE_TYPES] = [10000, 20000, 5000, 0];

// Max food per type
pub const MAX_FOOD: [i32; CREATURE_TYPES] = [10000, 20000, 5000, 0];

// Aging (health drain per 100ms)
pub const AGING: [i32; CREATURE_TYPES] = [5, 7, 5, 0];

// Base speed
pub const BASE_SPEED: [i32; CREATURE_TYPES] = [200, 400, 800, 0];

// Health-dependent speed bonus
pub const HEALTH_SPEED: [i32; CREATURE_TYPES] = [625, 0, 0, 0];

// Heal rate (food->health per second)
pub const HEAL_RATE: [i32; CREATURE_TYPES] = [500, 300, 600, 0];

// Eat rate (tile food->creature food per second)
pub const EAT_RATE: [i32; CREATURE_TYPES] = [800, 400, 600, 0];

// Attack damage per second [attacker][target]
pub const HITPOINTS: [[i32; CREATURE_TYPES]; CREATURE_TYPES] = [
    [0, 0, 1000, 0],      // Small attacks
    [1500, 1500, 1500, 0], // Big attacks
    [0, 0, 0, 0],          // Flyer attacks (none)
    [0, 0, 0, 0],
];

// Attack range [attacker][target]
pub const ATTACK_DISTANCE: [[i32; CREATURE_TYPES]; CREATURE_TYPES] = [
    [0, 0, 768, 0],     // Small
    [512, 512, 512, 0],  // Big
    [0, 0, 0, 0],        // Flyer
    [0, 0, 0, 0],
];

// Conversion food needed [from][to]
pub const CONVERSION_FOOD: [[i32; CREATURE_TYPES]; CREATURE_TYPES] = [
    [0, 8000, 5000, 0],  // From Small
    [8000, 0, 0, 0],     // From Big
    [5000, 0, 0, 0],     // From Flyer
    [0, 0, 0, 0],
];

// Conversion speed (food consumed per second)
pub const CONVERSION_SPEED: [i32; CREATURE_TYPES] = [1000, 1000, 1000, 0];

// Spawn food cost
pub const SPAWN_FOOD: [i32; CREATURE_TYPES] = [0, 5000, 0, 0];

// Spawn speed (food consumed per second during spawn)
pub const SPAWN_SPEED: [i32; CREATURE_TYPES] = [0, 2000, 0, 0];

// Health cost on spawn start
pub const SPAWN_HEALTH: [i32; CREATURE_TYPES] = [0, 4000, 0, 0];

// Type of offspring (-1 = cannot spawn)
pub const SPAWN_TYPE: [i32; CREATURE_TYPES] = [-1, 0, -1, -1];

// Feed distance
pub const FEED_DISTANCE: [i32; CREATURE_TYPES] = [256, 0, 256, 0];

// Feed speed (food transferred per second)
pub const FEED_SPEED: [i32; CREATURE_TYPES] = [400, 0, 400, 0];
