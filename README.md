# Pokémon Shiritori Solver

A Rust solver for two-player Pokémon Shiritori: players take turns naming Pokémon where each name must start with the last **letter** of the previous name (English, case-insensitive, punctuation stripped). No repeats; the first player with no legal move loses.

## Model

Each name maps to a directed edge `(first_letter → last_letter)` on a 26-node multigraph. This is exactly **Directed Edge Geography**, which is PSPACE-complete in general. The 26-letter constraint enables practical optimizations:

- **Reachability-restricted state key** — only edges reachable from the current required letter enter the memo key
- **Retrograde propagation** — exact W/L labels on the 26-node letter graph in O(26²)
- **Forced-move compression** — chains of single-option letters are resolved without branching
- **Zobrist hashing + FxHashMap** — O(1) hash updates per move
- **Move ordering** — lowest-out-degree targets tried first (trap moves)

## Gen 1 graph facts

- 151 Pokémon → 119 distinct `(first, last)` edge types; log₂(state space) ≈ 136
- Dead-end letters (no starters, immediate loss if sent there): **q, u, x, y**
- Forced-move letter (exactly one starter type): **i** (only Ivysaur, i→r)

## Agents

| Agent | Strategy |
|---|---|
| `Random` | Uniform random over legal moves |
| `Greedy` | Minimize opponent's out-degree at the landing letter |
| `DeadEndHunter` | Retrograde first (exact when labeled); greedy fallback |
| `Rollout` | Exact minimax to depth D, then random rollouts per leaf |
| `Hybrid` | Retrograde + rollout combined |
| `Exact` | Full memoized minimax (use `--count ≤ 15` only) |

## Usage

```sh
cargo build --release

# Solve with N Pokémon (warns if N > 15)
cargo run --release -- solve --count 5

# Run tournament between all agents
cargo run --release -- tournament --count 151 --depth 4 --rollouts 30 --games 1

# Sensitivity analysis: how often does one word change flip the outcome?
cargo run --release -- sensitivity --count 151 --depths 10,30,60,100,140 --samples 100 --rollouts 40
```

## Sample results (Gen 1, 151 Pokémon)

**Sensitivity** (fraction of single-edge perturbations that flip rollout-estimated outcome):

| Depth | Sensitivity |
|---|---|
| 10 | ~14% |
| 30 | ~1% |
| 60 | 0% |

Outcome is essentially determined by depth 30 — mid-game strategy is hard; endgame dead-end detection dominates.

**Tournament win rates** (row beats column):

```
Agent \ vs       Random   Greedy  DEHunter  Rollout   Hybrid
Random              ---    12.9%     8.6%    10.3%    10.3%
Greedy            87.1%      ---    42.1%    45.4%    42.1%
DeadEndHunter     91.4%    57.9%      ---    50.0%    50.0%
Rollout           89.7%    54.6%    50.0%      ---    50.0%
Hybrid            89.7%    57.9%    50.0%    50.0%      ---
```

`DeadEndHunter` matches `Rollout`/`Hybrid` using only O(26²) retrograde — rollout adds little once retrograde handles the tractable endgame positions.

## Data

`data/gen1_en.json` — 151 canonical English names in national dex order, sourced from PokéAPI.  
Regenerate with `scripts/fetch_gen1_json.sh`.
