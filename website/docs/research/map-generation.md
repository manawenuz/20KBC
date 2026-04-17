---
sidebar_position: 3
---

# Procedural Map Generation for RTS



## Requirements from Design Doc

- Maps range from small (1 region, 1v1) to giant (many regions, 30 players)
- Each region has a geographic zone: Hot & Dry, Hot & Humid, Moderate, Cold
- Terrain features: slopes (0-90+), rivers (deep/shallow), lakes, ocean, riverside
- Resources placed on terrain: trees, stones, clay, long grass, strawberry bushes
- Fair starting positions for all players
- Customizable settings: map size, climate, player locations


## Terrain Generation Pipeline

### Step 1: Heightmap Generation

Use layered Perlin/Simplex noise to create natural-looking terrain.

```
Base continent shape     → Large-scale noise (low frequency)
+ Mountain ranges        → Medium-scale noise
+ Local hills/valleys    → High-frequency noise
+ Erosion simulation     → Hydraulic erosion pass (optional)
= Final heightmap
```

**Rust crate**: `noise-rs` — supports Perlin, Simplex, Worley, Ridged Multi, and Fractal Brownian Motion in 1-4 dimensions.

### Step 2: Water Body Placement

1. **Ocean**: Edges of the map below a threshold height
2. **Lakes**: Fill depressions in the heightmap (flood-fill algorithm)
3. **Rivers**: Trace downhill paths from peaks/ridges to lakes/ocean
   - Use recursive pathfinding: start at high point, move to lowest neighbor
   - Rivers widen as they accumulate flow
   - Mark tiles as deep (center) or shallow (edges) based on flow volume
4. **Riverside**: 1-2 tile buffer around rivers on land

### Step 3: Region Division

1. Place region centers (Voronoi-like distribution or grid-based)
2. Assign geographic zone to each region based on latitude/elevation:
   - High elevation or high latitude → Cold
   - Low elevation, low latitude → Hot & Dry or Hot & Humid
   - Mid-range → Moderate
3. Blend zone boundaries for smooth transitions

### Step 4: Slope Calculation

- Calculate slope per tile from heightmap gradient
- Slope affects: movement speed (uphill penalty, downhill bonus), range bonus
- Slope categories per design doc: 0-10, 10-20, 20-40, 40-60, 60-90, 90+

### Step 5: Resource Placement

| Resource | Placement Rules |
|----------|----------------|
| Trees | Clusters near water, higher density in Moderate/Humid zones, less in Cold/Dry |
| Big stones | On hills and elevated terrain |
| Small stones | Scattered, slightly more on elevated terrain |
| Clay | Near rivers and in lowlands |
| Long grass | Random on flat empty land |
| Strawberry bushes | Near trees, not in Cold zones |
| Fish | In rivers and lakes |
| Water holes | Generated dynamically after rainfall |

### Step 6: Starting Position Selection

1. Identify candidate positions: flat terrain, near water, near resources
2. Ensure minimum distance between players
3. Validate fairness: each starting position has approximately equal access to wood, stone, water, food within a radius
4. Place initial Home Base and starting resources


## Map Validation

Automated checks after generation:

- [ ] Every starting position has flat ground for Home Base (8x8)
- [ ] Every starting position has trees within 30 tiles
- [ ] Every starting position has water source within 40 tiles
- [ ] Every starting position has stone within 30 tiles
- [ ] No starting position is isolated by impassable terrain
- [ ] At least one path exists between all starting positions
- [ ] Rivers are connected (no floating water segments)

If validation fails, regenerate with a new seed.


## Chunked Terrain System

For large maps (256x256+), use chunked loading:

- Divide map into chunks (e.g., 32x32 tiles each)
- Only render chunks within camera view + buffer
- All chunks loaded in memory for simulation (RTS needs global state)
- LOD (Level of Detail): distant chunks render at lower resolution


## Dynamic Terrain During Gameplay

The map is not static. GAIA spawns resources over time:

| What Spawns | Where | When |
|-------------|-------|------|
| Saplings | Adjacent to big/medium trees, outside player territory | Every X min |
| Small/big stones | Empty GAIA land | Every X min |
| Clay deposits | Empty GAIA land | Every X min |
| Long grass | Random empty land | Every X min |
| Strawberry bushes | Near trees, GAIA land | Spring, Summer, Fall |
| Water holes | Random empty land | After each rainfall |


## Rust Crates

| Crate | Purpose |
|-------|---------|
| `noise` (noise-rs) | Perlin, Simplex, FBM noise generation |
| `rand` + `rand_chacha` | Seeded RNG for deterministic generation |
| `pathfinding` | A* and Dijkstra for river tracing and connectivity checks |
| `image` | Export heightmaps as images for debugging |


## References

- [Terrain Generation with Bevy](http://clynamen.github.io/blog/2021/01/04/terrain_generation_bevy/)
- [bevy_generative](https://github.com/manankarnik/bevy_generative) — real-time procedural generation
- [bevy_terrain](https://github.com/kurtkuehnert/bevy_terrain) — terrain rendering
- [noise-rs](https://github.com/Razaekel/noise-rs) — noise functions
- [Hydraulic Erosion Simulation](https://www.firespark.de/resources/downloads/implementation%20of%20a%20methode%20for%20hydraulic%20erosion.pdf)
- [Red Blob Games: Map Generation](https://www.redblobgames.com/maps/terrain-from-noise/)
