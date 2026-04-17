# GAIA System

[Back to Index](./README.md) | [Prev: Season/Climate Tables](./05-environment-season-climate-tables.md) | [Next: Resources](./07-resources.md)

---

GAIA is named after the ancient Greek goddess of the Earth. It represents the world's autonomous systems: sun, animals, environment objects, and natural disasters.

## Sun Rotation and Shadow

- Sun rotation and shadow rendering based on sun location for all objects
- Tied to the day/night cycle (Morning / Noon / Afternoon / Night)

---

## Animals and Humans (GAIA-controlled)

All GAIA creatures share a **herd system**: herds have a max capacity, reproduce over time, and respawn if fully wiped out.

### Neanderthal

A GAIA-controlled human species.

| Stat | Value |
|------|-------|
| Health | 200 |
| Food | 300 |
| Skin | 3 |
| Attack | 30 per attack |
| Attack type | Melee |
| Attack range | 1 |
| Attack speed | 1/1s |
| Armor | 0 |
| Movement speed (normal) | 1/s |
| Movement speed (attack) | 5/s |
| Sight | 40 |
| Territory | X radius |

**Herd behavior:**
- Max **10** Neanderthals per herd (including babies)
- Every X min, 1 baby added to herd (up to max capacity)
- Each baby matures after X+Y min
- If a unit attacks a Neanderthal, **all mature Neanderthals** in the herd attack the nearest enemy units within their territory
- If the entire herd is destroyed, after 3*X min one baby spawns on empty land outside player territory and the herd restarts

### Lion

Predator animal.

| Stat | Value |
|------|-------|
| Health | 200 |
| Food | 300 |
| Skin | 3 |
| Attack | 30 per attack |
| Attack type | Melee |
| Attack range | 1 |
| Attack speed | 1/1s |
| Armor | 0 |
| Movement speed (normal) | 1/s |
| Movement speed (attack) | 5/s |
| Sight | 40 |
| Territory | X radius |

**Herd behavior:**
- Max **7** lions per herd (including cubs)
- Every X min, 1 cub added (up to max capacity)
- Each cub matures after X+Y min
- If attacked, **all mature lions** in the herd fight back within territory
- Full herd respawn: 3*X min after total wipeout

### Deer

Non-predator prey animal.

| Stat | Value |
|------|-------|
| Health | 50 |
| Meat | 300 |
| Skin | 2 |
| Armor | 0 |
| Movement speed (normal) | 1/s |
| Movement speed (flee) | 3/s |
| Sight | 40 |
| Territory | X radius |

**Herd behavior:**
- Max **15** deer per herd (including fawns)
- Every X min, 1 fawn added (up to max capacity)
- Each fawn matures after X+Y min
- If attacked, **all deer run away** after each hit, but stay within their territory
- Full herd respawn: 3*X min after total wipeout

### Elephant

Large prey animal (fights back).

| Stat | Value |
|------|-------|
| Health | 500 |
| Meat | 2,000 |
| Skin | 10 |
| Attack | 60 per attack |
| Attack type | Melee |
| Attack range | 1 |
| Attack speed | 1/2s |
| Armor | 5 |
| Movement speed (normal) | 1/s |
| Movement speed (attack) | 2/s |
| Sight | 40 |
| Territory | X radius |

**Herd behavior:**
- Max **7** elephants per herd (including calves)
- Every X min, 1 calf added (up to max capacity)
- Each calf matures after X+Y min
- If attacked, **all mature elephants** fight back within territory
- Full herd respawn: 3*X min after total wipeout

### Wild Cow

Medium prey animal (fights back).

| Stat | Value |
|------|-------|
| Health | 300 |
| Food | 500 |
| Skin | 5 |
| Attack | 20 per attack |
| Attack type | Melee |
| Attack range | 1 |
| Attack speed | 1/2s |
| Armor | 2 |
| Movement speed (normal) | 1/s |
| Movement speed (attack) | 3/s |
| Sight | 40 |
| Territory | X radius |

**Herd behavior:**
- Max **7** wild cows per herd (including calves)
- Every X min, 1 calf added (up to max capacity)
- Each calf matures after X+Y min
- If attacked, **all mature cows** fight back within territory
- Full herd respawn: 3*X min after total wipeout

---

## Environment Objects

### Trees

Trees grow through stages over time and are a primary source of wood.

#### Growth Stages

| Stage | Grows into | Time | Resource (fallen) | Cut Time | Cut Tool |
|-------|-----------|------|-------------------|----------|----------|
| Sapling | Small tree | X min | 100 wood | 2s | None required |
| Small tree | Medium tree | Y min | 100 wood | 5s | Rope OR Stone Axe |
| Medium tree | Big tree | Z min | 200 wood | 10s | Rope OR Stone Axe |
| Big tree | -- | -- | 300 wood | 10s | Stone Axe only |
| Dead tree (any size) | -- | -- | 50-200 wood | 10s | Rope OR Stone Axe |

#### Sapling Spawning Rules

- On empty land outside building territory, beside each big/medium tree, 1 new sapling appears every X min
- If a grown tree is in a player's territory, **no saplings** appear around it
- If all spaces around a grown tree are full, no new saplings appear
- Workers will **not** cut saplings by default (auto-gathering ignores them). Player must order directly.

#### Dead Trees

- Created when a tree catches fire (see [Natural Disasters](#natural-disasters))
- Can be small, medium, or big dead trees

### Fruit Bushes (Strawberry)

- Food: **500**
- Spawns near existing trees on empty GAIA land every X min
- **Available only in Spring, Summer, and Fall** (not Winter)

### Stone

| Type | Replenish | Resource | Gathering |
|------|-----------|----------|-----------|
| Small stone | Every X min on empty GAIA land | 10 stone | Immediate |
| Big stone | Every X min on empty GAIA land | 20 small stones | Requires cut stone action first |

### Clay

- Spawns every X min on empty GAIA land
- Clay per site: **500**

### Long Grass

- Spawns automatically and randomly on any empty land
- Each long grass: **10 grass** resource
- Purpose: creating **rope** (5 long grass = 1 rope, if rope upgrade is researched)
- Units can walk through long grass

---

## Natural Disasters

Triggers are based on the [Season/Climate Tables](./05-environment-season-climate-tables.md).

### River Flood

- Triggered by climate chance tables
- When flood occurs, the **riverside goes underwater**

**Impacts:**
- Players standing on riverside: **die (vanished)**
- Animals standing on riverside: **die (vanished)**
- Buildings on riverside: **destroyed**
- Natural resources on riverside: **vanish** (long grass, clay, etc.)
- During flood: **no one can cross the river**
- Boats: **destroyed**

**Properties:**
- Flood speed: 10/s
- Flood duration: X min
- After the flood, the river returns to normal

### Wildfire

- Triggered by climate chance tables
- Randomly affects **30% of trees** outside player territories
- Wildfire duration: X min
- Affected trees convert to **dead trees** when fire ends

### Famines

When a famine hits a region, **all resource reproduction stops**:
- No sapling trees appear
- No animal births in herds
- No new grass appears
- Weather: no rain, no snow, no cloud -- only sunny

**Duration:**
- Normal mode (1v1): X min
- Survival mode: Y min

### Fog

- Sight of all units: **-20%**
- When triggered, one of these types is randomly selected:
  - Thin fog
  - Medium fog
  - Thick fog

> Design note: Consider a 1-100 continuous scale instead of 3 discrete types.

### Wind

- **No impact on unit stats** -- purely visual
- Visual effects on: long grass, trees, rain, snow, clouds, fire
- Impact intensity depends on wind type

Wind types (randomly selected when triggered):
- Slow wind
- Medium wind
- Fast wind

> Design note: Consider a 1-100 continuous scale (wind speed) instead of 3 discrete types.

---

## Water Bodies

### River

| Section | Crossing | Fishing | Notes |
|---------|----------|---------|-------|
| Deep section | No (boat only) | By boat, or from land if fish are near | Only boats can navigate |
| Shallow section | Yes (with speed penalty) | By boat or without | Units can wade through |
| Riverside | N/A (it's land) | N/A | Floods affect this area. Players can build here. GAIA can spawn resources here. |

### Lake

- **Non-winter or non-cold climate**: No special impact
- **Winter + Cold climate**: Freezes
  - Boats **cannot move** on frozen lakes
  - Units **can walk** on frozen lakes
  - Players can perform **ice breaking** to create ice holes for fishing
- Source of **fresh water**

### Ocean

- **Not** a source of water
- **Does not freeze** during winter/cold climate

### Ice Hole

- Created by players only when a lake is frozen
- Can only be created where there are fish

---

## Cosmetic Elements

- Terrain: TBD
- Objects: TBD

---

[Back to Index](./README.md) | [Prev: Season/Climate Tables](./05-environment-season-climate-tables.md) | [Next: Resources](./07-resources.md)
