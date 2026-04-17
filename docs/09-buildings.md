# Buildings

[Back to Index](./index.md) | [Prev: Units](./08-units.md) | [Next: Tech Tree](./10-tech-tree.md)

---

## Common Building Rules

- Most buildings can only be built within the **supply cover range** of a home base
- If the home base moves and a building falls outside supply cover range, the building becomes **vacant** (belongs to GAIA)
- If a player builds/deploys a home base in range of a vacant building, that player **takes ownership**

---

## Home Base

The central building of any settlement. Serves as training center, storage, and command hub.

### Functions

- Train male workers
- Train female workers
- Train healers
- Train supply wagons
- Acts as a supply depot
- Upgrade to higher tier
- Upgrade workers
- Provides **20 population cap**

### Stats

| Stat | Value |
|------|-------|
| Size | 8x8x3 |
| Health | 500 |
| Armor | 40 |
| Sight | 90 |
| Supply cover range | X radius |
| Supply consumption | 1/20s |
| Build time | X s |
| Speed (mobile) | 0.5/s |
| Required: Wood | X |
| Required: Stone | X |
| Required: Clay | X |

### Loot on Destruction

If a player destroys another player's home base, they receive **80% of total gathered resources**.

### Mobile / Settle Mode

The home base can switch between settled and mobile modes for migration (especially useful in survival mode).

- **Unpack (Mobile mode)**: Home base converts to a large supply wagon
  - Supply cover range reduced by **50%**
  - Unpack time: **20s**
- **Deploy (Settle mode)**: Convert back to settled home base
  - Deploy time: **20s**

---

## Farm

Produces food through agriculture.

### Crop Seasons

| Season | Crop | Generation Rate |
|--------|------|----------------|
| Spring | Barley | 60 food / 30s per worker |
| Summer | Barley | 60 food / 30s per worker |
| Fall | Peas | 60 food / 30s per worker |
| Winter | -- | **Not working** |

### Properties

- Max **2 workers** per farm
- Can only be built within supply cover range of home base

### Stats

| Stat | Value |
|------|-------|
| Size | 10x10x1 |
| Health | 200 |
| Armor | 40 |
| Sight | 50 |
| Supply consumption | 1/20s (only when farm is working) |
| Build time | X s |
| Required: Wood | X |
| Required: Stone | X |

---

## Supply Depot

Stores gathered resources and distributes supply to nearby units.

### Functions

- Store all gathered resources
- Convert food + water into supply automatically
- **Total supply and resources are shared** between all connected supply depots
- Workers deliver to the **nearest** supply depot

### Delivery Time

`Distance * Movement Speed` (e.g., 30 units away at 1/s speed = 30 seconds)

### Supply Cover Rules

- Units out of range with no supply reserve: lose health at `supply consumption * 3` rate
- Auto-heal stops for units out of range

### Loot on Destruction

If destroyed, attacker receives **20% of total gathered resources**.

### Stats

| Stat | Value |
|------|-------|
| Size | 2x2x2 |
| Health | 200 |
| Armor | 40 |
| Sight | 50 |
| Supply cover range | X radius |
| Supply consumption | 1/20s |
| Build time | X s |
| Required: Wood | X |
| Required: Stone | X |
| Required: Clay | X |

### Placement Rules

- Can be built **inside or outside** home base supply range
- Supply cover range of depot **must overlap** with home base supply range, otherwise it becomes inactive (units cannot deliver to it)

---

## Water Well

An unlimited water source building.

| Stat | Value |
|------|-------|
| Size | 1x1x2 |
| Gathering rate | 5 water / 5s per worker |
| Health | 100 |
| Armor | 5 |
| Sight | 50 |
| Supply consumption | 1/20s |
| Build time | X s |
| Required: Wood | X |
| Required: Stone | X |
| Required: Clay | X |

- Must be built within home base supply cover range
- Especially useful in desert/dry climates

---

## Walls

Defensive structures that block enemy movement.

### Wooden Wall

| Stat | Value |
|------|-------|
| Size | nx1x2 |
| Health | 200 |
| Armor | 20 |
| Sight | 50 |
| Height | 2 units |
| Supply consumption | 1/20s |
| Build time | X s |
| Required: Wood | X |
| Required: Stone | X |
| Required: Clay | X |

**Properties:**
- **No one can stand on top** of this wall
- Can be destroyed by: fire, axes, spear
- Must be built within supply cover range

### Stone Wall

| Stat | Value |
|------|-------|
| Size | nx2x3 |
| Health | 400 |
| Armor | 40 |
| Sight | 50 |
| Height | 3 units |
| Supply consumption | 1/20s |
| Build time | X s |
| Required: Wood | X |
| Required: Stone | X |
| Required: Clay | X |

**Properties:**
- **Units can stand on top** of it (garrison)
- Enemy can use **wooden ladders** to climb and fight on top
- **Height increases range** for units standing on the wall
- Must be built within supply cover range

### Wall Height & Range Bonus

Walls (and towers) grant a range bonus to units on top, equal to the structure's height. This is the same mechanic as slope range impact.

---

## Towers

Defensive structures that provide height advantage and ranged attacks.

### Defence Tower A (Scout Tower)

| Stat | Value |
|------|-------|
| Size | 2x2x6 |
| Health | 200 |
| Armor | 5 |
| Height | 6 units |
| Build time | X s |
| Required: Wood | X |
| Required: Stone | X |
| Required: Clay | X |

**Properties:**
- Requires **1 unit** to garrison for activation
- **No melee attack** -- only ranged attack based on garrison unit's weapon
- Range enhancement: `unit range + tower height`
- Can be destroyed by **fire** or **rope**
- Must be built within supply cover range

### Defence Tower B (Battle Tower)

| Stat | Value |
|------|-------|
| Size | 4x4x6 |
| Health | 500 |
| Armor | 20 |
| Height | 4 units |
| Garrison size | 2 units |
| Melee attack | 50 per attack (AOE, radius X) -- throws stones from top |
| Ranged attack | Depends on garrison units (spear/sling) |
| Attack priority | Melee (when enemy is close) > Range |
| Melee attack range | 1 |
| Melee attack speed | 1/2s |
| Build time | X s |
| Required: Wood | X |
| Required: Stone | X |
| Required: Clay | X |

**Properties:**
- Requires **min 1, max 2 units** to garrison
- Range enhancement: `unit range + tower height`
- Must be built within supply cover range

---

## Boat

A mobile platform on water for fishing and transport.

### Stats

| Stat | Value |
|------|-------|
| Health | 200 |
| Armor | 40 |
| Sight | 50 |
| Space | 6 units (workers/fighters) for transport |
| Fish capacity | 50 fish base |
| Supply consumption | 1/20s |
| Build time | X s |
| Movement speed | 2/s |
| Required: Wood | X |
| Required: Rope | X |

### River Direction Impact on Boat

| Direction | Speed Modifier |
|-----------|---------------|
| In direction of water | +10% |
| Opposite direction | -10% |
| Cross river | 0% |

### Capacity Rules

- For each unit on board, food capacity is reduced by **10**
- Example: A boat with 50 fish (full food) can only hold **2 units**
- Example: A boat with 6 units has only **50** available fish capacity (base - 6*0, since 50 is the minimum)

### Special Rules

- Can be built on: rivers, lakes (free water)
- Requires **at least 1 unit** on board to move
- A **vacant boat** (no one on board) can be taken by any player -- including enemies
- Must be built within supply cover range

---

[Back to Index](./index.md) | [Prev: Units](./08-units.md) | [Next: Tech Tree](./10-tech-tree.md)
