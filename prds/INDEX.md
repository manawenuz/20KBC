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

## Batch 2 (after batch 1 integrated)

To be planned: supply HUD, day/night cycle, replay log, 10-worker spawn + stress test.

## Reading order for an agent

1. Read your assigned PRD top-to-bottom.
2. Read the file paths it points at (existing code in `spikes/godot-gdext/rust/src/`
   and `game-core/src/`).
3. Implement only what the PRD says. Do not refactor unrelated code.
4. Run the build/test commands listed in the Acceptance section.
5. If acceptance fails, fix and retry. If you cannot make it pass, STOP and explain why.
