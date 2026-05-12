# PRD-36 — UnitNode / BuildingNode Runtime Integration

## Goal

Wire the new runtime pipeline (PRDs 30–35) into the live `UnitNode`,
`GaiaNode`, `BuildingNode`, `ResourceNode` classes so they load
MDX models from MPQ at startup instead of `res://assets/models/*.glb`.

This is the **integration PRD** — depends on 30, 31, 32, 33, 34, 35
all merged.

## Files you MAY create

- `spikes/godot-gdext/rust/src/asset_registry.rs` — singleton-style
  cache from "MDX path string" → `(Gd<ArrayMesh>, Gd<Skeleton3D>,
  Gd<AnimationLibrary>, surface_materials)`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod asset_registry;` only
- `spikes/godot-gdext/rust/src/unit_node.rs` — replace `ResourceLoader.load("res://assets/models/peasant.glb")`
  with `AssetRegistry::load("Units/Human/Peasant/peasant.mdx")`. Keep
  capsule fallback.
- `spikes/godot-gdext/rust/src/gaia_node.rs` — same pattern for wolf
- `spikes/godot-gdext/rust/src/building_node.rs` — same pattern, per kind
- `spikes/godot-gdext/rust/src/resource_node_visual.rs` — same pattern
  for tree/stone
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — initialise the
  `AssetRegistry` in `SimBridge::ready()` with the MPQ path
  (default: `/Volumes/samGames/WC3/War3.mpq`)

## Files you MUST NOT touch

- `spikes/godot-gdext/rust/src/datasource/**` (use, don't modify)
- `spikes/godot-gdext/rust/src/blp/**` (use, don't modify)
- `spikes/godot-gdext/rust/src/mdx/**` (use, don't modify)
- `spikes/godot-gdext/rust/src/wc3_material/**` (use, don't modify)
- `spikes/godot-gdext/rust/src/team_color/**` (use, don't modify)
- `main.gd`, `Main.tscn`, `project.godot`
- `game-core/**`
- `scripts/asset-extract/**`

## Interface contract

```rust
// asset_registry.rs
use godot::prelude::*;
use godot::classes::{ArrayMesh, AnimationLibrary, Skeleton3D, ShaderMaterial};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct ResolvedModel {
    pub mesh: Gd<ArrayMesh>,
    pub skeleton: Option<Gd<Skeleton3D>>,
    pub anims: Option<Gd<AnimationLibrary>>,
    pub materials: Vec<Gd<ShaderMaterial>>,
}

pub struct AssetRegistry {
    ds: Arc<dyn crate::datasource::DataSource>,
    cache: Mutex<HashMap<String, ResolvedModel>>,
    team_colors: Mutex<crate::team_color::TeamColorCache>,
}

impl AssetRegistry {
    pub fn new(mpq_path: &Path) -> Result<Self, String>;
    pub fn load(&self, mdx_path: &str) -> Option<ResolvedModel>;
}
```

Calling `load()` twice with the same path returns the same cached
result (avoid re-parsing 200KB on every unit spawn).

UnitNode example replacement:

```rust
// BEFORE:
let model = ResourceLoader::singleton().load("res://assets/models/peasant.glb")
    .and_then(|r| r.try_cast::<PackedScene>().ok());

// AFTER:
let model = SimBridge::asset_registry()
    .and_then(|reg| reg.load("Units/Human/Peasant/peasant.mdx"));
if let Some(m) = model {
    let mesh_inst = MeshInstance3D::new_alloc();
    mesh_inst.set_mesh(&m.mesh);
    for (i, mat) in m.materials.iter().enumerate() {
        mesh_inst.set_surface_override_material(i, &mat.clone().upcast::<Material>());
    }
    // Skeleton + anims attached as children if present
    self.base_mut().add_child(&mesh_inst);
}
```

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
godot --path /Users/manwe/CascadeProjects/20KBC/spikes/godot-gdext/ --quit-after 200
```

- [ ] Build clean
- [ ] At runtime, peasants load directly from War3.mpq (no
      `assets/models/peasant.glb` referenced anywhere in the runtime path)
- [ ] Capsule fallback still works if MPQ is unreachable (test by
      pointing the bridge at a bad path)
- [ ] First-spawn cost amortises after caching (second peasant should
      reuse the parsed mesh; verifiable via timing log)
- [ ] ≤ 7 files modified

## Out of scope

- Removing the offline `assets/models/*.glb` artifacts (leave for now;
  delete in a cleanup PRD once runtime is proven stable)
- Hot-reloading MDX changes
- LOD selection
