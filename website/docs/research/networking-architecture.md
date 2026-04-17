---
sidebar_position: 2
---

# Networking Architecture for RTS



## The RTS Networking Problem

RTS games are uniquely challenging for networking because the game state is massive (hundreds/thousands of units, resources, buildings) but must stay perfectly synchronized across all players.


## Option 1: Deterministic Lockstep (Recommended for 1v1/2v2)

### How It Works

1. All clients run the **same simulation** locally
2. Each tick, clients exchange only **player input commands** (not game state)
3. Because the simulation is deterministic, all clients arrive at the same state
4. If inputs haven't arrived from a player, the simulation **stalls** until they do

### Bandwidth

- Scales with **number of player inputs**, not number of entities
- A game with 1,000 units uses the same bandwidth as one with 10 units
- Typical: ~1-5 KB/s per player

### Latency

- Input delay = ~0.5 * RTT (round-trip time)
- At 50ms ping: 25ms delay (imperceptible)
- At 200ms ping: 100ms delay (noticeable but playable for RTS)
- Input is buffered 2-3 ticks ahead for smooth play

### Requirements for Determinism

| Requirement | Solution |
|-------------|----------|
| No floating-point divergence | Use `fixed` crate (fixed-point math) in game-core |
| Deterministic iteration order | Use `BTreeMap` instead of `HashMap` |
| Deterministic random | Seeded PRNG (e.g., `rand_chacha`) shared across clients |
| Same update order | Fixed entity processing order (sorted by ID) |
| No platform-specific behavior | Pure Rust game-core, no OS calls in simulation |

### Desync Detection

- Checksum game state every N ticks (e.g., hash all entity positions + health)
- Compare checksums across clients
- On mismatch: flag desync, optionally resync from replay

### Used By

- StarCraft (1/2), Age of Empires (1/2/3/4), Warcraft III, Spring RTS Engine, OpenRA
- This is the **industry standard** for competitive RTS

### Pros
- Minimal bandwidth regardless of game complexity
- Enables replay system for free (just save inputs)
- Well-proven architecture for RTS

### Cons
- Requires fully deterministic simulation (hard to achieve, hard to debug)
- All players must be in sync — one slow player stalls everyone
- Reconnection is expensive (must replay all inputs from start)
- Doesn't scale well beyond ~8 players (input collection latency grows)


## Option 2: Server-Authoritative (Recommended for Survival Mode)

### How It Works

1. A **dedicated server** runs the simulation
2. Clients send input commands to the server
3. Server processes inputs, updates state, sends **state snapshots** to clients
4. Clients render the received state (with interpolation/prediction)

### Bandwidth

- Scales with **number of entities * number of clients**
- Can be optimized with delta compression (only send what changed)
- Typical: 10-100 KB/s per player depending on entity count

### Latency

- Client-side prediction for local player's units (feels responsive)
- Server reconciliation when prediction was wrong
- At 100ms ping: feels responsive with prediction, some rubber-banding on corrections

### Requirements

| Requirement | Solution |
|-------------|----------|
| Dedicated server | Headless game-core instance (game-server crate) |
| State serialization | Serde + bincode for compact binary format |
| Delta compression | Track dirty flags per entity, only send changes |
| Client prediction | Predict local unit movement, reconcile on server update |
| Interest management | Only send entities within player's view + buffer zone |

### Used By

- Most MMOs, battle royales, FPS games
- End of Nations (50-player RTS) used proxy-server architecture

### Pros
- No determinism requirement — server is authoritative
- Handles player disconnect gracefully (server keeps running)
- Scales to many players (20-30+) with interest management
- Anti-cheat is simpler (server validates all actions)

### Cons
- Higher bandwidth
- More complex client code (prediction, interpolation)
- Requires hosting a dedicated server
- State snapshots are harder to optimize for RTS (many small entities)


## Recommended Approach for 20KBC

### Hybrid Architecture

| Mode | Architecture | Why |
|------|-------------|-----|
| 1v1 Ranked | Deterministic Lockstep | Low latency, proven for competitive RTS, enables replays |
| 2v2 Ranked | Deterministic Lockstep | Same benefits, 4 players is well within lockstep limits |
| Survival (20-30 players) | Server-Authoritative | Lockstep doesn't scale to 30 players |
| Campaign Co-op | Deterministic Lockstep | 2 players, simple |
| LAN | Deterministic Lockstep | Low latency LAN makes lockstep ideal |

### Implementation Order

1. **Phase 4.1**: Make game-core deterministic (fixed-point, seeded RNG, BTreeMap)
2. **Phase 4.2**: Implement lockstep for LAN (simplest networking scenario)
3. **Phase 4.3**: Add online lockstep with relay server
4. **Phase 4.5**: Add server-authoritative mode for survival (reuses game-core headless)

### Rust Crate Options

| Crate | Purpose | Notes |
|-------|---------|-------|
| `fixed` | Fixed-point arithmetic | Replace f32/f64 in simulation |
| `rand_chacha` | Deterministic PRNG | Same seed = same sequence across platforms |
| `bincode` + `serde` | State serialization | Compact binary for network packets |
| `laminar` | UDP networking | Reliable/unreliable channels, connection management |
| `renet` | Game networking | Higher-level, Bevy-friendly but usable standalone |
| `bevy_replicon` | Replication framework | Bevy-specific, good API if using Bevy |
| `quinn` | QUIC protocol | Alternative to raw UDP, handles reliability/ordering |


## Infrastructure Needs

| Component | Cost | When |
|-----------|------|------|
| STUN/TURN relay server | ~$20/month (small VPS) | Phase 4.3 (online play) |
| Matchmaking/ranking API | ~$20/month (same VPS) | Phase 4.4 (ranked) |
| Dedicated game servers | ~$50-100/month (survival) | Phase 4.5 (survival mode) |


## Key References

- [Gaffer On Games: Deterministic Lockstep](https://gafferongames.com/post/deterministic_lockstep/)
- [1500 Archers: AoE2 Networking](https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond)
- [Spring RTS Engine](https://springrts.com/) — open-source RTS engine using lockstep
- [OpenRA](https://www.openra.net/) — open-source C&C/RA engine, lockstep networking
