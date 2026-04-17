---
sidebar_position: 99
---

# Project Tasks

> Auto-generated from `.taskmaster/tasks/tasks.json`
> Last updated: 2026-04-17

## Progress

```
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 0% (0/21)
```

| Status | Count |
|--------|-------|
| ✅ Done | 0 |
| 🔄 In Progress | 0 |
| ⬚ Pending | 21 |
| **Total** | **21** |

---

## Cross-Phase (0/21)

| | ID | Task | Status | Priority | Depends On |
|---|---|------|--------|----------|------------|
| ⬚ | 1 | **Godot + gdext Engine Spike** | pending | 🔴 high | — |
| ⬚ | 2 | **Bevy Engine Spike** | pending | 🔴 high | — |
| ⬚ | 3 | **Engine Decision & Workspace Setup** | pending | 🔴 high | 1, 2 |
| ⬚ | 4 | **Game-Core Foundation Types** | pending | 🔴 high | 3 |
| ⬚ | 5 | **A* Pathfinding System** | pending | 🔴 high | 4 |
| ⬚ | 6 | **Terrain Renderer & Camera** | pending | 🔴 high | 3 |
| ⬚ | 7 | **Unit Selection & Commands** | pending | 🔴 high | 6 |
| ⬚ | 8 | **Unit State Machine & Movement** | pending | 🔴 high | 4, 5 |
| ⬚ | 9 | **Resource System** | pending | 🔴 high | 8 |
| ⬚ | 10 | **Supply System** | pending | 🔴 high | 9 |
| ⬚ | 11 | **HUD, Resource Display & Minimap** | pending | 🟡 medium | 6, 9 |
| ⬚ | 12 | **Building System** | pending | 🔴 high | 10 |
| ⬚ | 13 | **Building Placement UI & Fog of War** | pending | 🟡 medium | 6, 12 |
| ⬚ | 14 | **Combat System** | pending | 🔴 high | 8 |
| ⬚ | 15 | **Unit Upgrades & Tech Tree (Phase 1 Subset)** | pending | 🔴 high | 12, 14 |
| ⬚ | 16 | **Combat Visuals & Placeholder Audio** | pending | 🟡 medium | 7, 14 |
| ⬚ | 17 | **Two-Player Hotseat & Game Setup** | pending | 🔴 high | 13, 16 |
| ⬚ | 18 | **Player Ownership & Win Condition** | pending | 🔴 high | 12, 14 |
| ⬚ | 19 | **Basic AI Opponent** | pending | 🔴 high | 18 |
| ⬚ | 20 | **Save/Load & Game Speed Controls** | pending | 🟡 medium | 17 |
| ⬚ | 21 | **Data-Driven Stats (TOML Config)** | pending | 🟡 medium | 4 |

<details>
<summary><strong>#1 Godot + gdext Engine Spike</strong> — subtasks (0/7)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Set up Godot 4 project with gdext Rust bindings | pending |
| ⬚ | Render 64x64 tile-based terrain from heightmap data | pending |
| ⬚ | Implement camera: pan/zoom/rotate | pending |
| ⬚ | Spawn 100 units with A* pathfinding on tile grid | pending |
| ⬚ | Click-to-select (single + box) and right-click-to-move | pending |
| ⬚ | Basic UDP send/receive between two instances | pending |
| ⬚ | Write evaluation report (score 1-5 per criterion) | pending |

</details>

<details>
<summary><strong>#2 Bevy Engine Spike</strong> — subtasks (0/7)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Set up Bevy project with ECS architecture | pending |
| ⬚ | Render 64x64 tile terrain from heightmap | pending |
| ⬚ | Implement camera: pan/zoom/rotate | pending |
| ⬚ | Spawn 100 units with A* pathfinding | pending |
| ⬚ | Click-to-select and right-click-to-move | pending |
| ⬚ | Basic UDP send/receive between two instances | pending |
| ⬚ | Write evaluation report | pending |

</details>

<details>
<summary><strong>#3 Engine Decision & Workspace Setup</strong> — subtasks (0/5)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Compare spike reports, document decision | pending |
| ⬚ | Create Rust workspace with Cargo.toml (game-core, game-client) | pending |
| ⬚ | Set up GitHub Actions CI | pending |
| ⬚ | Create game-data/ with TOML config structure | pending |
| ⬚ | Write CONTRIBUTING.md | pending |

</details>

<details>
<summary><strong>#4 Game-Core Foundation Types</strong> — subtasks (0/6)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Map struct: 2D tile grid with terrain type and walkability | pending |
| ⬚ | TilePosition and WorldPosition types with conversions | pending |
| ⬚ | EntityId, PlayerId, Entity struct with components | pending |
| ⬚ | Unit struct: position, health, speed, state machine | pending |
| ⬚ | World struct: entities, map, players, tick counter | pending |
| ⬚ | Unit tests for all core types | pending |

</details>

<details>
<summary><strong>#5 A* Pathfinding System</strong> — subtasks (0/5)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Implement A* algorithm on tile grid | pending |
| ⬚ | Movement cost per tile type | pending |
| ⬚ | Path caching and request batching | pending |
| ⬚ | Benchmark: 200 paths on 64x64 under 16ms | pending |
| ⬚ | Tests: correctness, obstacles, no-path | pending |

</details>

<details>
<summary><strong>#6 Terrain Renderer & Camera</strong> — subtasks (0/4)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Terrain mesh from Map data | pending |
| ⬚ | Camera pan (WASD + edge scroll) | pending |
| ⬚ | Camera zoom (scroll wheel, bounded) | pending |
| ⬚ | Camera rotate (middle mouse) | pending |

</details>

<details>
<summary><strong>#7 Unit Selection & Commands</strong> — subtasks (0/5)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Render units as colored capsules | pending |
| ⬚ | Single-click select with indicator | pending |
| ⬚ | Box/drag select for groups | pending |
| ⬚ | Right-click-to-move command | pending |
| ⬚ | Selected unit info panel | pending |

</details>

<details>
<summary><strong>#8 Unit State Machine & Movement</strong> — subtasks (0/5)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | State machine: Idle↔Moving transitions | pending |
| ⬚ | Path following with configurable speed | pending |
| ⬚ | Gathering and Attacking state stubs | pending |
| ⬚ | Basic unit collision avoidance | pending |
| ⬚ | Tests: transitions, movement, arrival | pending |

</details>

<details>
<summary><strong>#9 Resource System</strong> — subtasks (0/6)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Resource enum and ResourceNode struct | pending |
| ⬚ | Place resource nodes on map | pending |
| ⬚ | Gathering action: walk, timer, pickup | pending |
| ⬚ | Delivery to nearest depot | pending |
| ⬚ | Carry capacity per unit type | pending |
| ⬚ | Tests: gather cycle, limits, depletion | pending |

</details>

<details>
<summary><strong>#10 Supply System</strong> — subtasks (0/6)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Supply conversion: food + water → supply | pending |
| ⬚ | Unit consumption: 1 supply per 10s | pending |
| ⬚ | Supply reserve per unit type | pending |
| ⬚ | Health drain when no supply | pending |
| ⬚ | Auto-heal when in supply range | pending |
| ⬚ | Tests: conversion, consumption, drain, heal | pending |

</details>

<details>
<summary><strong>#11 HUD, Resource Display & Minimap</strong> — subtasks (0/4)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Resource counters HUD | pending |
| ⬚ | Render resource nodes as colored boxes | pending |
| ⬚ | Minimap with terrain/units/resources | pending |
| ⬚ | Unit carrying indicator | pending |

</details>

<details>
<summary><strong>#12 Building System</strong> — subtasks (0/7)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Building struct with all fields | pending |
| ⬚ | Home Base: train workers, pop cap, supply | pending |
| ⬚ | Supply Depot: cover range, shared stockpile | pending |
| ⬚ | Farm: food generation, max workers | pending |
| ⬚ | Construction: resource check, deduct, timer | pending |
| ⬚ | Supply cover range calculation | pending |
| ⬚ | Tests: placement, construction, coverage, pop cap | pending |

</details>

<details>
<summary><strong>#13 Building Placement UI & Fog of War</strong> — subtasks (0/4)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Placement ghost with validity coloring | pending |
| ⬚ | Construction progress bar | pending |
| ⬚ | Fog of war (unexplored/fogged/visible) | pending |
| ⬚ | Supply range indicator on placement | pending |

</details>

<details>
<summary><strong>#14 Combat System</strong> — subtasks (0/7)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Melee attack resolution | pending |
| ⬚ | Ranged attack: projectile with travel time | pending |
| ⬚ | Accuracy grid (60% center) | pending |
| ⬚ | Friendly fire on impact location | pending |
| ⬚ | Unit death and removal | pending |
| ⬚ | Attack-move command | pending |
| ⬚ | Tests: damage, accuracy, death | pending |

</details>

<details>
<summary><strong>#15 Unit Upgrades & Tech Tree (Phase 1 Subset)</strong> — subtasks (0/6)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Upgrade path: L1→L2.1 Axes | pending |
| ⬚ | Upgrade path: L2.1→L3.1 Axes+Spear throw | pending |
| ⬚ | Stat changes on upgrade | pending |
| ⬚ | Tech tree: 4 upgrades with prerequisites | pending |
| ⬚ | Research queue at Home Base | pending |
| ⬚ | Tests: prerequisites, stats, research | pending |

</details>

<details>
<summary><strong>#16 Combat Visuals & Placeholder Audio</strong> — subtasks (0/5)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Health bars above units | pending |
| ⬚ | Melee attack animation | pending |
| ⬚ | Ranged projectile rendering | pending |
| ⬚ | Death effect | pending |
| ⬚ | Placeholder beep sounds | pending |

</details>

<details>
<summary><strong>#17 Two-Player Hotseat & Game Setup</strong> — subtasks (0/4)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Player switching with fog swap | pending |
| ⬚ | Game setup screen | pending |
| ⬚ | Victory/defeat screens | pending |
| ⬚ | Performance: 200 units at 60fps | pending |

</details>

<details>
<summary><strong>#18 Player Ownership & Win Condition</strong> — subtasks (0/5)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | PlayerId on all entities | pending |
| ⬚ | Win condition: enemy bases destroyed | pending |
| ⬚ | Loot: 80% resources on base destruction | pending |
| ⬚ | Starting conditions per player | pending |
| ⬚ | Tests: ownership, win, loot | pending |

</details>

<details>
<summary><strong>#19 Basic AI Opponent</strong> — subtasks (0/7)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | AI decision loop (eval state, choose action) | pending |
| ⬚ | AI gather: assign workers to resources | pending |
| ⬚ | AI build: place depot and farm | pending |
| ⬚ | AI train: workers up to pop cap | pending |
| ⬚ | AI upgrade: research and upgrade units | pending |
| ⬚ | AI attack: attack-move toward enemy base | pending |
| ⬚ | Difficulty scaling: Easy 0.5x, Normal 1x | pending |

</details>

<details>
<summary><strong>#20 Save/Load & Game Speed Controls</strong> — subtasks (0/4)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | Serialize World to file (save) | pending |
| ⬚ | Deserialize World from file (load) | pending |
| ⬚ | Speed controls: pause/1x/2x | pending |
| ⬚ | Performance profiling and optimization | pending |

</details>

<details>
<summary><strong>#21 Data-Driven Stats (TOML Config)</strong> — subtasks (0/6)</summary>

| | Subtask | Status |
|---|---------|--------|
| ⬚ | game-data/units.toml with all unit stats | pending |
| ⬚ | game-data/buildings.toml with all building stats | pending |
| ⬚ | game-data/resources.toml with amounts and times | pending |
| ⬚ | game-data/tech_tree.toml with costs and effects | pending |
| ⬚ | Config loader in game-core | pending |
| ⬚ | Validate all values against design doc | pending |

</details>

---

