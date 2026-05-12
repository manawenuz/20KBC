# PRD Index — 20KBC Godot MVP

PRDs describe parallel work streams dispatched to Kimi via `kimi --print -y --afk`.
Each PRD is self-contained: an agent reading only the PRD must be able to execute it.

## Conventions

- **Files-you-may-create**: strict whitelist. Anything else is out of scope.
- **Files-you-may-modify**: also strict. If you need to touch something not listed, STOP and ask.
- **Files-you-MUST-NOT-touch**: integration glue (Main.tscn, main.gd, project.godot, lib.rs).
  The orchestrator will wire those up after merge.
- **Acceptance**: each PRD ends with a checklist the orchestrator runs.
- **Build verification**: every PRD must end with `cargo build` succeeding from
  `spikes/godot-gdext/rust/`.

## Batch 1 (parallel)

| ID | Title | Worktree | Status |
|----|-------|----------|--------|
| 01 | Wolf/GAIA rendering | `.worktrees/prd-01-wolf` | pending |
| 02 | Resource nodes + gather order | `.worktrees/prd-02-resources` | pending |
| 03 | Selection system | `.worktrees/prd-03-selection` | pending |
| 04 | WC3 asset extraction tool | `.worktrees/prd-04-assets` | pending |

## Batch 1 outcome (merged)

All 4 PRDs landed; integration glue committed; smoke-tested in Godot.

## Batch 2 (merged)

All 4 landed: 10 workers, day/night, replay-on-quit, stress + frame-time HUD.

## Batch 3 (parallel — asset infra + gameplay foundations)

| ID | Title | Worktree | Status |
|----|-------|----------|--------|
| 09 | WC3 MDX→glTF extraction (best effort) | `.worktrees/prd-09-mdx` | pending |
| 10 | OSS CC0 fallback model pack | `.worktrees/prd-10-oss-models` | pending |
| 11 | Terrain texture polish | `.worktrees/prd-11-terrain-polish` | pending |
| 12 | Box-drag multi-select | `.worktrees/prd-12-box-select` | pending |
| 13 | Formation move planner | `.worktrees/prd-13-formation` | pending |
| 14 | Attack-move + hostile target | `.worktrees/prd-14-attack-move` | pending |
| 15 | Combat feedback FX | `.worktrees/prd-15-combat-fx` | pending |
| 16 | Selected-unit portrait HUD | `.worktrees/prd-16-portrait` | pending |

## Batch 5 (path A: full MDX extraction — serial waves)

| ID | Title | Wave | Status |
|----|-------|------|--------|
| 21 | MDX BLP textures + glTF materials | 1 | pending |
| 22 | MDX bones + skinning → glTF skin | 2 (after 21) | pending |
| 23 | MDX animations → glTF anim tracks | 3 (after 22) | pending |
| 24 | Godot animation hookup (behavior → WC3 anim) | 3 (parallel with 23) | pending |

## Batch 4 (after batch 3 — animated models + audio)

| ID | Title | Worktree | Status |
|----|-------|----------|--------|
| 17 | Animated peasant model | `.worktrees/prd-17-peasant` | pending |
| 18 | Animated wolf model | `.worktrees/prd-18-wolf` | pending |
| 19 | Tree + stone outcrop models | `.worktrees/prd-19-doodads` | pending |
| 20 | WC3 audio extraction + SoundFx | `.worktrees/prd-20-audio` | pending |

## Reading order for an agent

1. Read your assigned PRD top-to-bottom.
2. Read the file paths it points at (existing code in `spikes/godot-gdext/rust/src/`
   and `game-core/src/`).
3. Implement only what the PRD says. Do not refactor unrelated code.
4. Run the build/test commands listed in the Acceptance section.
5. If acceptance fails, fix and retry. If you cannot make it pass, STOP and explain why.
