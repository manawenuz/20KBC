---
sidebar_position: 4
---

# Asset Strategy



## Guiding Principle

Art should never block gameplay development. Each phase uses the simplest assets that allow playtesting and validation of mechanics.


## Phase-by-Phase Asset Plan

### Phase 0-1: Colored Primitives (Months 1-4)

**Zero art dependency.** Everything is geometric shapes with colors.

| Entity | Visual |
|--------|--------|
| Male worker | Blue capsule (1x1x2) |
| Female worker | Pink capsule (1x1x2) |
| Healer | White capsule with green cross |
| Supply wagon | Brown box (1x2x2) |
| Home Base | Large grey box (8x8x3) |
| Supply Depot | Small grey box (2x2x2) |
| Farm | Flat green plane (10x10x1) |
| Walls | Rectangular prisms (brown = wood, grey = stone) |
| Towers | Tall rectangular prisms |
| Trees | Green spheres on brown cylinders |
| Stones | Grey spheres/cubes |
| Water | Flat blue plane |
| Terrain | Colored heightmap (green = grass, brown = dirt, white = snow) |

**Why**: Allows 100% focus on gameplay. Both devs can create these assets in code. No external dependencies.

### Phase 2-3: Free Low-Poly Placeholders (Months 5-11)

Replace primitives with recognizable low-poly models from free sources.

**Free Asset Sources (CC0 / CC-BY)**:

| Source | What | License |
|--------|------|---------|
| [Kenney.nl](https://kenney.nl/) | Nature kit, medieval kit, UI elements | CC0 (public domain) |
| [Quaternius](https://quaternius.com/) | Low-poly animals, nature, buildings | CC0 |
| [Poly Pizza](https://poly.pizza/) | Various low-poly models | CC-BY |
| [OpenGameArt](https://opengameart.org/) | Sprites, models, sounds | Various (check per asset) |

**AI-Generated Textures**:
- Use Stable Diffusion / DALL-E for terrain textures (grass, dirt, sand, snow, rock)
- Generate material maps (albedo, normal, roughness) with AI tools
- Manual cleanup in GIMP/Blender as needed

**Procedural Generation**:
- Trees: L-system or simple procedural (trunk + branches + leaf clusters)
- Terrain textures: Blend based on slope and height (grass on flat, rock on steep, snow on high)

### Phase 4: Continue Placeholders (Months 12-14)

Art is **not** a multiplayer blocker. Keep using Phase 2-3 assets while building networking.

### Phase 5: Final Art (Months 15-18)

**Options** (choose based on budget):

**Option A: Commission (~$2,000-5,000)**
- Hire freelance 3D artist for key models (units, buildings, animals)
- Platforms: ArtStation, Fiverr (high-end), Upwork
- Scope: ~20-30 unique models + texture sets
- Style: Low-poly with hand-painted textures (achievable, distinctive, performant)

**Option B: AI-Assisted Pipeline (~$500)**
- Generate initial 3D models with Meshy, Tripo3D, or similar
- Clean up topology and UVs manually in Blender
- Generate textures with AI, refine manually
- Good for: environment objects (trees, rocks, bushes)
- Less reliable for: characters, animals (need manual work)

**Option C: Distinctive Low-Poly Style (~$0)**
- Embrace the low-poly aesthetic as the art style
- Examples: Kingdoms and Castles, Before We Leave, Islanders
- Both devs can create models in Blender with basic training
- Consistent style is more important than high fidelity


## Art Pipeline

### Model Format
- **glTF 2.0 (.glb)** — industry standard, supported by Godot/Bevy/Fyrox
- PBR materials (albedo, metallic, roughness, normal)
- Max 1000 tris for units, 2000 for buildings, 500 for environment objects

### Animation
- Skeletal animation for units and animals
- Simple 3-4 animations per unit: idle, walk, attack, gather
- Animals: idle, walk, run, attack, death
- Blender for animation authoring

### Texture Atlas
- Group related textures into atlases to reduce draw calls
- 1024x1024 atlas for terrain
- 512x512 per unit type
- 256x256 for environment objects

### LOD (Level of Detail)
- 3 LOD levels: full, half-poly, billboard
- Units beyond 50 tiles: half-poly
- Units beyond 100 tiles: billboard/dot


## Sound & Music Strategy

### Phase 1-3: Placeholder
- Free sound effects from [Freesound.org](https://freesound.org/) (CC0/CC-BY)
- Categories needed: combat (hit, death), gather (chop, mine, splash), UI (click, research), ambient (wind, rain, birds)
- No music needed for early phases

### Phase 5: Final Audio
- Commission 3-5 ambient tracks + 2 combat tracks (~$500-1000)
- Or use AI music generation (Suno, Udio) as base, refine
- Positional 3D audio for combat and gathering sounds
- Adaptive music: calm during peace, intense during combat, somber in winter


## UI Art

| Phase | Approach |
|-------|----------|
| 1-3 | Default engine UI (egui for Bevy, Godot UI for Godot). System fonts. Simple colored rectangles. |
| 5 | Custom UI skin. Icon set for resources/buildings/upgrades. Hand-drawn or AI-generated icons. |


## Budget Summary

| Item | Low Budget | Medium Budget |
|------|-----------|---------------|
| 3D Models | $0 (low-poly self-made) | $2,000-5,000 (commissioned) |
| Textures | $0 (AI-generated + free) | $500 (AI + cleanup) |
| Sound Effects | $0 (Freesound.org CC0) | $200 (commissioned key sounds) |
| Music | $0 (AI-generated) | $500-1,000 (commissioned) |
| **Total** | **$0** | **$3,200-6,700** |
