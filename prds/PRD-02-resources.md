# PRD-02 — Resource Node Rendering + Gather Order

## Goal

Render the wood/stone resource nodes that already exist in the `CSimulation` as
visible boxes in Godot, and let the player right-click a resource to issue a
gather order. Add a Rust GDExtension class `ResourceNode` (visual) and expose
`get_resource_nodes()` + `issue_gather_order()` on `SimBridge`.

## Context

`game-core` already has resources — see `game-core/src/resource_node.rs`.
`CSimulation::resource_nodes: Vec<CResourceNode>` contains them. Each has:
- `id: u32` (ResourceNodeId)
- `kind: ResourceKind` (`Wood` or `Stone`)
- `pos: Vec2` (world position; treat as XZ plane)
- `amount: u32`, `max_amount: u32`

`CSimulation::issue_order(unit_id, Order::Gather { node: ResourceNodeId })`
already exists. The behavior is implemented in `simulation.rs::tick_gather`.

Currently three wood nodes spawn near map center (see `simulation.rs::new`).

## Files you MAY create

- `spikes/godot-gdext/rust/src/resource_node_visual.rs`

(Named `_visual` to disambiguate from the `CResourceNode` in game-core.)

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod resource_node_visual;` only
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add TWO new `#[func]` methods:
  - `get_resource_nodes(&self) -> Array<Vector3>` — flat array of (id, x, z) per node
  - `issue_gather_order(&mut self, unit_id: u32, node_id: u32)`

## Files you MUST NOT touch

- `spikes/godot-gdext/scenes/Main.tscn`
- `spikes/godot-gdext/scripts/main.gd`
- `spikes/godot-gdext/project.godot`
- `game-core/**`
- Any other Rust source

## Interface contract

```rust
// resource_node_visual.rs
#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct ResourceNode {
    /// 1 = Wood (brown), 2 = Stone (grey). Set BEFORE adding to scene tree
    /// so ready() picks the right color.
    #[var] pub kind: u32,
    /// Sim-side ResourceNodeId — used to correlate right-click → gather order.
    #[var] pub node_id: u32,
    base: Base<Node3D>,
}
// In ready(): build a BoxMesh child MeshInstance3D, 1.5×1.5×1.5,
// positioned at (0, 0.75, 0). Color by kind:
//   Wood:  Color { r: 0.42, g: 0.27, b: 0.13, a: 1.0 }
//   Stone: Color { r: 0.55, g: 0.55, b: 0.58, a: 1.0 }
//   default (unknown kind): magenta Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 }
// StandardMaterial3D, PER_PIXEL shading.
```

```rust
// sim_bridge.rs — append to existing #[godot_api] impl SimBridge

/// Returns a flat Array<Vector3> where each element packs (id as f32, x, z).
/// GDScript decodes: id = int(v.x), pos = Vector3(v.y, 0, v.z).
/// Only non-depleted nodes are returned (amount > 0).
#[func]
pub fn get_resource_nodes(&self) -> Array<Vector3> {
    // Iterate self.sim.as_ref()?.iter_resources(), filter !n.is_depleted(),
    // push Vector3::new(n.id as f32, n.pos.x, n.pos.y).
}

/// Order `unit_id` to gather from `node_id`. Forwards to CSimulation::issue_order.
#[func]
pub fn issue_gather_order(&mut self, unit_id: u32, node_id: u32) {
    // sim.issue_order(unit_id, Order::Gather { node: node_id })
}
```

Note: encode wood-vs-stone in a separate `get_resource_kinds() -> Array<i64>`
method if convenient, OR fold it into a fourth field. To keep this PRD tight,
add this helper instead:

```rust
/// Returns kinds aligned with get_resource_nodes order: 1=Wood, 2=Stone.
#[func]
pub fn get_resource_kinds(&self) -> Array<i64> { ... }
```

## Implementation hints

- `BoxMesh` is in `godot::classes::BoxMesh`. Use `set_size(Vector3::new(1.5, 1.5, 1.5))`.
- The existing `unit_node.rs` shows the capsule pattern. `BoxMesh` is the same shape
  of code, just a different mesh type.
- `ResourceKind` in game-core is an enum. Match on `&n.kind` and map to `1` or `2`.

## Acceptance criteria

Run from `spikes/godot-gdext/rust/`:

```bash
cargo build
```

Must succeed cleanly. Additionally:

- [ ] `resource_node_visual.rs` exists and compiles
- [ ] `lib.rs` has exactly one new line: `mod resource_node_visual;`
- [ ] `sim_bridge.rs` has the 3 new `#[func]` methods (nodes, kinds, gather order)
- [ ] No other files modified
- [ ] `git diff --stat` shows at most 3 files changed

## Out of scope

- Wiring `ResourceNode` instantiation into `main.gd`
- Right-click ray intersection in GDScript
- Visual "amount remaining" indicator
- Stone node spawning (sim currently only has wood — leave it)
