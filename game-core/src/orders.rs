use glam::Vec2;
use crate::unit::UnitId;
use crate::player::PlayerId;
use crate::resource_node::ResourceNodeId;

#[derive(Clone, Debug)]
pub enum Order {
    Stop,
    Move { target: Vec2 },
    Gather { node: ResourceNodeId },
    Attack { target: UnitId },
    AttackMove { target: Vec2 },
}

/// One recorded input event. Written to `replay.bin` for deterministic replay.
#[derive(Clone, Debug)]
pub struct InputEntry {
    pub tick: u64,
    pub player: PlayerId,
    pub unit: UnitId,
    pub order: Order,
}
