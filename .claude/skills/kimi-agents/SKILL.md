---
name: kimi-agents
description: |
  Delegate work to kimi-cli (Moonshot Kimi-k2.6) running a custom 20KBC
  agent tree with game-dev subagents (game-developer, game-designer,
  3d-artist, rust-pro, code-reviewer, debugger) plus the built-in coder /
  explore / plan. Use to offload long-running, parallelizable, or
  Kimi-better-suited tasks from the main Claude session.

  Triggers: kimi, kimi-cli, moonshot, k2.6, "delegate to kimi", "run with
  kimi", "use kimi for X", "offload to kimi", "second opinion from kimi".
---

# Kimi Agents Skill

Custom kimi-cli agent tree configured for 20KBC. Use Kimi as a parallel
worker when you need a second model's perspective, want to offload
context-heavy work, or want to run a long task in the background while
Claude continues elsewhere.

## Invocation

```bash
# Interactive TUI with full subagent tree
kimi --agent-file ~/.kimi/agents/20kbc/agent.yaml

# One-shot (YOLO = auto-approve tool calls; use for trusted work only)
kimi --agent-file ~/.kimi/agents/20kbc/agent.yaml --yolo -w /Users/manwe/CascadeProjects/20KBC

# AFK / non-interactive (good for background tasks via Bash run_in_background)
kimi --agent-file ~/.kimi/agents/20kbc/agent.yaml --afk --yolo
```

The agent is also addressable as the **20kbc** profile if symlinked into
the kimi built-ins dir, but the explicit `--agent-file` path above is the
canonical invocation.

## Agent Tree

Defined under `~/.kimi/agents/20kbc/` — main agent `extend`s the kimi
`default` and registers these subagents:

| Subagent         | When to delegate                                                          |
|------------------|---------------------------------------------------------------------------|
| `coder`          | General edits/commands (kimi built-in)                                    |
| `explore`        | Fast read-only codebase exploration (kimi built-in)                       |
| `plan`           | Read-only implementation planning (kimi built-in)                         |
| `game-developer` | Engine, rendering, networking, ECS, gameplay systems                      |
| `game-designer`  | Mechanics, balance, progression, economy, level flow (no shell)           |
| `3d-artist`      | Assets, UV/PBR, LOD, animation, pipeline                                  |
| `rust-pro`       | Idiomatic Rust for Bevy / Fyrox / gdext spikes                            |
| `code-reviewer`  | Pre-merge quality/security review with pre-checks                         |
| `debugger`       | Root-cause analysis of crashes, leaks, races                              |

Each subagent file is `~/.kimi/agents/20kbc/<name>.yaml` and inherits
from the built-in `coder.yaml`, then overrides `ROLE_ADDITIONAL` and
`when_to_use`.

## How Claude Should Use This

**Decide between Claude subagents and Kimi delegation:**

- Use Claude's own `Agent` tool when the task is short, needs to stay in
  the current conversation, or benefits from Claude's reasoning style.
- Delegate to Kimi when:
  - You want a **second-model opinion** (different training, different
    failure modes).
  - The task is **long-running** and parallelizable — start it with
    `Bash run_in_background=true` and check back later.
  - You want to **conserve main-context tokens** — Kimi runs in its own
    session and only returns the final summary if invoked headless.
  - The work fits Kimi's strengths (large 262K context, OAuth-backed
    moonshot search/fetch built in).

**Delegation pattern (background, headless):**

```bash
kimi --agent-file ~/.kimi/agents/20kbc/agent.yaml \
     --yolo --afk \
     -w /Users/manwe/CascadeProjects/20KBC \
     --config 'default_model="kimi-code/kimi-for-coding"' \
     <<< "Your task prompt here. Be explicit about deliverables and which subagent to use."
```

For one-shot prompts the cleanest pattern is to pipe via stdin or use
`kimi --help` to find the latest non-interactive flag. If the run is
expected to take > 5 min, launch it with `run_in_background=true` and
poll via the Monitor tool or check the session via `kimi -C`.

**Resume a previous Kimi session:**

```bash
kimi -C -w /Users/manwe/CascadeProjects/20KBC   # continue last session in this dir
kimi -S                                         # interactive session picker
```

## Verification

```bash
kimi --agent-file ~/.kimi/agents/20kbc/agent.yaml info
# → kimi-cli version, agent spec version, wire protocol
```

If the agent tree fails to load, the error names the bad YAML path —
usually a missing `extend` target. The `extend` paths are relative to
the YAML file itself, and currently resolve via
`../../../.local/share/uv/tools/kimi-cli/lib/python3.14/site-packages/kimi_cli/agents/default/`.
If the kimi-cli Python version changes (e.g. `python3.15`), update those
paths in every `*.yaml` under `~/.kimi/agents/20kbc/`.

## Project Context to Pass Kimi

Kimi does not auto-load Claude's `CLAUDE.md`. When delegating, include
the relevant context inline:

- Project root: `/Users/manwe/CascadeProjects/20KBC`
- PRDs: `prds/INDEX.md` and `prds/PRD-0[1-4]-*.md`
- Engine spikes: Bevy, Fyrox, Godot+gdext (all scaffolded and compiling)
- Deep dive: `WC3Analysis/` (Warsmash reimplementation notes)

Or run `/init` inside Kimi once to generate `AGENTS.md` for the repo,
which Kimi loads automatically on subsequent runs.

## Files

- `~/.kimi/agents/20kbc/agent.yaml` — main agent, subagent registry
- `~/.kimi/agents/20kbc/{game-developer,game-designer,3d-artist,rust-pro,code-reviewer,debugger}.yaml`
- `~/.kimi/config.toml` — default model `kimi-code/kimi-for-coding`
  (Kimi-k2.6), OAuth via `/login`
