# AGENTS.md — 20KBC Multi-Agent Workflow

This project is built by an **orchestrator** (Claude) coordinating
**workers** (Kimi-k2.6 subagents) over **PRDs** in **isolated worktrees**.
Read this before starting any non-trivial implementation work.

## Why this exists

A single agent context is the bottleneck. Splitting work into small,
self-contained PRDs lets multiple Kimi processes work in parallel on
disjoint files, and lets the orchestrator stay focused on integration,
review, and end-to-end testing — the parts that genuinely need a single
mind in charge.

## Roles

| Role | Who | Responsibility |
|------|-----|----------------|
| Orchestrator | Claude (you, in the main session) | Decompose goals into PRDs, dispatch workers, review output, integrate, run end-to-end tests, commit |
| Worker | Kimi (`kimi --agent-file ~/.kimi/agents/20kbc/agent.yaml`) | Read one PRD, implement it in its assigned worktree, run the PRD's acceptance checks, stop |
| Reviewer (optional) | Kimi `code-reviewer` subagent | Pre-merge second opinion when changes are risky |

The orchestrator **never** edits files inside `.worktrees/*` directly.
It reviews, then merges branches back to `main`.

## Workflow

### 1. Define the goal

User states an outcome (e.g. "Get the MVP playable"). Orchestrator reads
the relevant plans (`plans/`, `prds/INDEX.md`), audits current state,
proposes a decomposition, and confirms scope with the user via
`AskUserQuestion` when there is a meaningful fork.

### 2. Write PRDs

PRDs live in `prds/PRD-NN-<slug>.md`. Each PRD is **self-contained** —
an agent reading only the PRD must be able to execute it. Mandatory
sections:

- **Goal** — one paragraph
- **Context** — pointers to existing files the worker should read
- **Files you MAY create** — strict whitelist
- **Files you MAY modify** — strict whitelist
- **Files you MUST NOT touch** — explicit forbidden list (integration glue lives here)
- **Interface contract** — Rust signatures, function names, types
- **Implementation hints** — point at similar existing code
- **Acceptance criteria** — concrete commands the orchestrator will run
- **Out of scope** — explicit non-goals

Keep PRDs **disjoint**. If two PRDs would touch the same file, either
merge them into one PRD or designate one of them owner and the other
as out-of-scope for that file. The integration glue (e.g. `Main.tscn`,
`main.gd`, `project.godot`, sometimes `lib.rs`) is **always** the
orchestrator's responsibility, not a worker's.

### 3. Create worktrees

One worktree per PRD, one branch per PRD:

```bash
mkdir -p .worktrees
git worktree add .worktrees/prd-NN-<slug> -b prd-NN-<slug> main
```

Worktrees share the workspace `target/` cache via cargo, so subsequent
builds are fast.

### 4. Dispatch Kimi

Use the wrapper at `scripts/dispatch-kimi.sh`:

```bash
./scripts/dispatch-kimi.sh \
    .worktrees/prd-NN-<slug> \
    prds/PRD-NN-<slug>.md \
    logs/prd-NN.log \
    rust-pro      # subagent hint (optional, default rust-pro)
```

Subagent hints:

| Hint | Use for |
|------|---------|
| `rust-pro` | Rust crates, gdext code, game-core changes |
| `game-developer` | Engine integration, rendering, gameplay loop |
| `3d-artist` | Asset pipelines, BLP/MDX extraction, texture work |
| `code-reviewer` | Post-implementation review pass |
| `debugger` | Reproducing/diagnosing a known failure |
| `game-designer` | Mechanics, balance, level design (read-only mostly) |

Launch each dispatch as a background Bash with `run_in_background=true`
so multiple PRDs can run truly in parallel. The harness will notify the
orchestrator on completion.

The wrapper passes:
- `--agent-file ~/.kimi/agents/20kbc/agent.yaml` (full subagent tree)
- `--print --final-message-only` (non-interactive, just the summary)
- `-y --afk` (auto-approve tool calls, no human in the loop)
- `-w <worktree>` (scope to the worktree)

If `~/.kimi/agents/20kbc/agent.yaml` is missing, the wrapper falls back
to the default agent.

### 5. Review

When a worker finishes, the orchestrator reviews **inside the worktree**:

```bash
git -C .worktrees/prd-NN-<slug> status --short
git -C .worktrees/prd-NN-<slug> diff --stat
git -C .worktrees/prd-NN-<slug> diff <files>
```

Checklist:

- [ ] Diff scope matches the PRD's "files you may modify/create" whitelist
- [ ] No edits to forbidden files
- [ ] Interface signatures match the PRD's contract
- [ ] Acceptance commands pass (run them yourself — don't trust the worker's summary)
- [ ] No obvious style regressions vs. existing code

If review fails, do not edit the worktree by hand. Either:
1. Update the PRD with a clarifying note and re-dispatch Kimi to the
   same worktree (it will pick up the dirty tree), or
2. Reject the PRD, delete the worktree+branch, rewrite the PRD, restart.

### 6. Integrate

Merge each accepted branch back to `main`:

```bash
git checkout main
git merge --no-ff prd-NN-<slug> -m "Integrate PRD-NN: <title>"
```

Then **the orchestrator** (not a worker) updates the integration glue
that workers were forbidden from touching:

- `spikes/godot-gdext/scenes/Main.tscn` — add new node types to the scene
- `spikes/godot-gdext/scripts/main.gd` — wire up new SimBridge methods
  and new node instantiation
- `spikes/godot-gdext/project.godot` — input map, autoloads
- `game-core/src/lib.rs` — re-exports if needed

### 7. End-to-end test

The orchestrator launches Godot and verifies the feature works
visually. Build first:

```bash
cd spikes/godot-gdext/rust && cargo build
godot --path /Users/manwe/CascadeProjects/20KBC/spikes/godot-gdext/
```

Capture any regressions as bugs in `prds/INDEX.md` under a "Known issues"
section, then either fix in `main` directly or queue as a new PRD.

### 8. Clean up

```bash
git worktree remove .worktrees/prd-NN-<slug>
git branch -d prd-NN-<slug>
```

Keep `prds/PRD-NN-*.md` files committed — they are project history.

## When to skip the worker workflow

Do it yourself, no PRD, if:

- The change is one file, under ~30 lines, and isolated.
- It's an integration step (Main.tscn / main.gd / project.godot edits).
- It's a one-off investigation or read-only exploration.
- The user is asking a question, not requesting work.

Anything multi-file or spanning hours of work — write a PRD.

## Conventions

- PRDs are numbered sequentially across batches. Do not reuse numbers.
- Worktrees live under `.worktrees/` (gitignored by `.git/worktrees` mechanism).
- Logs live under `logs/prd-NN.log`. Gitignore them.
- Branches are named identically to worktree dirs: `prd-NN-<slug>`.
- Each PRD must end with a `cargo build` (or equivalent) acceptance check.
- Workers do not commit. Orchestrator commits after merge.

## Reference

- Project root: `/Users/manwe/CascadeProjects/20KBC`
- WC3 asset deep dive: `docs/WC3Analysis/`
- MVP plan: `plans/00-mvp-overview.md`
- Kimi skill: `.claude/skills/kimi-agents/SKILL.md`
- Kimi agent tree: `~/.kimi/agents/20kbc/`
- Dispatcher: `scripts/dispatch-kimi.sh`
