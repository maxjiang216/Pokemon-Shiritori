//! Agent trait and implementations for Pokémon Shiritori.
//!
//! | Agent          | Strategy                                                              |
//! |----------------|-----------------------------------------------------------------------|
//! | `RandomAgent`  | Uniform random over legal moves                                       |
//! | `GreedyAgent`  | Minimize opponent out-degree (target dead-end letters first)          |
//! | `RolloutAgent` | Exact minimax to `depth`, then `rollouts` random completions per leaf |
//! | `HybridAgent`  | Retrograde + SCC check first; among retro wins, shortest forced mate; else rollout (`name` = "DeadEnd") |
//! | `ExactAgent`   | Full memoized minimax (only usable for small inputs, ~15 Pokémon)     |

use rand::Rng;
use rustc_hash::FxHashMap;

use crate::gen1::pair_index;
use crate::graph::LetterGraph;
use crate::solver::can_win;

// ---------------------------------------------------------------------------
// Shared game-state view passed to agents each turn
// ---------------------------------------------------------------------------

/// Snapshot of the current game state passed to each agent on their turn.
pub struct GameState<'a> {
    /// `None` = opening move (no letter constraint); `Some(l)` = must play a name starting with `l`.
    pub required: Option<u8>,
    pub counts: &'a [u8; 676],
    pub graph: &'a LetterGraph,
}

impl<'a> GameState<'a> {
    /// Collect all legal `(first, last)` moves.
    pub fn legal_moves(&self) -> Vec<(u8, u8)> {
        match self.required {
            None => (0u8..26)
                .flat_map(|f| (0u8..26).map(move |t| (f, t)))
                .filter(|&(f, t)| self.counts[pair_index(f, t)] > 0)
                .collect(),
            Some(l) => (0u8..26)
                .map(|t| (l, t))
                .filter(|&(_, t)| self.counts[pair_index(l, t)] > 0)
                .collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// Agent trait
// ---------------------------------------------------------------------------

pub trait Agent {
    /// Choose a legal move `(first_letter, last_letter)`, or `None` if no moves exist (loss).
    fn choose_move(&mut self, state: &GameState<'_>) -> Option<(u8, u8)>;
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// RandomAgent
// ---------------------------------------------------------------------------

pub struct RandomAgent;

impl Agent for RandomAgent {
    fn choose_move(&mut self, state: &GameState<'_>) -> Option<(u8, u8)> {
        let moves = state.legal_moves();
        if moves.is_empty() {
            return None;
        }
        Some(moves[rand::rng().random_range(0..moves.len())])
    }
    fn name(&self) -> &str {
        "Random"
    }
}

// ---------------------------------------------------------------------------
// GreedyAgent — minimize opponent's total out-degree at the landing letter
// ---------------------------------------------------------------------------

pub struct GreedyAgent;

impl Agent for GreedyAgent {
    fn choose_move(&mut self, state: &GameState<'_>) -> Option<(u8, u8)> {
        state
            .legal_moves()
            .into_iter()
            .min_by_key(|&(_, t)| state.graph.out_degree[t as usize])
    }
    fn name(&self) -> &str {
        "Greedy"
    }
}

// ---------------------------------------------------------------------------
// Rollout helpers (shared between RolloutAgent and HybridAgent)
// ---------------------------------------------------------------------------

fn random_game_win_inner(required: Option<u8>, init_counts: &[u8; 676]) -> bool {
    let mut counts = *init_counts;
    let mut req = required;
    let mut my_turn = true;
    loop {
        let moves: Vec<(u8, u8)> = match req {
            None => (0u8..26)
                .flat_map(|f| (0u8..26).map(move |t| (f, t)))
                .filter(|&(f, t)| counts[pair_index(f, t)] > 0)
                .collect(),
            Some(l) => (0u8..26)
                .map(|t| (l, t))
                .filter(|&(_, t)| counts[pair_index(l, t)] > 0)
                .collect(),
        };
        if moves.is_empty() {
            return !my_turn; // current player has no move → they lose → other player wins
        }
        let (f, t) = moves[rand::rng().random_range(0..moves.len())];
        counts[pair_index(f, t)] -= 1;
        req = Some(t);
        my_turn = !my_turn;
    }
}

/// Score a move `(f,t)` for the player to move: returns P(we win) ∈ [0,1].
/// Uses exact minimax to `depth`, then `rollouts` random games at the leaf.
fn score_move(
    f: u8,
    t: u8,
    counts: &mut [u8; 676],
    graph: &mut LetterGraph,
    memo: &mut FxHashMap<(u8, u64), bool>,
    depth: usize,
    rollouts: usize,
) -> f64 {
    let idx = pair_index(f, t);
    counts[idx] -= 1;
    graph.on_decrement(f, t, counts);

    let score = match can_win(Some(t), counts, graph, memo, depth) {
        Some(false) => 1.0, // opponent loses after this move → we win
        Some(true) => 0.0,  // opponent wins → we lose
        None => {
            // Depth limit: estimate via random rollouts.
            // P(opponent-to-move at t wins) estimated by rollouts.
            // We want P(we win) = 1 - P(opponent wins).
            let wins: usize = (0..rollouts)
                .filter(|_| random_game_win_inner(Some(t), counts))
                .count();
            // wins/rollouts = P(player at t wins) = P(opponent wins)
            if rollouts == 0 {
                0.5
            } else {
                1.0 - wins as f64 / rollouts as f64
            }
        }
    };

    counts[idx] += 1;
    graph.on_increment(f, t);
    score
}

// ---------------------------------------------------------------------------
// RolloutAgent
// ---------------------------------------------------------------------------

pub struct RolloutAgent {
    pub depth: usize,
    pub rollouts: usize,
}

impl RolloutAgent {
    pub fn new(depth: usize, rollouts: usize) -> Self {
        RolloutAgent { depth, rollouts }
    }
}

impl Agent for RolloutAgent {
    fn choose_move(&mut self, state: &GameState<'_>) -> Option<(u8, u8)> {
        let moves = state.legal_moves();
        if moves.is_empty() {
            return None;
        }
        let mut counts = *state.counts;
        let mut graph = state.graph.clone();
        let mut memo: FxHashMap<(u8, u64), bool> = FxHashMap::default();
        let depth = self.depth;
        let rollouts = self.rollouts;
        moves.into_iter().max_by(|&(f1, t1), &(f2, t2)| {
            let s1 = score_move(f1, t1, &mut counts, &mut graph, &mut memo, depth, rollouts);
            let s2 = score_move(f2, t2, &mut counts, &mut graph, &mut memo, depth, rollouts);
            s1.partial_cmp(&s2).unwrap_or(std::cmp::Ordering::Equal)
        })
    }
    fn name(&self) -> &str {
        "Rollout"
    }
}

// ---------------------------------------------------------------------------
// HybridAgent — retrograde + SCC check, then rollout fallback (public name: "DeadEnd")
// ---------------------------------------------------------------------------

pub struct HybridAgent {
    pub depth: usize,
    pub rollouts: usize,
}

impl HybridAgent {
    pub fn new(depth: usize, rollouts: usize) -> Self {
        HybridAgent { depth, rollouts }
    }
}

impl Agent for HybridAgent {
    fn choose_move(&mut self, state: &GameState<'_>) -> Option<(u8, u8)> {
        let moves = state.legal_moves();
        if moves.is_empty() {
            return None;
        }
        let (labels, lose_mate) = state.graph.retrograde_with_lose_mate_plies();

        // 1. Retrograde-winning moves: prefer shortest forced win (fewest plies).
        let win_candidates: Vec<(u8, u8)> = moves
            .iter()
            .filter(|&&(_, t)| labels[t as usize] == Some(false))
            .copied()
            .collect();
        if let Some(&m) = win_candidates.iter().min_by(|&&(f1, t1), &&(f2, t2)| {
            let k1 = lose_mate[t1 as usize].unwrap_or(u16::MAX);
            let k2 = lose_mate[t2 as usize].unwrap_or(u16::MAX);
            (k1, f1, t1).cmp(&(k2, f2, t2))
        }) {
            return Some(m);
        }

        // 2. All moves lead to proven winner for opponent → we're lost; pick any.
        let all_proven_losing = moves.iter().all(|&(_, t)| labels[t as usize] == Some(true));
        if all_proven_losing {
            return moves.into_iter().next();
        }

        // 3. Restrict to non-proven-losing moves and run rollout.
        let pool: Vec<(u8, u8)> = moves
            .into_iter()
            .filter(|&(_, t)| labels[t as usize] != Some(true))
            .collect();

        let mut counts = *state.counts;
        let mut graph = state.graph.clone();
        let mut memo: FxHashMap<(u8, u64), bool> = FxHashMap::default();
        let depth = self.depth;
        let rollouts = self.rollouts;
        pool.into_iter().max_by(|&(f1, t1), &(f2, t2)| {
            let s1 = score_move(f1, t1, &mut counts, &mut graph, &mut memo, depth, rollouts);
            let s2 = score_move(f2, t2, &mut counts, &mut graph, &mut memo, depth, rollouts);
            s1.partial_cmp(&s2).unwrap_or(std::cmp::Ordering::Equal)
        })
    }
    fn name(&self) -> &str {
        "DeadEnd"
    }
}

// ---------------------------------------------------------------------------
// ExactAgent — full memoized minimax (use with --count ≤ ~15)
// ---------------------------------------------------------------------------

pub struct ExactAgent {
    memo: FxHashMap<(u8, u64), bool>,
}

impl ExactAgent {
    pub fn new() -> Self {
        ExactAgent {
            memo: FxHashMap::default(),
        }
    }
}

impl Default for ExactAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for ExactAgent {
    fn choose_move(&mut self, state: &GameState<'_>) -> Option<(u8, u8)> {
        let moves = state.legal_moves();
        if moves.is_empty() {
            return None;
        }
        let mut counts = *state.counts;
        let mut graph = state.graph.clone();
        let mut fallback = None;
        for (f, t) in moves {
            let idx = pair_index(f, t);
            counts[idx] -= 1;
            graph.on_decrement(f, t, &counts);
            let opp = can_win(Some(t), &mut counts, &mut graph, &mut self.memo, usize::MAX);
            counts[idx] += 1;
            graph.on_increment(f, t);
            if opp == Some(false) {
                return Some((f, t));
            }
            if fallback.is_none() {
                fallback = Some((f, t));
            }
        }
        fallback
    }
    fn name(&self) -> &str {
        "Exact"
    }
}
