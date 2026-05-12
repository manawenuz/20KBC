# MVP Plan — 20,000 BC

## Goal

Validate the technology stack with a **playable vertical slice** in 2-3 weeks.  
The MVP is not a demo — it is an experiment that answers one question:

> **Which engine gives us the best simulation/rendering split, networking path, and iteration speed for a 20-30 player prehistoric RTS?**

---

## What the MVP Proves

| Question | How the MVP answers it |
|----------|----------------------|
| Can simulation run decoupled from the renderer? | game-core crate compiles and runs headless |
| Can the engine render 50+ units at 60fps on one map? | Stress test: spawn 50 workers, run 20Hz sim |
| Is RTS camera + selection + orders feasible? | Box-select, right-click move, command card stub |
| Can supply drain run in background? | Out-of-supply health decay logic in game-core |
| Can we ship a multiplayer test in week 4? | Lockstep-ready: deterministic tick, input log |

---

## MVP Scope

### Included

- **Terrain**: Flat grid, 64×64 tiles, each 2 world-units. Procedural tile coloring (grass/dirt/stone).
- **Camera**: RTS-style angled camera (~60° tilt). Pan (WASD + edge scroll) + zoom (scroll wheel).
- **Units**: Male Worker (Level 1). HP 100, move 1/s, melee attack 20, supply reserve 10.
- **Selection**: Left-click select single unit. Box-drag select multiple. Selected units show selection circle.
- **Orders**: Right-click ground → move order. Right-click resource → gather order.
- **Resources**: 3 Wood Piles, 3 Stone Outcrops on the map. Gather and return to base.
- **Resource HUD**: Top-left: Wood count, Stone count.
- **Supply system**: Workers deplete supply at 1/10s. No supply depot = health drains at 3×. Auto-heal 1HP/10s when supplied.
- **GAIA predator**: 1 wolf. Roams. Attacks workers that enter its territory radius (50 units). Pathfinds to target.
- **Day/night**: Simple ambient light lerp over a 10-min cycle. Units have reduced sight at night.
- **Simulation tick**: 20Hz fixed-step. Deterministic (seeded RNG). Input log written to file.

### Explicitly Out of Scope

- Buildings, training, tech tree
- Multiplayer (but input log enables replay = lockstep ready)
- Female workers, healers, supply wagon
- Ranged weapons, accuracy system
- Map generation
- Audio
- Campaign / menus beyond a "Start" button

---

## Architecture

```
20KBC/
├── game-core/          # Pure Rust crate — simulation only, zero engine deps
│   ├── src/
│   │   ├── simulation.rs      CSimulation, tick()
│   │   ├── unit.rs            CUnit, CBehavior trait
│   │   ├── behaviors/         move, gather, attack, idle, death
│   │   ├── pathfinding.rs     Grid A* (grid cell = 2 world units)
│   │   ├── supply.rs          supply drain, auto-heal
│   │   ├── resources.rs       CResourceNode
│   │   ├── gaia.rs            predator roam + territory attack
│   │   ├── player.rs          CPlayer (resources, food)
│   │   └── rng.rs             seeded deterministic RNG
│   └── Cargo.toml
│
├── spikes/
│   ├── godot-gdext/    # Godot 4 + gdext rendering client
│   ├── bevy/           # Bevy rendering client
│   └── fyrox/          # Fyrox rendering client
│
└── plans/              # This directory
```

Each spike crate depends on `game-core`. The engine provides:
- Window + render loop
- Asset loading (placeholder colored meshes, no MPQ)
- Input (mouse clicks, keyboard)
- Camera
- HUD rendering

game-core provides:
- All game state
- Deterministic simulation at 20Hz
- Query API for renderer (`iter_units()`, `iter_resources()`, etc.)
- Command intake (`issue_order(unit_id, order)`)

---

## Success Criteria

At the end of the 2-week spike period, each prototype must:

1. Show a flat terrain with colored tiles
2. Spawn 10 workers + 1 wolf
3. Allow click-to-select + right-click-to-move for workers
4. Workers auto-gather from a resource node when ordered
5. Wolf patrols and attacks workers in territory
6. Supply drain kills unsupplied workers over time
7. Day/night ambient cycle visible
8. Simulation runs at 20Hz without frame-rate coupling
9. Input log written to `replay.bin`

---

## Timeline

| Week | Milestone |
|------|-----------|
| 1 | game-core crate: simulation, pathfinding, behaviors, supply, GAIA |
| 1 | All 3 engine spikes: terrain + camera rendering |
| 2 | All 3 engine spikes: units, selection, orders, HUD |
| 2 | Stress test: 50 units, measure frame time |
| 2 | Team decision: pick 1 engine |
| 3 | Winning engine: multiplayer networking stub (lockstep inputs) |

---

## Engine Decision Rubric

After the 2-week spike, score each engine:

| Criterion | Weight |
|-----------|--------|
| game-core integration friction | 25% |
| Rendering quality & iteration speed | 20% |
| UI builder quality (command card, minimap) | 20% |
| Networking path (lockstep support) | 20% |
| Rust ergonomics & compile time | 15% |
