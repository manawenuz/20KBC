# Tech Tree

[Back to Index](./index.md) | [Prev: Buildings](./09-buildings.md)

---

All upgrades are researched at the **Home Base** (Global) or on **individual units** as noted.

## Upgrade List

| # | Upgrade | Scope | Effect | Prerequisites |
|---|---------|-------|--------|---------------|
| a | **Skinning** | Global (Home Base) | Workers gather **skin** from wild animals in addition to meat | -- |
| b | **Skin Bag** | Individual (Unit) | Increase unit carry capacity by **+20%** | Skinning |
| c | **Rope** | Global (Home Base) | Home base auto-converts 5 long grass -> 1 rope. Enables rope usage for units. | -- |
| d | **Stone Tool** | Global (Home Base) | Allows upgrading human units with **stone axes** and **stone spears** | -- |
| e | **Irrigation** | Global (Home Base) | Improve farms: generation rate **+30%** | -- |
| f | **Stone Axe Upgrade** | Global (Home Base) | **+20%** axe damage | Stone Tool |
| g | **Stone Spear Upgrade** | Global (Home Base) | **+20%** spear damage | Stone Tool |
| h | **Clothes** | Global (Home Base) | Decrease supply consumption of units by **-20%** | Skinning |
| i | **Supply Depot Upgrade** | Global (Home Base) | Increase supply cover range by **+30%** | -- |
| j | **Ladder** | Global (Home Base) | Allows units to use **wooden ladders** to climb on top of walls | -- |

## Tech Tree Diagram

```
(No prerequisites)
├── Skinning
│   ├── Skin Bag (per unit)
│   └── Clothes
├── Rope
│   └── (Enables: sling upgrade, rope wall destruction, boat building, tree cutting)
├── Stone Tool
│   ├── Stone Axe Upgrade
│   └── Stone Spear Upgrade
├── Irrigation
├── Supply Depot Upgrade
└── Ladder
```

## Upgrade Details

### Skinning
- **Scope**: Global -- affects all workers
- **Effect**: When hunting wild animals, workers now also collect **skin** (a resource used for clothing and skin bags)
- **Why it matters**: Skin is needed for Clothes upgrade and Skin Bags, both of which improve unit efficiency

### Skin Bag
- **Scope**: Individual -- must be applied per unit
- **Cost**: TBD
- **Effect**: +20% carry capacity for the upgraded unit
- **Requires**: Skinning upgrade

### Rope
- **Scope**: Global -- enables the rope economy
- **Effect**: Home base automatically converts 5 long grass into 1 rope
- **Enables**:
  - Sling upgrade path for units
  - Using rope to pull down wooden walls/buildings
  - Building boats
  - Cutting small/medium trees with rope

### Stone Tool
- **Scope**: Global -- unlocks combat upgrades
- **Effect**: Allows upgrading human units to Level 2.1 (Axes) or Level 2.2 (Spear)
- **Why it matters**: Core combat upgrade that transitions workers into fighters

### Irrigation
- **Scope**: Global -- affects all farms
- **Effect**: Farm food generation rate increased by 30%
- **Why it matters**: Crucial for late-game food production, especially before winter

### Stone Axe Upgrade
- **Scope**: Global -- affects all axe-wielding units
- **Effect**: +20% damage for all axe attacks
- **Requires**: Stone Tool

### Stone Spear Upgrade
- **Scope**: Global -- affects all spear-wielding units
- **Effect**: +20% damage for all spear attacks
- **Requires**: Stone Tool

### Clothes
- **Scope**: Global -- affects all units
- **Effect**: Supply consumption reduced by 20% for all units
- **Requires**: Skinning
- **Why it matters**: Extends how long your units can survive, especially in harsh winters

### Supply Depot Upgrade
- **Scope**: Global -- affects all supply depots
- **Effect**: Supply cover range increased by 30%
- **Why it matters**: Allows wider territory control and more distant resource gathering

### Ladder
- **Scope**: Global -- enables a siege tactic
- **Effect**: Units can use wooden ladders to climb enemy walls and towers (3s climb time)
- **Why it matters**: Essential for attacking fortified enemies

---

[Back to Index](./index.md) | [Prev: Buildings](./09-buildings.md)
