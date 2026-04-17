# Units

[Back to Index](./README.md) | [Prev: Resources](./07-resources.md) | [Next: Buildings](./09-buildings.md)

---

## General Rules

- Each unit heals automatically: **1 HP / 10s** (if supply is available)
- Each human unit occupies **1 location unit**
- Two types of spear exist: one for melee and one for ranged
- Players can switch unit abilities (Axes <-> Spear melee, Spear throw <-> Sling) at the cost of required resources. Switch only available out of combat and within territory.

## Upgrade Path

```
Level 1 (Worker)
├── Level 2.1 (Axes) ──┬── Level 3.1 (Axes + Spear throw)
│                       └── Level 3.2 (Axes/Spear + Sling)
└── Level 2.2 (Spear) ─┬── Level 3.1 (Axes + Spear throw)
                        └── Level 3.2 (Axes/Spear + Sling)
```

- **Level 3.1**: Can attack in melee or ranged (throw spear at range 3-15, melee at range 1-3)
- **Level 3.2**: Can attack in melee or ranged (throw stone by sling, melee at close range 1-4)

## General Human Abilities

All human units can:

1. **Create fire** to destroy wooden walls/buildings (X damage/s)
2. **Use rope** to destroy wooden walls (X damage/s)
3. **Repair buildings**
4. **Use wooden ladders** to climb walls/towers (takes 3s), then melee fight enemies on top
5. **Auto-fill supply reserve** when in range of supply depot
6. **Self-sustain** outside supply range by killing wild animals or fishing (1 food + 1 water = 1 supply)
7. **Prioritize reserve refill** when gathering meat with empty reserve
8. **Drop gathered resources** (not reserves) when ordered to attack during gathering

### Population Rule

**1 woman for every 5 men.** To train the 11th man, you must have at least 3 women.

---

## Male Worker

- Size: 1x1x2
- Gather rate: **100% efficiency** for wood, stone, clay, water, meat
- Gather rate: **80% efficiency** for farm

### Male Stats by Level

| Stat | Level 1 (Default) | Level 2.1 (Axes) | Level 2.2 (Spear) | Level 3.1 (Axes + Spear) | Level 3.2 (Axes/Spear + Sling) |
|------|-------------------|-------------------|--------------------|--------------------------|---------------------------------|
| Space | 1 | 1 | 1 | 1 | 1 |
| Health | 100 | 100 | 100 | 100 | 100 |
| Capacity (resource) | 15 | 15 | 15 | 15 | 15 |
| Supply reserve | 10 | 10 | 10 | 10 | 10 |
| Attack | 20 | 40 | 40 | 40 | 40 |
| Attack type | Melee | Melee | Melee | Melee & Range | Melee & Range |
| Attack range | 1 | 1 | 2 | Melee: 1, Range: 3-15 | Melee: 1, Range: 4-25 |
| Attack speed | 1/1s | 1/1s | 1/1s | Melee: 1/1s, Range: 1/3s | Melee: 1/1s, Range: 1/3s |
| Projectile speed | -- | -- | -- | Spear: 20/s | Sling: 20/s |
| Attack accuracy (range) | -- | -- | -- | 2/9 random | 2/9 random |
| Armor | 2 | 2 | 2 | 2 | 2 |
| Movement (normal) | 1/s | 1/s | 1/s | 1/s | 1/s |
| Movement (attack) | 2/s | 2/s | 2/s | 2/s | 2/s |
| Sight | 40 | 40 | 40 | 40 | 40 |
| Supply consumption | 1/10s | 1/10s | 1/10s | 1/10s | 1/10s |
| Train/Upgrade time | 10s | 10s | 10s | 10s | 10s |

### Male Upgrade Costs

| Resource | Level 1 | Level 2.1 | Level 2.2 | Level 3.1 | Level 3.2 |
|----------|---------|-----------|-----------|-----------|-----------|
| Supply | 100 | 0 | 0 | 0 | 0 |
| Wood | 0 | 10 | 40 | 50 | 50 |
| Stone | 0 | 20 | 10 | 50 | 50 |
| Rope | 0 | 1 | 1 | 1 | 1 |

> Upgrades require unit to be **out of combat**.

### Clothes Upgrade (Male)

- Cost: **1 skin**
- Effect: Supply consumption reduced to **0.5 / 10s**

### Capacity Note

Stone carrying is fixed: each worker can carry **1 small stone per trip** regardless of capacity stat.

---

## Female Worker

- Size: 1x1x2
- Gather rate: **80% efficiency** for wood, stone, clay, water, meat
- Gather rate: **100% efficiency** for farm

### Female Stats by Level

| Stat | Level 1 (Default) | Level 2.1 (Axes) | Level 2.2 (Spear) | Level 3.1 (Axes + Spear) | Level 3.2 (Axes/Spear + Sling) |
|------|-------------------|-------------------|--------------------|--------------------------|---------------------------------|
| Space | 1 | 1 | 1 | 1 | 1 |
| Health | 80 | 80 | 80 | 80 | 80 |
| Capacity (resource) | 10 | 10 | 10 | 10 | 10 |
| Supply reserve | 7 | 7 | 7 | 7 | 7 |
| Attack | 10 | 30 | 30 | 30 | 30 |
| Attack type | Melee | Melee | Melee | Melee & Range | Melee & Range |
| Attack range | 1 | 1 | 2 | Melee: 1, Range: 3-10 | Melee: 1, Range: 4-20 |
| Attack speed | 1/1s | 1/1s | 1/1s | Melee: 1/1s, Range: 1/3s | Melee: 1/1s, Range: 1/3s |
| Projectile speed | -- | -- | -- | Spear: 20/s | Sling: 20/s |
| Attack accuracy (range) | -- | -- | -- | 1/5 random | 2/9 random |
| Armor | 2 | 2 | 2 | 2 | 2 |
| Movement (normal) | 1/s | 1/s | 1/s | 1/s | 1/s |
| Movement (attack) | 2/s | 2/s | 2/s | 2/s | 2/s |
| Sight | 40 | 40 | 40 | 40 | 40 |
| Supply consumption | 1/10s | 1/10s | 1/10s | 1/10s | 1/10s |
| Train/Upgrade time | 10s | 10s | 10s | 10s | 10s |

### Female Upgrade Costs

| Resource | Level 1 | Level 2.1 | Level 2.2 | Level 3.1 | Level 3.2 |
|----------|---------|-----------|-----------|-----------|-----------|
| Supply | 75 | 0 | 0 | 0 | 0 |
| Wood | 0 | 10 | 40 | 50 | 50 |
| Stone | 0 | 20 | 10 | 50 | 50 |
| Rope | 0 | 1 | 1 | 1 | 1 |

### Clothes Upgrade (Female)

- Cost: **1 skin**
- Effect: Supply consumption reduced to **0.5 / 10s**

---

## Special Units

### Healer

A support unit that heals nearby friendly units.

| Stat | Value |
|------|-------|
| Size | 1x1x2 |
| Space | 1 |
| Health | 100 |
| Attack | 10 (only retaliates in melee) |
| Attack type | Melee |
| Attack range | 1 |
| Attack speed | 1/1s |
| **Heal power** | **5 HP/s** |
| **Heal range** | **10** |
| Armor | 10 |
| Movement speed (normal) | 1/s |
| Movement speed (attack) | 1/s |
| Sight | 50 |
| Supply consumption | 1/5s |
| Train time | X s |
| Required: Food | X |
| Required: Wood | X |

### Supply Wagon

A mobile supply depot. Keeps units alive when far from base.

> **Critical rule**: If a unit is out of range of supply depot / supply wagon **or** not enough supply is available, units start losing health at a rate of `supply consumption * 3` until death. Auto-heal also stops.

When selected and commanded to move, 2 men appear carrying the wagon (visual only, not real units).

| Stat | Value |
|------|-------|
| Size | 1x2x2 |
| Space | 2 |
| Health | 300 |
| Attack | None |
| Supply cover range | X radius |
| Armor | 10 |
| Movement speed (normal) | 1/s |
| Sight | 40 |
| Supply consumption | 1/5s |
| Train time | X s |
| Required: Wood | X |
| Required: Rope | X |
| Required: Skin | X |

---

[Back to Index](./README.md) | [Prev: Resources](./07-resources.md) | [Next: Buildings](./09-buildings.md)
