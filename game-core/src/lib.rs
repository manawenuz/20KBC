pub mod building;
pub mod config;
pub mod formation;
pub mod gaia;
pub mod orders;
pub mod pathfinding;
pub mod player;
pub mod resource_node;
pub mod rng;
pub mod simulation;
pub mod supply;
pub mod unit;

// Re-export glam math types so dependents don't need a separate glam dep.
pub use glam::Vec2;

// Re-export the public surface the spec requires.
pub use building::{BuildingKind, CBuilding};
pub use config::SimConfig;
pub use formation::formation_positions;
pub use gaia::CGaiaEntity;
pub use orders::{InputEntry, Order};
pub use pathfinding::GridPathfinder;
pub use player::{CPlayer, PlayerId};
pub use resource_node::{CResourceNode, ResourceKind, ResourceNodeId};
pub use rng::DeterministicRng;
pub use simulation::CSimulation;
pub use unit::{BehaviorState, CUnit, GatherPhase, UnitId};
