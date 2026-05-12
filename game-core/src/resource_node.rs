use glam::Vec2;

pub type ResourceNodeId = u32;

#[derive(Clone, Debug)]
pub enum ResourceKind {
    Wood,
    Stone,
}

#[derive(Clone, Debug)]
pub struct CResourceNode {
    pub id: ResourceNodeId,
    pub kind: ResourceKind,
    pub pos: Vec2,
    pub amount: u32,
    pub max_amount: u32,
}

impl CResourceNode {
    pub fn new(id: ResourceNodeId, kind: ResourceKind, pos: Vec2, amount: u32) -> Self {
        Self {
            id,
            kind,
            pos,
            amount,
            max_amount: amount,
        }
    }

    /// Attempt to harvest `quantity` from this node. Returns amount actually taken.
    pub fn harvest(&mut self, quantity: u32) -> u32 {
        let taken = quantity.min(self.amount);
        self.amount -= taken;
        taken
    }

    pub fn is_depleted(&self) -> bool {
        self.amount == 0
    }
}
