#!/usr/bin/env bash
# Dispatch Kimi against a single PRD WITHOUT the 20kbc agent tree.
# Useful when the agent tree is misbehaving and we want pure kimi-cli.
# Usage: dispatch-kimi-plain.sh <worktree-path> <prd-file-relative> <log-file>
set -u
worktree="$1"
prd="$2"
log="$3"

mkdir -p "$(dirname "$log")"

prompt="Read the file ${prd} in the working directory (${worktree}). It is a complete PRD. Execute it exactly as specified. Honour the file whitelist and forbidden list. Run cargo build at the end. Do NOT commit. When acceptance criteria pass, stop and print a one-line summary."

echo "[dispatch-plain] worktree=$worktree prd=$prd log=$log" >&2
exec kimi --print --final-message-only -y --afk \
     -w "$worktree" -p "$prompt" >"$log" 2>&1
