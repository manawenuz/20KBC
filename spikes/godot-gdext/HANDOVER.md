# Handover — Godot 4 + gdext Spike

## Status: Running with runtime WC3 assets ✅

The spike loads MDX models, BLP textures, and MPQ archives **at runtime**
(no offline glTF conversion), renders peasants with skinned animation,
and drives geoset visibility per-frame from GEOA curves the same way
WC3 / WarsmashModEngine does.

### What works

- Terrain mesh + day/night + camera + selection ring + HUD (early-spike work).
- `SimBridge` ticks `game-core` at 20 Hz; positions sync to `UnitNode` / `GaiaNode` / `BuildingNode` / `ResourceNode` each tick.
- **Runtime asset pipeline** (PRDs 30–36):
  - `datasource/mpq.rs` opens `/Volumes/samGames/WC3/War3.mpq` and reads files by virtual path.
  - `blp/mod.rs` decodes BLP1 textures to RGBA8.
  - `mdx/parser.rs` parses MDX800 binary (VERS, MODL, SEQS, MTLS, TEXS, GEOS, GEOA, BONE, HELP, PIVT).
  - `mdx/builder.rs` builds `ArrayMesh` with positions / normals / UVs / `ARRAY_BONES` / `ARRAY_WEIGHTS` and emits one surface per geoset.
  - `mdx/skin.rs` builds `Skeleton3D` + `Skin` (parent-relative rest, absolute-pivot inverse-bind, self-cycle guard).
  - `mdx/animation.rs` emits an `AnimationLibrary` with one `Animation` per sequence; position keys are `rest + delta`.
  - `asset_registry.rs` is a thread-local cache from `mdx_path → ResolvedModel` (mesh, textures, skeleton, skin, anims, parsed model).
- **Per-frame geoset alpha** (PRD-37, Warsmash-style):
  - `mdx_instance.rs` is a `Node` child of each visual; every frame it samples each geoset's GEOA curve at the active sequence's elapsed time and writes the result into the corresponding surface material's `albedo.a`.
  - GEOA sampling is sequence-scoped: only keys within `[seq_start, seq_end]` are interpolated; no in-window keys → `static_alpha`. Matches WC3's runtime semantics so out-of-window keys (e.g. Stand Work) don't bleed into Stand.
  - Materials use `Transparency::ALPHA` so fades are smooth, no halos.
  - Behavior-driven animation switch in `UnitNode::process` calls `mdx_instance.set_sequence(name)` so the alpha sampler tracks the same window the `AnimationPlayer` is in.

### What's rough or unfinished

- **Posture issues remain on some peasants** — the top-left peasant in screenshot #31 is hunched. Most likely cause: MDX scale tracks (`KGSC`) emitted into Godot `SCALE_3D` tracks have keyframes that drive bones to near-zero scale. Suspect fix mirrors what we did for GEOA: scope scale-track samples to the active sequence window (or clamp away anything below ~0.01).
- **Arms** — fixed in principle by the sequence-windowed sampler in commit `e3e106f`, but not yet visually re-verified.
- **Only `UnitNode` is wired to `MdxInstance`.** `GaiaNode` (wolf), `BuildingNode`, and `ResourceNode` still go through the older "build materials inline" path. Same wiring needs to be applied to all four for consistent behavior (death-fade alpha on units, fall-down trees, etc.).
- **Team color**: `team_color.rs` and the replaceable-texture path exist (PRD-35) but are inactive — peasants are not painted per-player.
- **WC3 multi-layer shader**: `wc3_material/mod.rs` exists (PRD-33) but isn't used; everything renders with `StandardMaterial3D`.
- **Lumber-on-shoulder when carrying wood**: deferred. With per-frame GEOA alpha working, this becomes free once the sim emits a "carrying lumber" behavior that selects the right sequence (Stand Work / Stand Lumber).
- **Keep producing peasants** is a gameplay feature, never started.

---

## Project layout (current)

```
spikes/godot-gdext/
├── project.godot                   Godot 4 project (physics=20Hz)
├── spike-godot-gdext.gdextension   Extension manifest
├── scenes/Main.tscn                Root scene
├── scripts/main.gd                 Bootstrap + per-tick position sync
├── rust/
│   ├── Cargo.toml                  cdylib; game-core = ../../../game-core
│   └── src/
│       ├── lib.rs                  GDExtension entry point + module list
│       ├── sim_bridge.rs           Wraps CSimulation, initializes AssetRegistry
│       ├── asset_registry.rs      Thread-local MDX/BLP cache → ResolvedModel
│       ├── mdx/                    parser, builder, skin, animation, types
│       │   └── mod.rs              sample_alpha_at (sequence-scoped GEOA sampler)
│       ├── mdx_instance.rs         Per-spawn per-frame alpha driver (PRD-37)
│       ├── blp/mod.rs              BLP1 decoder → RGBA8
│       ├── datasource/             mpq, compound (compound unused so far)
│       ├── terrain_node.rs / day_night.rs / camera_controller.rs / hud.rs / ...
│       ├── unit_node.rs            Peasant visual; wired to MdxInstance
│       ├── gaia_node.rs            Wolf visual; not yet on MdxInstance
│       ├── building_node.rs        Keep/Castle/Townhall; not yet on MdxInstance
│       ├── resource_node_visual.rs Tree/stone; not yet on MdxInstance
│       ├── team_color.rs           Built (PRD-35), inactive
│       └── wc3_material/mod.rs     Built (PRD-33), inactive
└── target/                         (gitignored)
```

---

## Build / run

```bash
cd spikes/godot-gdext/rust && cargo build
godot --path /Users/manwe/CascadeProjects/20KBC/spikes/godot-gdext/
```

Requires Godot 4.6+ and a War3.mpq at `/Volumes/samGames/WC3/War3.mpq`.
If the MPQ is unreachable, the visual nodes fall back to the legacy
`res://assets/models/*.glb` path (or capsule for `UnitNode`), so the
game still runs without textures/animation.

---

## Key design decisions

**Thread-local AssetRegistry.** `Gd<T>` is `!Send`, so the registry is
`thread_local!<RefCell<Option<AssetRegistry>>>`. All Godot callbacks
run on the main thread anyway.

**Per-instance materials, shared mesh.** `ResolvedModel.mesh` is shared
across all spawns of the same MDX (cached). Materials are recreated per
spawn so `MdxInstance` can mutate `albedo.a` without bleeding alpha
across peasants. `Skeleton3D` is also recreated (via `.duplicate()`)
per spawn because nodes can have only one parent.

**Sequence-scoped GEOA sampling.** Critical for correctness: each WC3
sequence is independent. Sampling keys globally would let a Stand Work
key with alpha=0 fade out the arm geoset during Stand. The fix is in
`mdx/mod.rs::sample_alpha_at(entry, t, seq_start, seq_end)`.

**WC3 → Godot coord transform.** `(x, y, z) → (x, z, -y)`. This flips
handedness, so triangle winding is reversed (`i0, i2, i1`) in
`builder.rs`. Normals get the same swizzle. Pivots, bind matrices, and
animation translation keys all live in Godot space after conversion.

**Animation position tracks add rest.** WC3 `KGTR` keys are deltas from
the bone's bind pose in parent space, but Godot `POSITION_3D` tracks
replace the pose position. `animation.rs` therefore inserts
`rest + delta` for each keyframe. Rotations use identity rest so they
go in unchanged.

---

## Known gotchas

- **Skeleton self-cycles**: some MDX nodes have `parent_id == object_id`. `skin.rs` filters these defensively (`if parent_id == 0xFFFFFFFF || parent_id == object_id`). Same filter applied in the offline glTF writer historically.
- **HELP chunk required**: peasant uses a HELP node (Bone_Root) to connect Pelvis to Chest. Parser reads HELP with the same node-header reader as BONE.
- **Material layer 0 is often team color**: `builder.rs::resolve_texture_name` walks **all** layers and picks the first one with `replaceable_id == 0` and a non-empty file_name. Picking layer 0 misses the actual body texture on units like peasants.
- **Disk space**: `cargo build` outputs to `spikes/godot-gdext/target/`. The repo root and any worktrees can accumulate multi-GB `target/` directories. `rm -rf` is safe.
- **Kimi dispatch is fragile**: `scripts/dispatch-kimi-plain.sh` works but Kimi processes may exit silently. Cap concurrent Kimi runs at ≤4. If a worker stalls, kill it and integrate manually from the worktree.

---

## Next tasks (priority order)

1. **Visually verify arms appear** after `e3e106f` (sequence-scoped GEOA). Screenshot a fresh launch.
2. **Fix hunched-posture peasant** (scale tracks). Scope scale-track keyframes to the active sequence window in `animation.rs`, mirroring the GEOA fix.
3. **Apply `MdxInstance` to `GaiaNode`, `BuildingNode`, `ResourceNode`**. Same wiring as `UnitNode::try_spawn_from_registry`:
   - Build materials per-instance.
   - Add `MdxInstance` child, configure with `(model, geoset_indices, materials)`.
   - Call `set_sequence` on behavior transitions.
4. **Activate team color** (PRD-35). The cache + replaceable-id logic is built; UnitNode needs to look up the unit's player owner via `SimBridge` and request the matching team-color texture for surfaces whose original `TextureRef.replaceable_id == 1`.
5. **Activate WC3 multi-layer shader** (PRD-33). Replace `StandardMaterial3D` with the `ShaderMaterial` that handles MDX filter modes (None / Transparent / Blend / Additive / AddAlpha / Modulate / Modulate2x). Needed for the team-color glow, fire, water, etc.
6. **Keep produces peasants**: gameplay feature, needs game-core support (production queue per building) + GDScript UI.
7. **Lumber on shoulder**: requires sim-side "carrying lumber" behavior bit + `UnitNode` mapping it to "Stand Work" / "Stand Lumber". With per-frame GEOA alpha working, the bundle geoset will appear automatically when the right sequence is active.

---

## game-core interface (current)

```rust
fn issue_move_order(&mut self, unit_id: u32, x: f32, z: f32)
fn get_unit_positions(&self) -> Array<Vector2>
fn get_unit_facing(&self, unit_id: u32) -> f32
fn get_unit_behavior(&self, unit_id: u32) -> i64
fn get_wood(&self) -> u32
fn get_stone(&self) -> u32
fn get_resource_amount(&self, node_id: u32) -> i64
// plus gaia / resource / building accessors used by main.gd
```

---

## PRDs

Located in `prds/`:
- 30 — Runtime MPQ data source
- 31 — Runtime BLP decoder
- 32 — Runtime MDX parser + mesh builder
- 33 — WC3 multi-layer material shader (built, inactive)
- 34 — Runtime skinning + animation library
- 35 — Team color (built, inactive)
- 36 — UnitNode / BuildingNode runtime integration
- 37 — Warsmash-style per-frame geoset alpha (landed in commit 497d7b3 + e3e106f)
