# PRD-16 — Selected-Unit Portrait + HP/Supply HUD

## Goal

When the player selects exactly one unit, the HUD shows a portrait
panel in the bottom-left with:
- Unit name (hardcoded "Worker" for now)
- HP bar (green to red, current/max)
- Supply bar (yellow)
- Unit count of current selection (top-right of the panel)

When multiple units selected, show summary: "N units selected" + sum HP
bar.

## Context

There is already a `SelectionCountLabel` in the HUD. We extend the HUD
with a new Rust Control type `UnitPortrait`.

## Files you MAY create

- `spikes/godot-gdext/rust/src/unit_portrait.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod unit_portrait;`
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add ONE `#[func]`:
  `get_unit_stats(&self, unit_id: u32) -> Dictionary` returning
  `{ "hp": f32, "max_hp": f32, "supply": f32, "max_supply": f32 }`.

## Files you MUST NOT touch

- `main.gd`, `Main.tscn`, `project.godot`
- Other Rust source (`unit_node.rs`, etc.)

## Interface contract

```rust
// unit_portrait.rs
use godot::prelude::*;
use godot::classes::{Control, IControl, ProgressBar, Label};

#[derive(GodotClass)]
#[class(base = Control)]
pub struct UnitPortrait {
    base: Base<Control>,
}

#[godot_api]
impl IControl for UnitPortrait {
    fn init(base: Base<Control>) -> Self { Self { base } }
    // No process — orchestrator pushes data in via #[func]s below.
}

#[godot_api]
impl UnitPortrait {
    /// Called by main.gd whenever selection changes.
    /// On 0 units: hide the panel.
    /// On 1 unit: show name + per-unit HP/supply.
    /// On N units: show "N units" + aggregate HP.
    #[func]
    pub fn show_single(&mut self, hp: f32, max_hp: f32, supply: f32, max_supply: f32) {
        // self.base().get_node("NameLabel") as Label → set_text("Worker")
        // self.base().get_node("HpBar") as ProgressBar → set_max(max_hp), set_value(hp)
        // self.base().get_node("SupplyBar") as ProgressBar → set_max(max_supply), set_value(supply)
        // self.base().get_node("CountLabel") as Label → set_text("1 unit")
        // self.base_mut().set_visible(true)
    }
    #[func]
    pub fn show_multi(&mut self, count: i64, total_hp: f32, total_max_hp: f32) {
        // Show name = "N units selected"
        // HP bar reflects sum
        // hide supply bar
        // Set visible
    }
    #[func] pub fn hide_panel(&mut self) {
        // self.base_mut().set_visible(false)
    }
}
```

```rust
// sim_bridge.rs — append
#[func]
pub fn get_unit_stats(&self, unit_id: u32) -> Dictionary {
    let mut d = Dictionary::new();
    if let Some(sim) = &self.sim {
        if let Some(u) = sim.iter_units().find(|u| u.id == unit_id) {
            d.set("hp", u.hp);
            d.set("max_hp", u.max_hp);
            d.set("supply", u.supply);
            d.set("max_supply", u.max_supply);
        }
    }
    d
}
```

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
```

- [ ] Clean build
- [ ] `unit_portrait.rs` exists with 3 `#[func]`s
- [ ] `sim_bridge.rs` has `get_unit_stats`
- [ ] ≤ 3 files modified

## Out of scope

- Actual portrait texture (orchestrator may add via res:// later)
- Command card / ability buttons
- Multi-row unit grid
