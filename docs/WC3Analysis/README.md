# WC3 / Warsmash Engine Analysis

Reference documentation for reimplementing the Warcraft III engine stack in another language (Rust, Go, C++, etc.).

**Source engine:** [Warsmash Mod Engine](https://github.com/manawenuz/WarsmashModEngine)  
**Language:** Java 17 (Java 8 syntax), LibGDX 1.12.1, LWJGL3, OpenAL  
**Dev system asset path:** `/Volumes/samGames/WC3/` (external drive, MPQ-based Patch 1.27)

---

## Document Index

| File | What it covers |
|---|---|
| [01_architecture.md](01_architecture.md) | Module layout, entry point, startup sequence |
| [02_asset_loading.md](02_asset_loading.md) | MPQ archives, CASC, DataSource abstraction, INI config |
| [03_file_formats.md](03_file_formats.md) | Binary format specs: BLP, MDX, W3X, SLK, FDF, JASS |
| [04_rendering.md](04_rendering.md) | MDX model rendering, terrain, shaders, HiDPI |
| [05_jass_interpreter.md](05_jass_interpreter.md) | JASS parser, interpreter, native function registry, triggers |
| [06_game_simulation.md](06_game_simulation.md) | Map loading, units, behaviors, combat, simulation tick |
| [07_ui_system.md](07_ui_system.md) | FDF frame parser, GameUI, coordinate system, rendering |
| [08_audio.md](08_audio.md) | OpenAL integration, format decoders, 3D positional audio |
| [09_dev_environment.md](09_dev_environment.md) | Local paths, build commands, run.sh, known macOS quirks |

---

## One-Page Summary

Warsmash is a clean-room reimplementation of the WC3 engine. Here is how the systems connect:

```
warsmash.ini
  └─ CompoundDataSource (layered virtual FS)
       ├─ MpqDataSource (.mpq archives)
       └─ FolderDataSource (flat folders / resources/)

On startup:
  DesktopLauncher
    └─ War3MapViewer (AbstractMdxModelViewer)
         ├─ Load terrain      ← War3MapW3e  → Terrain (GL)
         ├─ Load doodads      ← War3MapDoo  → MdxModel instances
         ├─ Load units        ← War3MapUnitsDoo → CSimulation
         ├─ Load data tables  ← UnitData.slk, AbilityData.slk …
         ├─ Execute JASS      ← war3map.j / war3map.lua
         └─ Render loop       → Scene → WebGL → screen

Each game tick:
  simulation.update()
    ├─ Move units (CBehaviorMove → CPathfindingProcessor)
    ├─ Combat  (CBehaviorAttack → CUnitAttack → projectiles)
    ├─ Trigger events (JassGameEventsWar3 → GlobalScope threads)
    └─ Destructable / item updates

UI:
  GameUI
    ├─ Parse FDF files (ANTLR grammar)
    ├─ Create frame hierarchy (UIFrame tree)
    └─ Render via SpriteBatch (orthographic projection)
```

## Key Design Principles to Carry Over

1. **DataSource is the only I/O interface.** Nothing reads files directly — everything goes through `DataSource.getResourceAsStream(path)`. This is the abstraction you want to keep in a rewrite.

2. **Simulation is decoupled from rendering.** `CSimulation` knows nothing about OpenGL. It works in game-world coordinates (float x/y, tile units). The renderer reads `CUnit.position` and draws.

3. **JASS is not optional.** All map logic runs in JASS. If you want to run any WC3 map, you need a JASS interpreter.

4. **FDF drives all UI.** Every menu, button, and dropdown is defined in `.fdf` text files loaded from the MPQ. The Java code only creates generic frame objects; the FDF file controls layout, sizing, and anchoring.

5. **SLK is the data layer.** Unit stats, abilities, upgrades — all in tab-separated `.slk` spreadsheets inside the MPQ. Nothing is hard-coded.
