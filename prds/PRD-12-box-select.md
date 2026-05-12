# PRD-12 — Box-Drag Multi-Select

## Goal

Add a draggable selection rectangle. While left mouse is held and moved
> 4 pixels from the press position, draw a translucent yellow rectangle.
On release, all units whose ground position is inside that rectangle
(after projecting to screen space) become the new selection.

## Context

Single-click selection already works via `SimBridge::get_unit_at`. We
need a second code path for drag-rect.

## Files you MAY create

- `spikes/godot-gdext/rust/src/box_selector.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod box_selector;` only
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add ONE new `#[func]`:
  `get_units_in_rect(&self, min_x: f32, min_z: f32, max_x: f32, max_z: f32) -> Array<i64>`
  returning UnitIds whose world XZ is inside the rect.

## Files you MUST NOT touch

- `main.gd`, `Main.tscn`, `project.godot` — orchestrator will wire
- Other Rust source

## Interface contract

```rust
// box_selector.rs — a Control that owns the drag rect drawing and emits a signal.

use godot::prelude::*;
use godot::classes::{Control, IControl, InputEvent, InputEventMouseButton, InputEventMouseMotion};

#[derive(GodotClass)]
#[class(base = Control)]
pub struct BoxSelector {
    dragging: bool,
    start: Vector2,
    end: Vector2,
    base: Base<Control>,
}

#[godot_api]
impl IControl for BoxSelector {
    fn init(base: Base<Control>) -> Self { ... }

    fn process(&mut self, _delta: f64) {
        // queue_redraw if dragging
    }

    fn draw(&mut self) {
        if !self.dragging { return; }
        let rect = ...; // min/max of start/end
        // draw yellow translucent fill + outline
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        // mouse_down LEFT → dragging = true, start = event.position
        // mouse_motion → end = event.position; queue_redraw
        // mouse_up LEFT → emit "selection_box" with (start, end) IF distance > 4 px,
        //                 set dragging = false, queue_redraw.
        //                 If distance <= 4 px, do nothing (single-click handled elsewhere).
    }
}

#[godot_api]
impl BoxSelector {
    /// Signal emitted on drag release. Args: Vector2 start, Vector2 end (screen px).
    #[signal] fn selection_box(start: Vector2, end: Vector2);
}
```

```rust
// sim_bridge.rs — append
#[func]
pub fn get_units_in_rect(&self, min_x: f32, min_z: f32, max_x: f32, max_z: f32) -> Array<i64> {
    // Iterate units, push id if !is_dead && min_x <= u.pos.x <= max_x && min_z <= u.pos.y <= max_z.
}
```

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
```

- [ ] Clean build
- [ ] `box_selector.rs` exists with `#[signal] selection_box`
- [ ] `lib.rs` adds one `mod` line
- [ ] `sim_bridge.rs` has `get_units_in_rect`
- [ ] ≤ 3 files modified

## Out of scope

- Shift-click additive selection
- Subtractive (alt-drag) selection
- Drawing the rect from main.gd — orchestrator wires the signal handler
  that performs the screen-to-world projection and calls
  `get_units_in_rect`.
