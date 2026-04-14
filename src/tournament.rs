//! Tournament framework: run agents against each other across all starting words.
//!
//! For each of the N Pokémon as the forced opening word (by its `(first,last)` edge type):
//! - **Game A**: Agent 1 plays that word first; Agent 2 responds.
//! - **Game B**: Agent 2 plays that word first; Agent 1 responds.
//!
//! Total: 2 × N × games_per_start games per agent pair.

use crate::agents::{Agent, GameState};
use crate::gen1::pair_index;
use crate::graph::LetterGraph;
use crate::normalize::first_last_letters;

// ---------------------------------------------------------------------------
// Single game
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    One,
    Two,
}

/// Play a game with a forced opening move `(open_f, open_t)`.
///
/// The opening move has already been "played" — `p1` responds first.
/// Returns which player wins.
pub fn play_game(
    base_counts: &[u8; 676],
    open_f: u8,
    open_t: u8,
    p1: &mut dyn Agent,
    p2: &mut dyn Agent,
) -> Player {
    let mut counts = *base_counts;
    let open_idx = pair_index(open_f, open_t);
    if counts[open_idx] == 0 {
        // This opening word isn't in the current dictionary; opener loses immediately.
        return Player::Two;
    }
    counts[open_idx] -= 1;
    let mut graph = LetterGraph::from_counts(&counts);

    // p1 must respond to open_t; p2 played the opening.
    let mut required = Some(open_t);
    let mut active = Player::One; // whose turn it is to respond

    loop {
        let state = GameState {
            required,
            counts: &counts,
            graph: &graph,
        };
        let agent: &mut dyn Agent = if active == Player::One { p1 } else { p2 };
        match agent.choose_move(&state) {
            None => {
                // Active player has no move → they lose → other player wins.
                return if active == Player::One {
                    Player::Two
                } else {
                    Player::One
                };
            }
            Some((f, t)) => {
                // Validate move
                debug_assert_eq!(required, Some(f), "agent returned illegal first letter");
                debug_assert!(counts[pair_index(f, t)] > 0, "agent used an exhausted edge");
                counts[pair_index(f, t)] -= 1;
                graph.on_decrement(f, t, &counts);
                required = Some(t);
                active = if active == Player::One {
                    Player::Two
                } else {
                    Player::One
                };
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tournament result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TournamentResult {
    pub name_a: String,
    pub name_b: String,
    pub wins_a: u32,
    pub wins_b: u32,
    pub total: u32,
}

impl TournamentResult {
    pub fn win_pct_a(&self) -> f64 {
        if self.total == 0 {
            return 0.5;
        }
        self.wins_a as f64 / self.total as f64
    }
    pub fn win_pct_b(&self) -> f64 {
        if self.total == 0 {
            return 0.5;
        }
        self.wins_b as f64 / self.total as f64
    }
}

// ---------------------------------------------------------------------------
// Run tournament
// ---------------------------------------------------------------------------

/// Run a tournament between agent A and agent B.
///
/// For each Pokémon name in `names`, play `games_per_start` rounds where:
/// - Game A: the named word is the opening, agent A responds first.
/// - Game B: the named word is the opening, agent B responds first.
///
/// Returns per-pair aggregate result.
pub fn run_tournament(
    names: &[String],
    a: &mut dyn Agent,
    b: &mut dyn Agent,
    base_counts: &[u8; 676],
    games_per_start: u32,
) -> TournamentResult {
    let name_a = a.name().to_string();
    let name_b = b.name().to_string();
    let mut wins_a = 0u32;
    let mut wins_b = 0u32;
    let mut total = 0u32;

    for name in names {
        let Some((f, l)) = first_last_letters(name) else {
            continue;
        };
        // Only play if this edge exists in the dictionary.
        if base_counts[pair_index(f, l)] == 0 {
            continue;
        }

        for _ in 0..games_per_start {
            // Game A: opening word played, a responds first.
            let winner = play_game(base_counts, f, l, a, b);
            // The opener "won" the opening slot but still needs to play well.
            // Winner here means: who wins the rest of the game after the opening move.
            // The opener (who played the first word) is effectively Player Two in play_game.
            // We count wins from A's perspective across both roles.
            match winner {
                Player::One => wins_a += 1,
                Player::Two => wins_b += 1,
            }
            total += 1;

            // Game B: opening word played, b responds first.
            let winner = play_game(base_counts, f, l, b, a);
            match winner {
                Player::One => wins_b += 1,
                Player::Two => wins_a += 1,
            }
            total += 1;
        }
    }

    TournamentResult {
        name_a,
        name_b,
        wins_a,
        wins_b,
        total,
    }
}

// ---------------------------------------------------------------------------
// Multi-agent round-robin
// ---------------------------------------------------------------------------

pub struct RoundRobinResult {
    pub agents: Vec<String>,
    /// `wins[i][j]` = wins of agent i against agent j.
    pub wins: Vec<Vec<u32>>,
    /// `games[i][j]` = total games between i and j.
    pub games: Vec<Vec<u32>>,
}

impl RoundRobinResult {
    pub fn win_pct(&self, i: usize, j: usize) -> f64 {
        let g = self.games[i][j];
        if g == 0 {
            return 0.5;
        }
        self.wins[i][j] as f64 / g as f64
    }

    /// Print a compact win-% table to stdout.
    pub fn print_table(&self) {
        let n = self.agents.len();
        let col_w = 12usize;
        // Header
        print!("{:<16}", "Agent \\ vs");
        for a in &self.agents {
            print!(" {:>col_w$}", a, col_w = col_w);
        }
        println!();
        // Rows
        for i in 0..n {
            print!("{:<16}", self.agents[i]);
            for j in 0..n {
                if i == j {
                    print!(" {:>col_w$}", "---", col_w = col_w);
                } else {
                    let pct = self.win_pct(i, j) * 100.0;
                    print!(" {:>col_w$.1}%", pct, col_w = col_w - 1);
                }
            }
            println!();
        }
    }
}

/// Run a full round-robin between all agents.
pub fn round_robin(
    names: &[String],
    agents: &mut Vec<Box<dyn Agent>>,
    base_counts: &[u8; 676],
    games_per_start: u32,
) -> RoundRobinResult {
    let n = agents.len();
    let agent_names: Vec<String> = agents.iter().map(|a| a.name().to_string()).collect();
    let mut wins = vec![vec![0u32; n]; n];
    let mut games = vec![vec![0u32; n]; n];

    for i in 0..n {
        for j in (i + 1)..n {
            let mut wins_i = 0u32;
            let mut wins_j = 0u32;
            let mut total = 0u32;

            for name in names {
                let Some((f, l)) = first_last_letters(name) else {
                    continue;
                };
                if base_counts[pair_index(f, l)] == 0 {
                    continue;
                }

                for _ in 0..games_per_start {
                    // Safety: we need mutable refs to two distinct agents.
                    // Use index-based splitting.
                    let (left, right) = agents.split_at_mut(j);
                    let ai = &mut *left[i];
                    let aj = &mut *right[0];

                    // Game A: i responds to opener, j played opener
                    let w = play_game(base_counts, f, l, ai, aj);
                    match w {
                        Player::One => wins_i += 1,
                        Player::Two => wins_j += 1,
                    }
                    total += 1;

                    // Game B: j responds to opener, i played opener
                    let (left, right) = agents.split_at_mut(j);
                    let ai = &mut *left[i];
                    let aj = &mut *right[0];
                    let w = play_game(base_counts, f, l, aj, ai);
                    match w {
                        Player::One => wins_j += 1,
                        Player::Two => wins_i += 1,
                    }
                    total += 1;
                }
            }

            wins[i][j] = wins_i;
            wins[j][i] = wins_j;
            games[i][j] = total;
            games[j][i] = total;
        }
    }

    RoundRobinResult {
        agents: agent_names,
        wins,
        games,
    }
}
