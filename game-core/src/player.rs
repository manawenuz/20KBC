use glam::Vec2;

pub type PlayerId = u8;

#[derive(Clone, Debug)]
pub struct CPlayer {
    pub id: PlayerId,
    pub wood: u32,
    pub stone: u32,
    /// Food produced by structures; consumed by units.
    pub food: u32,
    /// World-space position of the player's supply depot (camp fire / hut).
    pub supply_depot: Option<Vec2>,
}

impl CPlayer {
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            wood: 100,
            stone: 0,
            food: 10,
            supply_depot: None,
        }
    }
}
