# Game Engine Comparison for 20KBC

[Back to Research Index](./README.md)

---

## Summary

We evaluated Rust-compatible game engines for building a prehistoric RTS with large maps, 20-30 player survival mode, dynamic weather, and day/night cycles. All options use permissive open-source licenses (MIT or Apache-2.0).

**Recommendation**: Spike both **Godot + gdext** and **Bevy** in parallel (2 weeks each), then commit.

---

## Engine Evaluations

### 1. Godot 4 + gdext (Rust Bindings) — Recommended

| Attribute | Details |
|-----------|---------|
| License | MIT (engine), open source (gdext) |
| gdext Version | v0.5 (March 2026) |
| 2D/3D | Full 2D and 3D support |
| Editor | Full visual scene editor, inspector, asset browser |
| Multiplayer | Built-in multiplayer synchronizer, extendable |
| Community | Godot: massive; gdext: growing, active Discord |

**Strengths**:
- Proven engine with years of production use
- Visual editor is indispensable for complex UI, scene layout, and asset management
- Rust integration improving rapidly (v0.5: removed mutex from `Gd<T>`, typed dicts, singleton support)
- Asset pipeline out of the box (import, convert, hot-reload)
- Vast majority of Godot APIs mapped to Rust
- GDExtension allows Rust as first-class language alongside GDScript

**Weaknesses**:
- gdext bindings still maturing — occasional breaking changes
- GDScript ecosystem (tutorials, plugins) doesn't directly help Rust developers
- Default multiplayer architecture is server-authoritative (need custom lockstep for RTS)
- Default player limit ~16 (need custom networking layer for 30-player survival)

**RTS Feasibility**: Strong. Godot handles rendering, UI, asset pipeline. Rust handles simulation, AI, networking. The editor dramatically speeds up iteration on UI and scene layout.

### 2. Bevy — Strong Alternative

| Attribute | Details |
|-----------|---------|
| License | MIT / Apache-2.0 (dual) |
| Version | v0.18.1 (March 2026) |
| Architecture | Data-driven ECS (Entity Component System) |
| 2D/3D | Full 2D and 3D rendering |
| Editor | None (code-only) |
| Community | 45.6k GitHub stars, 599+ contributors |

**Strengths**:
- Pure Rust — no FFI, no bindings, no impedance mismatch
- ECS architecture naturally fits RTS (entities = units/buildings, components = stats/position)
- Active RTS project exists: [Digital Extinction](https://github.com/DigitalExtinction/Game)
- Networking ecosystem: Bevy Replicon (high-level replication), Renet (low-level UDP)
- Deterministic lockstep well-documented in community
- Terrain plugins: bevy_terrain, bevy_generative
- Hot shader reloading, asset hot-reload via file_watcher
- UI via bevy_egui (mature integration, v0.39.1)

**Weaknesses**:
- No visual editor — all scene setup is code. Steep learning curve for non-programmers
- Still early-stage — breaking API changes between versions
- Documentation sparse in places
- Building complex UI (tech tree panel, minimap, building menus) is harder without an editor
- Smaller plugin/asset ecosystem than Godot

**RTS Feasibility**: Viable with more upfront effort. Full control over every aspect. The ECS architecture is arguably better for RTS simulation performance but worse for rapid UI/scene iteration.

### 3. Fyrox — Backup Option

| Attribute | Details |
|-----------|---------|
| License | MIT |
| Version | 1.0.0 (stable, 2026) |
| 2D/3D | Primary 3D, 2D support |
| Editor | Built-in scene editor (Fyroxed) |
| Community | Smaller but dedicated |

**Strengths**:
- Reached stable 1.0 after 7 years of development
- Built-in visual editor similar to Godot/Unity
- PBR rendering, deferred shading, volumetric lighting
- UI editor integrated
- Asset browser with shader preview

**Weaknesses**:
- Smallest community of the three — fewer resources, plugins, examples
- No networking solutions (must build from scratch)
- No RTS examples or demos
- Less terrain/procedural generation tooling
- Documentation is thinner

**RTS Feasibility**: Possible but would require building more from scratch. The editor is a plus, but the lack of community and networking ecosystem is a significant risk for a 2-person team.

### 4. Others (Not Recommended)

| Engine | Status | Why Not |
|--------|--------|---------|
| Amethyst | Discontinued | Dead project, do not use |
| Macroquad | Active, v0.4 | Too lightweight — primarily 2D, no advanced UI, no networking |
| Piston | Active | Too low-level, better for graphics research |
| ggez | Active | 2D only, requires SDL2, not suitable for production RTS |

---

## Non-Rust Engines Considered

| Engine | Rust Integration | Verdict |
|--------|-----------------|---------|
| Unity | Limited — C# only, Rust via FFI/DLLs | Not practical for "Rust wherever possible" |
| Unreal | No Rust support | Not viable |
| Godot (GDScript only) | N/A | Loses Rust advantage — use gdext instead |

---

## How Commercial RTS Games Handle Tech

- **StarCraft II**: Custom Blizzard engine, deterministic lockstep
- **Age of Empires IV**: Essence Engine (Relic), shifting to Unreal
- **Age of Empires III**: Havok physics
- **Warcraft III**: Custom Blizzard engine

All major commercial RTS games use **custom or proprietary engines**. None ship on Unity/Unreal/Godot (though this is changing). RTS requirements are demanding enough to warrant specialized solutions — which aligns with our pure-Rust game-core approach regardless of rendering engine choice.

---

## Decision Matrix

| Criteria (weight) | Godot+gdext | Bevy | Fyrox |
|-------------------|-------------|------|-------|
| Rust ergonomics (20%) | 3/5 | 5/5 | 4/5 |
| Editor/tooling (20%) | 5/5 | 1/5 | 4/5 |
| RTS networking (15%) | 3/5 | 4/5 | 1/5 |
| Community/ecosystem (15%) | 5/5 | 4/5 | 2/5 |
| Rendering capability (10%) | 4/5 | 4/5 | 4/5 |
| Terrain/map gen (10%) | 3/5 | 4/5 | 2/5 |
| Stability/maturity (10%) | 4/5 | 2/5 | 4/5 |
| **Weighted Score** | **3.8** | **3.3** | **2.9** |

**Conclusion**: Spike Godot+gdext (Dev A) and Bevy (Dev B) in parallel for 2 weeks, then decide based on hands-on experience.

---

## Sources

- [Bevy Engine](https://www.bevyengine.org/) — v0.18.1
- [Digital Extinction (Bevy RTS)](https://github.com/DigitalExtinction/Game)
- [Fyrox Engine](https://fyrox.rs/) — v1.0.0
- [godot-rust/gdext](https://github.com/godot-rust/gdext) — v0.5
- [Bevy Replicon](https://github.com/projectharmonia/bevy_replicon)
- [Renet networking](https://github.com/lucaspoffo/renet)
- [bevy_generative](https://github.com/manankarnik/bevy_generative)
- [noise-rs](https://github.com/Razaekel/noise-rs)
