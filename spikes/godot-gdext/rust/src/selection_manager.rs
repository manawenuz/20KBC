use std::collections::HashSet;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct SelectionManager {
    selected: HashSet<u32>,
    base: Base<Node>,
}

#[godot_api]
impl INode for SelectionManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            selected: HashSet::new(),
            base,
        }
    }
}

#[godot_api]
impl SelectionManager {
    #[func]
    pub fn clear(&mut self) {
        self.selected.clear();
    }

    #[func]
    pub fn add(&mut self, unit_id: u32) {
        self.selected.insert(unit_id);
    }

    #[func]
    pub fn remove(&mut self, unit_id: u32) {
        self.selected.remove(&unit_id);
    }

    #[func]
    pub fn contains(&self, unit_id: u32) -> bool {
        self.selected.contains(&unit_id)
    }

    /// Returns the selection as an Array<i64> of unit ids (sorted ascending).
    #[func]
    pub fn get_all(&self) -> Array<i64> {
        let mut ids: Vec<u32> = self.selected.iter().copied().collect();
        ids.sort();
        let mut arr = Array::new();
        for uid in ids {
            arr.push(uid as i64);
        }
        arr
    }

    #[func]
    pub fn count(&self) -> i64 {
        self.selected.len() as i64
    }
}
