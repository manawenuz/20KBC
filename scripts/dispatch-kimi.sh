#!/usr/bin/env bash
# Dispatch Kimi against a single PRD inside a worktree.
# Usage: dispatch-kimi.sh <worktree-path> <prd-file-relative> <log-file> [subagent-hint]
#
# subagent-hint (optional): one of coder | rust-pro | game-developer | 3d-artist |
#                           code-reviewer | debugger | explore | plan
#                           Embedded into the prompt as a routing hint.
set -u
worktree="$1"
prd="$2"
log="$3"
subagent="${4:-rust-pro}"

mkdir -p "$(dirname "$log")"

agent_file="$HOME/.kimi/agents/20kbc/agent.yaml"
# Fall back to default agent if the 20kbc tree isn't installed.
[ -f "$agent_file" ] || agent_file=""

prompt="You are an autonomous coding agent executing a PRD.

Suggested subagent for the bulk of the work: ${subagent}. Delegate to other
subagents when appropriate (e.g. code-reviewer before declaring done).

1. Read the file ${prd} in the working directory (${worktree}). It is a complete spec.
2. Execute it exactly as specified.
3. The PRD has a strict 'Files you MAY create/modify' whitelist and a 'Files you MUST NOT touch' list. Honour them.
4. Run the build/acceptance commands from the PRD. If they fail, fix and retry, up to 3 attempts.
5. When all acceptance criteria pass, stop and print a one-line summary.
6. Do NOT commit or push. Just leave the working tree dirty with your changes.
7. Do NOT modify files outside the whitelist. If you think you need to, STOP and write WHY to stdout.

Begin."

echo "[dispatch] worktree=$worktree prd=$prd log=$log subagent=$subagent" >&2

if [ -n "$agent_file" ]; then
    exec kimi --agent-file "$agent_file" --print --final-message-only -y --afk \
         -w "$worktree" -p "$prompt" >"$log" 2>&1
else
    exec kimi --print --final-message-only -y --afk \
         -w "$worktree" -p "$prompt" >"$log" 2>&1
fi
