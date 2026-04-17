---
sidebar_position: 5
---

# System Dependencies



## Dependency Graph

Systems at the top must be built first. Arrows show "depends on" relationships.

```
FOUNDATION LAYER (build first)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Map/Terrain ──────────────────────────────────────┐
  ├── Tile grid, walkability, heightmap             │
  └── Slope calculation                             │
                                                    │
  Movement System ──────────────────────────────────┤
  ├── A* / flow field pathfinding                   │
  ├── Slope speed modifiers                         │
  ├── River crossing rules                          │
  └── Terrain obstacle avoidance                    │
                                                    │
  Entity System ────────────────────────────────────┘
  ├── Entity ID, ownership (player/GAIA)
  ├── Position, health, state machine
  └── Component storage (World struct)

ECONOMY LAYER (depends on: foundation)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Resource System
  ├── 8 resource types + derived (supply, rope)
  ├── Carry capacity per unit
  └── Resource nodes on map (trees, stones, clay, grass)
        │
        ▼
  Supply System ◄────── critical: units die without supply
  ├── food + water → supply (auto-conversion)
  ├── Supply cover range (radius from depot/base)
  ├── Shared stockpile across connected depots
  └── Unit consumption rate (1/10s) + health drain outside range
        │
        ▼
  Gathering Actions
  ├── Auto-gather within supply range
  ├── Woodcutting (depends on tree stages)
  ├── Stone cutting (depends on big stone)
  ├── Fishing (depends on water bodies + spear)
  └── Delivery to nearest depot

COMBAT LAYER (depends on: foundation + economy)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Unit Stats
  ├── Health, armor, damage, attack speed
  ├── Attack type (melee / ranged)
  ├── Movement speed (normal / attack)
  └── Sight range
        │
        ▼
  Combat Resolution
  ├── Melee: damage - armor = HP loss
  ├── Ranged: projectile speed + accuracy grid (2/9 or 1/5 random)
  ├── Height/slope bonus to range
  └── Friendly fire (projectile hits unit at impact location)
        │
        ▼
  Unit Upgrade System
  ├── Level 1 → 2.1 (Axes) or 2.2 (Spear)
  ├── Level 2.x → 3.1 (Axes+Spear throw) or 3.2 (Axes/Spear+Sling)
  ├── XP from combat (0-30, 30-70, 70+)
  └── Ability switching (costs resources, out of combat only)

BUILDING LAYER (depends on: economy + combat)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Building Placement
  ├── Must be within supply cover range of Home Base
  ├── Terrain validation (flat enough, not water)
  └── Resources deducted immediately on construction start
        │
        ├──► Home Base (8x8x3)
        │    ├── Train workers, healer, supply wagon
        │    ├── Research upgrades
        │    ├── 20 population cap
        │    ├── Mobile/settle mode
        │    └── Loot on destruction (80% resources)
        │
        ├──► Supply Depot (2x2x2)
        │    ├── Extends supply cover range
        │    ├── Must overlap with Home Base range
        │    └── Loot on destruction (20% resources)
        │
        ├──► Farm (10x10x1)
        │    ├── Produces food (60/30s per worker, max 2)
        │    ├── Season-dependent: barley (spring/summer), peas (fall), disabled (winter)
        │    └── Depends on: season system (Phase 3)
        │
        ├──► Water Well (1x1x2) — unlimited water source
        │
        ├──► Walls (wooden nx1x2, stone nx2x3)
        │    ├── Block movement
        │    ├── Stone walls: units can stand on top, height range bonus
        │    └── Destructible: fire (wood only), rope, melee
        │
        ├──► Towers (Scout 2x2x6, Battle 4x4x6)
        │    ├── Garrison slots (1-2 units)
        │    ├── Height adds to unit range
        │    └── Battle tower: AOE melee attack (stones from top)
        │
        └──► Boat
             ├── Mobile platform on water
             ├── Capacity: 6 units OR 50 fish (trade-off)
             ├── Requires rope to build
             └── Vacant boats can be captured by enemies

TECH TREE (depends on: buildings + unit upgrades)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  (No prerequisites)
  ├── Skinning → gather skin from animals
  │   ├── Skin Bag (per unit) → +20% carry capacity
  │   └── Clothes → -20% supply consumption
  ├── Rope → 5 grass = 1 rope, enables sling/boat/wall-destruction
  ├── Stone Tool → enables Axes/Spear unit upgrades
  │   ├── Stone Axe Upgrade → +20% axe damage
  │   └── Stone Spear Upgrade → +20% spear damage
  ├── Irrigation → +30% farm generation rate
  ├── Supply Depot Upgrade → +30% supply cover range
  └── Ladder → units can climb enemy walls (3s)

WORLD SYSTEMS (depends on: all above)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Season/Climate System
  ├── 4 seasons cycle every 10 min
  ├── Climate changes 4x per season (probability tables)
  ├── Affects: movement speed, sight, supply consumption,
  │   resource reproduction, animal spawning, disaster chance
  └── 4 geographic zones per region modify probabilities
        │
        ▼
  GAIA Animals
  ├── 5 species with herd behavior
  ├── Reproduction, maturation, territory
  ├── Predator/prey interactions
  ├── Herd respawn after wipeout
  └── Affected by season/climate (population modifiers)
        │
        ▼
  Natural Disasters
  ├── River flood → destroy riverside content
  ├── Wildfire → 30% of GAIA trees become dead trees
  ├── Famine → all reproduction stops
  ├── Fog → sight -20%
  └── Wind → visual only
        │
        ▼
  Day/Night Cycle
  ├── Morning/Noon/Afternoon/Night
  ├── Sun rotation + dynamic shadows
  └── Night: sight -20% for all units

MULTIPLAYER (depends on: deterministic simulation)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Deterministic Simulation
  ├── Fixed-point math (no f32 in game-core)
  ├── Seeded RNG
  ├── BTreeMap (no HashMap)
  └── Fixed timestep (20 ticks/sec)
        │
        ▼
  Lockstep Networking (1v1, 2v2)
  ├── Exchange inputs per tick
  ├── Input delay buffer
  └── Desync detection (state checksums)
        │
        ▼
  Server-Authoritative (Survival 20-30 players)
  ├── Dedicated server runs simulation
  ├── State snapshots to clients
  └── Client-side prediction
```


## Build Order (What to Implement When)

### MVP (Phase 1) — Minimum viable subset
1. Map (flat grid)
2. Entity system (units with position/health)
3. Movement (A* pathfinding)
4. Resources (wood, stone, food — simplified)
5. Supply system (simplified — global stockpile)
6. Buildings (Home Base, Supply Depot, Farm)
7. Combat (melee only)
8. Unit upgrades (2 levels)
9. Basic AI

### Full Systems (Phase 2) — Complete design doc
10. All 8 resource types + derived
11. All 8 building types
12. All unit types (male, female, healer, supply wagon)
13. Full upgrade path (5 levels)
14. Full tech tree (10 upgrades)
15. Procedural map generation
16. Better AI

### World (Phase 3) — Environment simulation
17. Season/climate system + probability tables
18. Day/night cycle
19. GAIA animals (5 species + herd behavior)
20. Natural disasters (5 types)
21. Terrain slopes + water mechanics

### Multiplayer (Phase 4) — Networking
22. Deterministic simulation
23. Lockstep networking
24. Server-authoritative (survival)
25. Ranked system


## Cross-Cutting Concerns

These affect multiple systems and should be designed early:

| Concern | Affected Systems | Decision |
|---------|-----------------|----------|
| Tile vs continuous coords | Map, movement, buildings, resources | Hybrid: tiles for buildings/resources, continuous for units |
| Data-driven stats | All units, buildings, tech, climate | TOML files loaded at startup, no hardcoded values |
| Entity ownership | Units, buildings, resources | Player ID on every entity, GAIA = player 0 |
| Fog of war | Rendering, combat, AI | Tile-based visibility per player |
| Determinism | All simulation code | f32 OK in Phase 1-3, fixed-point in Phase 4 |
