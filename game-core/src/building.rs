use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildingKind {
    TownHall = 1,
    Keep = 2,
    Castle = 3,
}

#[derive(Clone, Debug)]
pub struct CBuilding {
    pub kind: BuildingKind,
    pub pos: Vec2,
    pub rotation: f32, // radians, around Y / world-up
}

impl CBuilding {
    pub fn new(kind: BuildingKind, pos: Vec2, rotation: f32) -> Self {
        Self { kind, pos, rotation }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_kind_discriminants() {
        assert_eq!(BuildingKind::TownHall as u8, 1);
        assert_eq!(BuildingKind::Keep as u8, 2);
        assert_eq!(BuildingKind::Castle as u8, 3);
    }
}
