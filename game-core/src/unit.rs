use glam::Vec2;
use crate::resource_node::ResourceNodeId;

pub type UnitId = u32;

/// Per-unit behavior state machine. Variants are mutually exclusive active behaviors.
#[derive(Clone, Debug)]
pub enum BehaviorState {
    Idle,
    MovingTo {
        target: Vec2,
        path: Vec<Vec2>,
        path_idx: usize,
    },
    Gathering {
        node: ResourceNodeId,
        phase: GatherPhase,
    },
    Attacking {
        target: UnitId,
        cooldown: f32,
    },
    Dead,
}

#[derive(Clone, Debug)]
pub enum GatherPhase {
    MovingToNode,
    Harvesting { ticks_remaining: u32 },
    ReturningToBase,
}

/// A unit in the simulation. Owns its own behavior state.
#[derive(Clone, Debug)]
pub struct CUnit {
    pub id: UnitId,
    pub owner: crate::player::PlayerId,
    pub pos: Vec2,
    /// Facing angle in radians.
    pub facing: f32,
    pub hp: f32,
    pub max_hp: f32,
    /// Current supply reserve (0..=max_supply). Drains over time.
    pub supply: f32,
    pub max_supply: f32,
    pub behavior: BehaviorState,
    /// Remaining ticks before the next attack can land.
    pub attack_cooldown: f32,
    pub is_dead: bool,
}

impl CUnit {
    pub fn new(id: UnitId, owner: crate::player::PlayerId, pos: Vec2) -> Self {
        Self {
            id,
            owner,
            pos,
            facing: 0.0,
            hp: 100.0,
            max_hp: 100.0,
            supply: 10.0,
            max_supply: 10.0,
            behavior: BehaviorState::Idle,
            attack_cooldown: 0.0,
            is_dead: false,
        }
    }

    /// Movement speed in world-units per tick.
    pub const MOVE_SPEED: f32 = 4.0 / 20.0; // 4 world-units/s at 20 tps

    /// Melee attack range (world-units).
    pub const ATTACK_RANGE: f32 = 1.5;

    /// Ticks between attacks (1 attack/s at 20 tps).
    pub const ATTACK_COOLDOWN_TICKS: f32 = 20.0;

    /// Base attack damage per hit.
    pub const ATTACK_DAMAGE: f32 = 10.0;

    /// Harvesting duration in ticks.
    pub const HARVEST_TICKS: u32 = 40; // 2 seconds

    /// Wood/stone delivered per harvest cycle.
    pub const HARVEST_AMOUNT: u32 = 10;
}
