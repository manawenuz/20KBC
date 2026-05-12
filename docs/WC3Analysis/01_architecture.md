# 01 — Architecture & Module Structure

## Gradle Subprojects

```
WarsmashModEngine/
├── core/           Game logic, rendering, parsers, simulation
├── desktop/        Platform launcher (LWJGL3/OpenAL), macOS shims
├── server/         Headless multiplayer simulation (no rendering)
├── shared/         Shared DTOs/interfaces between core and server
├── fdfparser/      ANTLR-generated FDF UI parser (separate module to allow regen)
└── jassparser/     ANTLR-generated JASS script parser
```

**Why separate parser modules?**  
ANTLR generates large Java source files from grammar definitions. Keeping them in dedicated modules avoids re-running the generator on every build and lets IDE tooling handle them cleanly.

---

## Entry Point

```
desktop/src/com/etheller/warsmash/desktop/DesktopLauncher.java
  └── main(String[] args)
        ├── Parse CLI args (-window, -loadfile, -ini, -nolog)
        ├── [macOS only] Set ShaderProgram.prependVertex/FragmentCode for GLSL 1.50
        ├── Build Lwjgl3ApplicationConfiguration
        │     ├── setOpenGLEmulation(GL32, 3, 2)  [macOS only]
        │     ├── setTitle, setWindowedMode / setFullscreenMode
        │     └── useVsync(true)
        └── new Lwjgl3Application(WarsmashGdxMultiScreenGame, config)
```

`WarsmashGdxMultiScreenGame` decides the first screen:
- If `-loadfile` arg: jump straight to `WarsmashGdxMenuScreen` and load the map
- Otherwise: show `WarsmashGdxMenuScreen` (main menu)

---

## Screen Flow

```
WarsmashGdxMultiScreenGame (ApplicationListener)
  │
  ├─ WarsmashGdxMenuScreen      (Screen + InputProcessor)
  │    └─ MenuUI                 manages FDF-based main menu, campaign UI
  │         └─ when map selected → WarsmashGdxMapScreen
  │
  └─ WarsmashGdxMapScreen       (Screen + InputProcessor)
       └─ War3MapViewer          loads terrain, units, JASS, runs simulation
            └─ MeleeUI           in-game HUD, command card, minimap
                 └─ MeleeToggleUI (swaps between melee UI states)
```

---

## Core Package Map

```
core/src/com/etheller/warsmash/
  ├── datasources/          DataSource abstraction + MPQ/CASC/Folder impls
  ├── parsers/
  │   ├── fdf/              GameUI, frame classes, FDF runtime
  │   ├── jass/             JASS interpreter, native functions, scope system
  │   └── w3x/              W3X map file parsers (w3e, w3i, wpm, doo, …)
  ├── units/                SLK/INI data table parsers + unit type data
  ├── util/                 WarsmashConstants, INI reader, math helpers
  └── viewer5/
      ├── gl/               WebGL wrapper, shader management, extensions
      ├── handlers/
      │   ├── blp/          BLP texture loader
      │   ├── mdx/          MDX model loader + renderer
      │   └── w3x/
      │       ├── environment/  Terrain, GroundTexture, CliffMesh, PathingGrid
      │       ├── simulation/   CSimulation, CUnit, CAbility, combat, AI
      │       ├── ui/           MeleeUI, MenuUI, HUD frames
      │       └── camera/       GameCameraManager
      ├── ModelViewer.java      Base viewer: resource loading, scene management
      ├── Scene.java            Abstract render scene
      ├── SimpleScene.java      Flat orthographic scene (UI)
      └── WorldScene.java       3D perspective scene (terrain + units)

desktop/src/com/etheller/warsmash/desktop/
  ├── DesktopLauncher.java
  └── audio/                OpenAL audio backend

core/src/mpq/               MPQ archive library (DrSuperGood)
core/src/com/hiveworkshop/rms/parsers/mdlx/  MDX/MDL parser
core/src/io/nayuki/flac/    FLAC decoder
```

---

## LibGDX Rendering Stack

```
LWJGL3Application
  └── OpenGL 3.2 Core (macOS) / OpenGL 2.0 compat (others)
       └── Gdx.gl  (GL20 interface)
            └── WebGL.java  (utility wrapper: shader cache, texture bind)
                 ├── ShaderProgram  (vertex + fragment GLSL)
                 ├── Scene.startFrame()   glViewport + glScissor via HdpiUtils
                 └── SpriteBatch         2D UI rendering
```

**HiDPI note:** On Retina displays in windowed mode, `Gdx.graphics.getWidth()` returns logical pixels (e.g. 1280) but `glViewport` needs physical pixels (e.g. 2560). Always use `HdpiUtils.glViewport()` and `HdpiUtils.glScissor()` — raw `gl.glViewport()` will render to the bottom-left quadrant only.

---

## Threading Model

Warsmash is **single-threaded** on the render thread.

- LibGDX calls `render()` on the GL thread at vsync frequency
- `War3MapViewer.update()` accumulates delta time and runs simulation ticks
- JASS trigger threads are cooperative, not OS threads — they run to completion inside the simulation tick
- No background loading: all asset loads are synchronous (see `ModelViewer.load()`)

---

## Key Constants

```java
// core/src/com/etheller/warsmash/util/WarsmashConstants.java
INPUT_HOTKEY_MODE = 0       // 0=data-driven, 1=QWER grid
ENABLE_DEBUG = false
SIMULATION_STEP_TIME = 1/20f  // 20 ticks per second
```

```ini
; core/assets/warsmash.ini
MaxPlayers=28       ; 16 for pre-1.27 patches
GameVersion=1       ; 1=TFT
InputHotkeyMode=0
```
