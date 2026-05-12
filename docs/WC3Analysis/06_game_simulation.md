# 06 — Game Simulation

## Simulation vs. Rendering

`CSimulation` is the game world. It knows nothing about OpenGL, textures, or screens. It works purely in world-space float coordinates. The renderer reads positions/states from `CSimulation` objects and draws them.

This clean separation is the right architecture to carry into a Rust rewrite: `game-core` (pure simulation) vs. `game-client` (rendering).

---

## Map Loading Sequence

```
War3MapViewer.loadMap(mapPath)
  │
  ├─ 1. Open map as MPQ
  │      War3Map map = new War3Map(dataSource, mapPath)
  │
  ├─ 2. Read sub-files
  │      War3MapW3i info     = map.readMapInformation()
  │      War3MapW3e terrain  = map.readEnvironment()
  │      War3MapWpm pathing  = map.readPathing()
  │      War3MapDoo doodads  = map.readDoodads(info)
  │      War3MapUnitsDoo units = map.readUnits(info)
  │      War3MapW3r regions  = map.readRegions()
  │
  ├─ 3. Load global data tables (from MPQ, not map)
  │      UnitData.slk, ItemData.slk, AbilityData.slk,
  │      UpgradeData.slk, DestructableData.slk
  │
  ├─ 4. Build terrain renderer
  │      new Terrain(terrain, dataSource, viewer)
  │
  ├─ 5. Create simulation
  │      CSimulation sim = new CSimulation(config, unitData, ...)
  │
  ├─ 6. Populate world from map data
  │      for each unit in units.doo:
  │          sim.createUnit(typeId, x, y, facing, player)
  │      for each doodad in doo:
  │          if (isDestructable): sim.createDestructable(...)
  │          else:                spawn MDX instance at position
  │
  ├─ 7. Parse + execute JASS
  │      JassProgram program = parse(common.j, blizzard.j, war3map.j)
  │      program.initialize()   ← runs InitGlobals
  │      globalScope.callFunction("main")  ← registers triggers, creates units
  │
  └─ 8. Start render loop
```

---

## CSimulation

```
core/src/com/etheller/warsmash/viewer5/handlers/w3x/simulation/CSimulation.java
```

**State:**
```java
List<CUnit>        units
List<CDestructable> destructables
List<CItem>        items
List<CPlayer>      players      // [0..27], 27 = neutral passive
CUnitData          unitData
CAbilityData       abilityData
CUpgradeData       upgradeData
CPathfindingProcessorList pathfinders   // one per player
CHandleIdAllocator handleAllocator
int                gameTurnTick
long               gameTimeMs
```

**Tick rate:** `1/20f` seconds (20 ticks/second = 50ms per tick)

**update() per tick:**
```java
void update() {
    gameTurnTick++;
    gameTimeMs += 50;

    // 1. Fire elapsed-time triggers
    for (CTimer timer : activeTimers) timer.update(this);

    // 2. Update all units
    for (CUnit unit : units) unit.update(this);

    // 3. Update projectiles in flight
    for (CUnitAttackMissile missile : missiles) missile.update(this);

    // 4. Drain pending JASS threads
    for (JassThread thread : pendingJassThreads) {
        globalScope.runThread(thread);
    }

    // 5. Process pending unit orders (orders issued mid-tick)
    processPendingOrders();

    // 6. Update destructable regeneration, etc.
    for (CDestructable d : destructables) d.update(this);
}
```

---

## CUnit

```
core/src/com/etheller/warsmash/viewer5/handlers/w3x/simulation/CUnit.java
```

**Key fields:**
```java
String    typeId          // "hpea" = Peasant, "hfoo" = Footman, etc.
CUnitType unitType        // stats looked up from UnitData.slk
CPlayer   owner
float     x, y            // world position (1 tile = 128 units)
float     facing          // radians, 0 = east
float     currentLife
float     currentMana
int       handleId
CBehavior currentBehavior  // what the unit is currently doing
List<CAbility> abilities
float     moveSpeed        // current speed (affected by buffs)
float     attackCooldown   // time until next attack
boolean   invisible, invulnerable, paused
CUnit     orderTarget      // current attack/follow target
float[]   orderTargetPos   // move target position
```

### Unit Type Data (CUnitType)

Loaded from SLK tables via `CUnitData.getUnitType(typeId)`:

```java
class CUnitType {
    String  id, name, model    // "hpea", "Peasant", "units/human/Peasant/Peasant.mdx"
    float   maxLife, maxMana
    float   armor
    float   moveSpeed
    float   attackRange1       // 0 = melee
    float   attackDamageBase1, attackDamageNumberOfDice1, attackDamageSidesPerDie1
    float   attackCooldown1
    int     attackTargetsAllowed  // bitmask
    float   sightRadiusDay, sightRadiusNight
    float   acquisitionRange
    int     foodCost
    List<String> abilities     // ability IDs for this unit type
    List<String> trains        // unit type IDs this unit can train
    List<String> researches    // upgrade IDs
    String  soundSet
    String  selectionScale
    float   collisionSize
    int     buildTime
    int     goldCost, lumberCost
}
```

---

## Behavior System

Each unit has exactly one `CBehavior` (current action). Behaviors are replaced, not stacked.

```
core/src/com/etheller/warsmash/viewer5/handlers/w3x/simulation/behaviors/
```

```java
interface CBehavior {
    CBehavior update(CUnit unit, CSimulation sim);  // returns next behavior (or this)
    boolean interruptable();
    String getOrderId();
}
```

**Built-in behaviors:**

| Class | Triggers when | Description |
|-------|-------------|-------------|
| `CBehaviorStop` | order "stop" | Idle, acquires targets automatically |
| `CBehaviorMove` | order "move" | Pathfind to position |
| `CBehaviorAttack` | order "attack" | Attack specific target |
| `CBehaviorAttackMove` | order "attack-move" | Move, attack anything in range |
| `CBehaviorPatrol` | order "patrol" | Patrol between two points |
| `CBehaviorHoldPosition` | order "holdposition" | Fight but don't move |
| `CBehaviorGather` | order "harvest" | Move to resource, gather, return |
| `CBehaviorRepair` | order "repair" | Move to building, repair |
| `CBehaviorTrain` | button command | Train unit inside building |
| `CBehaviorResearch` | button command | Research upgrade inside building |
| `CBehaviorBuild` | order "build" | Worker moves to site, constructs |
| `CBehaviorStun` | spell effect | Cannot act |
| `CBehaviorDeath` | HP <= 0 | Play death animation, become corpse |

**Behavior update loop:**
```java
// CUnit.update()
CBehavior next = currentBehavior.update(this, simulation);
if (next != currentBehavior) {
    currentBehavior = next;
}
```

### Movement (CBehaviorMove)

```java
// Each tick:
float[] path = pathfinder.getNextWaypoint(unit.x, unit.y, targetX, targetY);
float dx = path[0] - unit.x, dy = path[1] - unit.y;
float dist = sqrt(dx*dx + dy*dy);
float step = unit.moveSpeed * TICK_TIME;
if (dist <= step) {
    unit.x = path[0]; unit.y = path[1];
    advanceWaypoint();
} else {
    unit.x += dx/dist * step;
    unit.y += dy/dist * step;
    unit.facing = atan2(dy, dx);
}
```

### Pathfinding

```
core/src/com/etheller/warsmash/viewer5/handlers/w3x/simulation/pathfinding/
  CPathfindingProcessor.java
```

Simple grid-based A* on the WPM pathing grid. One processor per player (allows player-specific vision in future).

Cell size = 32 world units (4× terrain tile resolution).  
Uses 8-directional movement with diagonal penalty.

---

## Combat

```
core/src/com/etheller/warsmash/viewer5/handlers/w3x/simulation/combat/
```

### Attack Flow

```
CBehaviorAttack.update()
  ├─ Move into attack range (if not already)
  ├─ Face target
  ├─ Wait for attackCooldown
  ├─ Play attack animation
  ├─ At animation "launch point":
  │     if (projectile): launch CUnitAttackMissile
  │     else:            apply instant damage
  └─ Reset attackCooldown
```

### Damage Formula

```java
float damage = roll(base, dice, sides) + bonuses;
float reduced = max(0, damage - target.armor * armorReduction(target.armorType));
target.currentLife -= reduced;
```

Armor reduction by attack type (lookup table):
```
Attack\Defense  Normal  Medium  Large  Fort  Magic  Hero
Normal          100%    75%     50%    35%   100%   100%
Pierce          150%    75%     100%   35%   100%   100%
Siege           100%    50%     100%   150%  100%   100%
Magic           100%    75%     75%    35%   175%   100%
Chaos           100%    100%    100%   100%  100%   100%
Spells          100%    100%    100%   100%  0%     100%
```

### Projectiles

`CUnitAttackMissile`:
- Spawns an MDX effect model at the attacker's hand attachment point
- Moves toward the target at `missileSpeed` units/second
- On arrival: apply damage, spawn impact effect, remove missile model
- If target dies in flight: missile disappears harmlessly (no splash on corpse)

---

## Players

```java
class CPlayer {
    int      id           // 0-27
    String   name
    int      gold, lumber, food, foodUsed
    Race     race         // HUMAN, ORC, UNDEAD, NIGHTELF, NEUTRAL
    Alliance[] alliances  // per player: ally/enemy/neutral
    ControlType controlType  // HUMAN, COMPUTER, NEUTRAL, RESCUABLE
}
```

**Player slots 0-23:** actual players  
**Player 24:** neutral hostile (GAIA creeps)  
**Player 25:** neutral passive  
**Player 26:** neutral aggressive  
**Player 27:** neutral extra  

---

## Unit Orders (JASS API)

Units receive orders via native functions, which translate to behavior changes:

| JASS Function | Java | Behavior |
|--------------|------|----------|
| `IssueImmediateOrder(u,"stop")` | `unit.order(STOP, null, 0,0)` | CBehaviorStop |
| `IssuePointOrder(u,"move",x,y)` | `unit.order(MOVE, null, x,y)` | CBehaviorMove |
| `IssueTargetOrder(u,"attack",t)` | `unit.order(ATTACK, target, 0,0)` | CBehaviorAttack |
| `IssueTargetOrder(u,"harvest",t)` | `unit.order(GATHER, target, 0,0)` | CBehaviorGather |
| `IssuePointOrder(u,"patrol",x,y)` | `unit.order(PATROL, null, x,y)` | CBehaviorPatrol |

---

## Fog of War

Each player has a `CVisionManager` that tracks which cells are:
- `VISIBLE` — in a unit's current sight radius
- `EXPLORED` — seen at some point (partial fog / greyed out)
- `UNEXPLORED` — never seen (black)

Sight radius computation respects day/night (`sightRadiusDay` vs `sightRadiusNight`) but does not yet implement slope-based line-of-sight (the terrain height is not factored in).
