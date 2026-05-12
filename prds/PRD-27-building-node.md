# PRD-27 — `BuildingNode` Visual + Scene Integration

## Goal

Render the 3 buildings spawned by PRD-26 in the Godot scene by adding a
new GDExtension class `BuildingNode` that loads the right `.glb` per
kind. This PRD **depends on PRDs 25 & 26 being merged.**

## Files you MAY create

- `spikes/godot-gdext/rust/src/building_node.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod building_node;` only

## Files you MUST NOT touch

- `main.gd`, `Main.tscn`, `project.godot` — orchestrator will wire
  spawning + scene integration after merge.
- Other Rust source
- `scripts/asset-extract/**`
- `game-core/**`

## Interface contract

```rust
// building_node.rs — pattern mirrors gaia_node.rs / resource_node_visual.rs
use godot::prelude::*;
use godot::classes::{INode3D, Node3D, PackedScene, ResourceLoader, StandardMaterial3D, BoxMesh, MeshInstance3D};
use godot::classes::base_material_3d::ShadingMode;

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct BuildingNode {
    /// 1 = TownHall, 2 = Keep, 3 = Castle. Set BEFORE adding to scene
    /// tree so ready() picks the right model.
    #[var] pub kind: u32,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingNode {
    fn init(base: Base<Node3D>) -> Self {
        Self { kind: 0, base }
    }
    fn ready(&mut self) {
        let path = match self.kind {
            1 => "res://assets/models/townhall.glb",
            2 => "res://assets/models/keep.glb",
            3 => "res://assets/models/castle.glb",
            _ => "",
        };
        let scene: Option<Gd<PackedScene>> = ResourceLoader::singleton()
            .load(path)
            .and_then(|r| r.try_cast::<PackedScene>().ok());
        if let Some(s) = scene {
            if let Some(inst) = s.instantiate() {
                if let Ok(mut n) = inst.clone().try_cast::<Node3D>() {
                    // WC3 buildings use the same ~1cm-per-unit scale.
                    n.set_scale(Vector3::new(0.02, 0.02, 0.02));
                }
                self.base_mut().add_child(&inst);
                return;
            }
        }
        // Fallback: a coloured cube so missing models are obviously placeholders.
        let color = match self.kind {
            1 => Color { r: 0.65, g: 0.55, b: 0.35, a: 1.0 },
            2 => Color { r: 0.55, g: 0.55, b: 0.60, a: 1.0 },
            3 => Color { r: 0.70, g: 0.65, b: 0.55, a: 1.0 },
            _ => Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 },
        };
        let mut mesh = BoxMesh::new_gd();
        mesh.set_size(Vector3::new(6.0, 6.0, 6.0));
        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(color);
        mat.set_shading_mode(ShadingMode::PER_PIXEL);
        mesh.surface_set_material(0, &mat);
        let mut inst = MeshInstance3D::new_alloc();
        inst.set_mesh(&mesh);
        inst.set_position(Vector3::new(0.0, 3.0, 0.0));
        self.base_mut().add_child(&inst);
    }
}
```

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
```

- [ ] Clean build
- [ ] `building_node.rs` compiles with the BoxMesh fallback path intact
- [ ] `lib.rs` has the new `mod building_node;`
- [ ] ≤ 2 files modified

## Out of scope

- Wiring instantiation into `main.gd` — orchestrator
- Wiring scene parent node — orchestrator
- Selection/clickability on buildings
- Animations on buildings
