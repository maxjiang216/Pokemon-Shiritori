# Pokémon Shiritori Solver

A Rust solver for two-player Pokémon Shiritori: players take turns naming Pokémon where each name must start with the last **letter** of the previous name (English, case-insensitive, punctuation stripped). No repeats; the first player with no legal move loses.

The same engine powers a **command-line** workflow (exact solving, tournaments, analysis) and a **browser UI** where you play human vs CPU in the tab—no server required.

## Web interface

The app lives in `web/`: **React**, **Vite**, and the solver compiled to **WebAssembly** (`wasm32-unknown-unknown` with the `wasm` feature). Game logic runs entirely in the browser.

**What you can do**

- **Play** against the CPU with the same letter rules; choose whether you or the CPU opens.
- **Generations 1–6** — Toggle which national-dex generations are in the word pool (English names). The default includes all six.
- **CPU engines** — Random, Greedy, DeadEnd (retrograde + rollout fallback), or Rollout; adjust **rollouts** to trade speed vs strength (minimax depth is fixed at 4 in the WASM build, matching the CLI `play` default).
- **Fair openings** — Each match starts from a random first Pokémon whose last letter still gives the next player at least one legal reply (no instant dead-letter wins on move one).
- **Entry** vs **Picker** — Type names or pick from a searchable list of legal moves; optional **Hint** shows a suggested move from the engine.
- **Pokédex** — Slide-out reference for names in your current pool.
- **Records** — Wins, losses, current streak, and best streak (stored in `localStorage`); optional reset.
- **Rules** modal, **light/dark** theme, last-move **artwork** and **type** badges (from embedded metadata), scrollable **chain history**, and a **victory** summary with move count and elapsed time.

**Run it locally**

```sh
# From repo root: install wasm target + wasm-bindgen CLI once
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli

cd web
npm install
npm run build:wasm   # builds ../target/... and refreshes web/src/wasm-pkg/
npm run dev          # http://localhost:5173 (typical Vite port)
```

For production assets, `npm run build` outputs to `web/dist/` (after `build:wasm`).

## Model

Each name maps to a directed edge `(first_letter → last_letter)` on a 26-node multigraph. This is exactly **Directed Edge Geography**, which is PSPACE-complete in general. The 26-letter constraint enables practical optimizations:

- **Reachability-restricted state key** — only edges reachable from the current required letter enter the memo key
- **Retrograde propagation** — exact W/L labels on the 26-node letter graph in O(26²)
- **Forced-move compression** — chains of single-option letters are resolved without branching
- **Zobrist hashing + FxHashMap** — O(1) hash updates per move
- **Move ordering** — lowest-out-degree targets tried first (trap moves)

## Gen 1 graph facts

(Using only generation 1 — 151 Pokémon — as a small illustrative pool.)

- 151 Pokémon → 119 distinct `(first, last)` edge types; log₂(state space) ≈ 136
- Dead-end letters (no starters, immediate loss if sent there): **q, u, x, y**
- Forced-move letter (exactly one starter type): **i** (only Ivysaur, i→r)

Larger pools (e.g. generations 1–6) change letter coverage and statistics; use `stats` or the web app to explore your chosen subset.

## Agents

| Agent | Strategy |
|---|---|
| `Random` | Uniform random over legal moves |
| `Greedy` | Minimize opponent's out-degree at the landing letter |
| `DeadEnd` | Retrograde first (exact when labeled); rollout fallback (`HybridAgent` in code) |
| `Rollout` | Exact minimax to depth D, then random rollouts per leaf |
| `Exact` | Full memoized minimax (use `--count ≤ 15` only) |

The **web app** exposes Random, Greedy, DeadEnd, and Rollout. The CLI tournament and `play` also support `exact` when the pool is small enough.

## Usage

```sh
cargo build --release

# Name pool: comma-separated generations 1–6, or "all" (default). Optional --count caps list size.
cargo run --release -- solve --count 5
cargo run --release -- solve --gens 1 --count 151

# Interactive human vs CPU in the terminal
cargo run --release -- play --cpu deadend --gens 1,2,3 --first human

# Run tournament between agents
cargo run --release -- tournament --count 151 --depth 4 --rollouts 30 --games 1

# Optimal opening classification on the letter multigraph (per name)
cargo run --release -- stats --gens 1 --count 151

# Sensitivity analysis: how often does one word change flip the outcome?
cargo run --release -- sensitivity --count 151 --depths 10,30,60,100,140 --samples 100 --rollouts 40

# List CPU agent names and descriptions
cargo run --release -- cpus
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
Agent \ vs       Random   Greedy  DeadEnd  Rollout
Random              ---    12.9%     8.6%    10.3%
Greedy            87.1%      ---    42.1%    45.4%
DeadEnd           91.4%    57.9%      ---    50.0%
Rollout           89.7%    54.6%    50.0%      ---
```

`DeadEnd` matches `Rollout` using only O(26²) retrograde — rollout adds little once retrograde handles the tractable endgame positions.

## Data

- `data/gens_1_6_en.json` — English names for national dex generations 1–6 (used by the CLI default pool and the web app).
- `data/gen1_en.json` — Generation 1 only (legacy helper / scripts).

Regenerate Gen 1 names with `scripts/fetch_gen1_json.sh` where applicable.
