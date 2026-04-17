---
sidebar_position: 2
---

# Core Mechanics



## Ranged Weapon Attack Accuracy

This mechanism applies to **all units** including GAIA and players with ranged attacks.

### Factors

When a unit uses a ranged weapon to attack, these factors are involved:

1. Distance between attacker and target
2. Movement of target (is it moving?)
3. Movement speed of the target
4. Speed of weapon (spear or stone)
5. Chance (probability of impact)

### Example

Given:
- Distance between attacker and target: **20 units**
- Movement speed of target: **0** (stationary)
- Speed of weapon (spear or stone): **20 / s**
- Result: spear reaches target in **1 second**

Impact location is random across a grid:

| Location | Chance |
|----------|--------|
| Location 1 (top) | 10% |
| Location 2 (left) | 10% |
| Location 3 (center) | 60% |
| Location 4 (right) | 10% |
| Location 5 (bottom) | 10% |

**Key rules:**
- If the target is **not moving**, there is a **60% chance** of impact
- If the target **is moving**, the chance of impact is significantly lower
- If the projectile misses the intended target but **another unit occupies the impact location**, that unit takes the hit instead
- **Slope impacts range** similarly to wall height impact (see [Buildings - Walls](./09-buildings.md#walls))

## Skill Upgrades

Each unit gains experience by using tools and levels up.

### Axes

Units gain XP by attacking animals or enemies. Cutting trees does **not** grant XP.

| Level 1 | Level 2 | Level 3 |
|---------|---------|---------|
| XP: 0 - 30 | XP: 30 - 70 | XP: 70+ |
| No impact | Axes damage +5 | Axes damage +10 |

### Spear (Melee)

| Level 1 | Level 2 | Level 3 |
|---------|---------|---------|
| XP: 0 - 30 | XP: 30 - 70 | XP: 70+ |
| No impact | Spear damage +5 | Spear damage +10 |

### Spear (Throw)

| Level 1 | Level 2 | Level 3 |
|---------|---------|---------|
| XP: 0 - 30 | XP: 30 - 70 | XP: 70+ |
| No impact | Spear damage +5, Attack range +5 | Spear damage +10, Attack range +10 |

### Sling (Throw)

| Level 1 | Level 2 | Level 3 |
|---------|---------|---------|
| XP: 0 - 30 | XP: 30 - 70 | XP: 70+ |
| No impact | Sling damage +5, Attack range +5 | Sling damage +10, Attack range +10 |

## Movement

These rules apply to **all units** in the game (player units and GAIA).

### River Direction Impact on Movement Speed

| Condition | Speed Modifier |
|-----------|---------------|
| In the direction of water | +60% |
| Opposite direction of water | -40% |
| Crossing river | -50% |
| Deep section of river | **Not movable** |

### Other Movement Modifiers

- **Slope**: Impacts movement speed (see [Environment - Slope](./04-environment.md#slope-of-ground))
- **Climate**: Impacts movement speed (see [Season/Climate Tables](./05-environment-season-climate-tables.md))
- **Terrain objects**: Some objects block movement (big stones, trees, deep rivers), some do not (small stones, long grass, shallow rivers)

## Decomposition

Organic substances break down into simpler matter over time.

| Entity | Behavior |
|--------|----------|
| Dead human body | After X min, converts to skeleton. Cannot be used for food gathering. No cannibalism. |
| Dead animal body | After Z min, converts to skeleton. Food can be gathered before Z min. |
| Skeleton | Disappears after Y min |

## GAIA Animal Behavior

### Predators vs Non-Predators

- **Predators attack non-predators** if they enter the predator's territory
  - Non-predators run away
  - Can be hunted and killed, or lose health
  - Attack triggers are 1-for-1

- **Different predators attack each other** if they enter each other's territory
  - Fight to the death
  - Attack triggers are 1-for-1


