# PRD-09 — WC3 MDX→glTF Extraction (Best Effort)

## Goal

Extend the asset extraction tool to pull WC3 MDX models from War3.mpq
and convert at least the **Peasant** and **Wolf** models to glTF 2.0
(`.glb`) format that Godot can import directly. If full conversion is
not feasible, ship whatever partial pipeline you can and **document
what's blocked** so the orchestrator can plan around it.

## Context

The existing tool is `scripts/asset-extract/extract.py` — Python +
StormLib + Pillow. It currently extracts BLP textures. WC3 MDX is a
binary chunked format documented widely in the modding community.

Asset paths in War3.mpq:
- `Units/Human/Peasant/Peasant.mdx` (worker)
- `Units/Critters/Wolf/Wolf.mdx` (wolf)
- `Doodads/LordaeronSummer/Trees/AshenTree/AshenTree.mdx` (tree, fallback any TreeXxx.mdx)
- `Doodads/Cinematic/RockChunks/RockChunks0.mdx` (rock cluster, fallback any Rock*.mdx)

## Approach (try in order)

1. **Search crates.io for a WC3-MDX→glTF crate.** If something works,
   use it via a tiny Rust shim added to the asset-extract tool.
2. **Look for `mdx-m3-viewer` Node tool.** It's a TS library that parses
   MDX. If you can drive it from CLI to dump glTF, great.
3. **Hand-roll a minimal MDX parser** that handles just enough to emit
   a static mesh + UVs + first texture (no skinning, no animation).
   The MDX format spec is in plain-text references easy to find.
4. **If none of the above land in reasonable time:** ship a JSON
   manifest that lists which MDXes exist in the MPQ, sizes, and write
   `scripts/asset-extract/MDX_STATUS.md` documenting the gap. Don't
   silently fail.

## Files you MAY create

- Anything under `scripts/asset-extract/`
- Anything under `spikes/godot-gdext/assets/models/`
- `scripts/asset-extract/MDX_STATUS.md` (status report)

## Files you MAY modify

- `scripts/asset-extract/extract.py` (extend with MDX commands)

## Files you MUST NOT touch

- `spikes/godot-gdext/rust/**`
- `spikes/godot-gdext/scenes/`, `scripts/`, `project.godot`
- `game-core/**`
- Any other PRD's territory

## Acceptance criteria

At least ONE of these must be true:

- [ ] `spikes/godot-gdext/assets/models/peasant.glb` exists and Godot
      can import it (try: `godot --headless --import` exits 0)
- [ ] `spikes/godot-gdext/assets/models/wolf.glb` exists and imports
- [ ] `scripts/asset-extract/MDX_STATUS.md` exists with: (a) a list of
      every MDX path in War3.mpq that matches Peasant/Wolf/Tree/Rock,
      (b) which conversion approach you tried, (c) why it didn't work,
      (d) a recommendation for next step (e.g. "use mdx-m3-viewer JS
      port via Node, blocked on glTF export — needs ~4h dedicated work")

And always:

- [ ] `python scripts/asset-extract/extract.py --help` still works
- [ ] `git diff --stat` shows changes only under the whitelist

## Out of scope

- Animation export (static mesh acceptable for MVP)
- BLP texture re-encoding to PNG inside glTF (use external texture refs is fine)
- Reforged-era MDX900 support (Patch 1.27 = MDX800 only)
