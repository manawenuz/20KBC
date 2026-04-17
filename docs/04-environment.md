# Environment

[Back to Index](./README.md) | [Prev: Actions](./03-actions.md) | [Next: Season/Climate Tables](./05-environment-season-climate-tables.md)

---

## Map Characteristics

- Each map may have **one or more regions**
- Each map has **one season mechanic** (season changes apply to the whole map simultaneously)
  - Seasons change every **10 minutes**: Spring -> Summer -> Fall -> Winter -> ...
  - Players can fix a season in game settings (only climate changes, no season transitions)
- Each region can have a **different geographic factor**
- Each region can have a **different climate** (but the season is the same for all regions at the same time)
- **Small maps**: 1 region, for 1v1 or 2v2 skirmish
- **Giant maps**: Many regions, for survival mode with many players

## Environment Factors & Variables

### Seasons

1. Spring
2. Summer
3. Fall
4. Winter

### Climate

| Climate | Variants |
|---------|----------|
| Rainfall | Medium, Heavy |
| Snowfall | Medium, Heavy |
| Sunny | -- |
| Cloudy | -- |

### Geographic Zones

1. Hot and dry
2. Hot and humid
3. Moderate
4. Cold

### Day/Night Cycle

**Morning / Noon / Afternoon / Night**

- Players should visually feel changing hours: shadows of objects, direction of sunlight, etc.
- **Night effect**: Sight for all units decreased by **20%**

> For the full matrix of Season x Geographic Zone x Climate impacts, see [Season/Climate Tables](./05-environment-season-climate-tables.md)

## Slope of Ground

Slope impacts speed for **all units including animals**.

### Moving Uphill

| Slope Range | Speed Modifier |
|-------------|---------------|
| 0 - 10 degrees | No impact |
| 10 - 20 degrees | -10% |
| 20 - 40 degrees | -30% |
| 40 - 60 degrees | -50% |
| 60 - 90 degrees | -80% |
| > 90 degrees | **Not movable** |

### Moving Downhill

| Slope Range | Speed Modifier |
|-------------|---------------|
| 0 - 10 degrees | No impact |
| 10 - 20 degrees | +10% |
| 20 - 40 degrees | +30% |
| 40 - 60 degrees | +50% |
| > 60 degrees | +80% |

### Slope Impact on Range

Same as height impact on range (see [Buildings - Walls/Towers](./09-buildings.md))

## Season / Climate Transitions

- Each map starts with a predefined or random season
- Each season lasts **X minutes**
- During each season, climate changes **4 times** (e.g., if each season is 30 min, climate changes every ~10 min based on chance tables)
- Same climate can occur consecutively

### Transition Effects

| Transition | Duration | Behavior |
|------------|----------|----------|
| Season change | 5 s | New stats apply immediately; visual transition is gradual |
| Climate change | 5 s | New stats apply immediately; visual transition is incremental (e.g., sunny to snowfall shows gradual change) |

---

[Back to Index](./README.md) | [Prev: Actions](./03-actions.md) | [Next: Season/Climate Tables](./05-environment-season-climate-tables.md)
