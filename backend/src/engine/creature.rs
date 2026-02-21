/// Represents a creature in the game world.
pub struct Creature {
    pub id: u32,
    pub creature_type: u8,
    pub state: u8,
    pub player_id: u32,
    pub x: i32,
    pub y: i32,
    pub health: i32,
    pub food: i32,
    pub target_id: Option<u32>,
}
