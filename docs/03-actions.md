# Actions

[Back to Index](./README.md) | [Prev: Core Mechanics](./02-core-mechanics.md) | [Next: Environment](./04-environment.md)

---

Actions are processes of doing something in the game. They may be commanded by the player or GAIA (AI).

## Auto Resource Gathering

Units automatically gather all available resources within the home base supply cover range.

- Player can enable auto-gathering on workers
- Workers will gather: long grass, water, fish (from boat), hunted animals, fallen trees, etc.

## Fishing

Workers hunt fish, gather them, and deliver to the nearest supply depot or home base.

### Requirements

- Workers must have a **spear** to fish

### On Land (Near Water)

- When the worker's resource capacity is reached, they automatically return to the nearest supply depot or home base to deliver fish

### On Boat

- When the worker's resource capacity and boat capacity are reached, the boat returns to the nearest land
- Workers deliver gathered food (worker + boat)
- Other workers can help gather from the boat faster

### Timing

- **5 seconds per fish**

## Attack

Units deal damage to other entities. Applies to PvP, PvE, and EvE (environment vs environment).

### Attack Types

- **Attack animals**: Standard combat
- **Predator animals attack back**: They fight
- **Non-predator animals**: Run away within their territory
- **Range vs melee**: Range attack if target is far, melee if close

### Attack Buildings

#### Fire Attack (requires fire upgrade)
- Damage: **40 / s**
- Preparation time: **5 s**
- **Not applicable** for stone walls or stone towers

#### Rope Attack (requires rope upgrade)
- Damage: **30 / s**
- Preparation time: **5 s**
- **Not applicable** for boats

## Gathering

When a player commands a worker to gather a resource. For wood: workers first perform "cutting down tree" then gather from the fallen tree. If a tree is already fallen, gathering happens immediately.

| Resource | Notes |
|----------|-------|
| Strawberry bushes | Direct gathering |
| Meat from dead animals | From hunted/killed animals |
| Wood from fallen trees | Tree must be fallen first |
| Wood/stone from destroyed buildings | Direct gathering |
| Fish from fish boat | Workers can gather from a boat |
| Long grass | Direct gathering |
| Stone (small) | Immediate, 1s per stone. Each worker carries 1 small stone (= 20 stone resources) |
| Stone (big) | Requires "cut stone" action first, then 1s per stone. Priority: gather small stone first |
| Water | Male and female workers, 2s per gather |
| Clay | Male and female workers, 5s per gather |

## Cutting Down Trees

To gather wood, workers must first cut down a standing tree, converting it to a fallen tree.

| Tree Type | Requirements | Time |
|-----------|-------------|------|
| Sapling | None | 1 s |
| Small tree | Rope (3 units) OR Stone Axe | 2 s |
| Medium tree | Rope (5 units) OR Stone Axe | 5 s |
| Big tree | Stone Axe | 10 s |

## Cutting Stone

To gather stone from big stones, perform the cutting stone action first.

- **No tools required** -- workers use a small stone with both hands to crush it on the big stone
- **Time**: 10 s
- **Result**: 3 small stones appear, which workers can then gather

## Ice Breaking

When a lake is frozen, players must break the ice before fishing.

- Only on frozen lakes
- Must break ice first, then fish through the hole
- **Time**: 10 s

## Move

Command to move a unit from one place to another.

- Slope and weather can impact movement speed for both GAIA and player units

## Construction

Command to construct a building.

- Required resources must be available
- Resources are deducted from storage **immediately** when construction starts

---

[Back to Index](./README.md) | [Prev: Core Mechanics](./02-core-mechanics.md) | [Next: Environment](./04-environment.md)
