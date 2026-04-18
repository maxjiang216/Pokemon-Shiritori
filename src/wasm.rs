//! WASM bindings for the web frontend, compiled only with the `wasm` feature.
//!
//! Exposes a stateful `GameHandle` object that the TypeScript UI uses to drive
//! a full Human-vs-CPU game without any server round-trips.

use std::collections::{HashMap, HashSet};

use js_sys::Array;
use rand::Rng;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::agents::{Agent, GameState, GreedyAgent, HybridAgent, RandomAgent, RolloutAgent};
use crate::gen1::{counts_from_names, pair_index};
use crate::gens::{default_generation_list, names_for_generations, parse_generation_list};
use crate::graph::{non_terminal_opening_indices, LetterGraph};
use crate::normalize::first_last_letters;

// ---------------------------------------------------------------------------
// Agent enum — avoids Box<dyn Agent> across the wasm-bindgen boundary
// ---------------------------------------------------------------------------

enum CpuKind {
    Random(RandomAgent),
    Greedy(GreedyAgent),
    Rollout(RolloutAgent),
    /// Retrograde + SCC first, rollout fallback (`HybridAgent`, display name DeadEnd).
    DeadEnd(HybridAgent),
}

impl CpuKind {
    fn choose_move(&mut self, state: &GameState<'_>) -> Option<(u8, u8)> {
        match self {
            CpuKind::Random(a) => a.choose_move(state),
            CpuKind::Greedy(a) => a.choose_move(state),
            CpuKind::Rollout(a) => a.choose_move(state),
            CpuKind::DeadEnd(a) => a.choose_move(state),
        }
    }

    fn display_name(&self) -> &str {
        match self {
            CpuKind::Random(_) => "Random",
            CpuKind::Greedy(_) => "Greedy",
            CpuKind::Rollout(_) => "Rollout",
            CpuKind::DeadEnd(_) => "DeadEnd",
        }
    }
}

// ---------------------------------------------------------------------------
// Name-lookup helpers (mirrors play.rs without stdin/stdout)
// ---------------------------------------------------------------------------

fn lookup_key(s: &str) -> String {
    s.chars()
        .flat_map(|c| c.to_lowercase())
        .filter(|c| c.is_ascii_alphabetic())
        .collect()
}

fn build_key_to_names(names: &[String]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for name in names {
        let key = lookup_key(name);
        if !key.is_empty() {
            map.entry(key).or_default().push(name.clone());
        }
    }
    map
}

/// Legal display names: letter rules, edge still has stock, and this **exact name** was not played.
fn legal_names_for(
    required: Option<u8>,
    counts: &[u8; 676],
    names: &[String],
    used_keys: &HashSet<String>,
) -> Vec<String> {
    let mut out = Vec::new();
    for name in names {
        if used_keys.contains(&lookup_key(name)) {
            continue;
        }
        let Some((f, l)) = first_last_letters(name) else {
            continue;
        };
        if counts[pair_index(f, l)] == 0 {
            continue;
        }
        match required {
            None => out.push(name.clone()),
            Some(r) if f == r => out.push(name.clone()),
            _ => {}
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Serializable result types (returned as JsValue via serde_wasm_bindgen)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct MoveResult {
    ok: bool,
    error: Option<String>,
    name: Option<String>,
}

#[derive(Serialize)]
struct CpuMoveResult {
    name: Option<String>,
    lost: bool,
}

#[derive(Serialize)]
struct HistoryEntry {
    name: String,
    by_human: bool,
}

// ---------------------------------------------------------------------------
// GameHandle — main WASM export
// ---------------------------------------------------------------------------

#[wasm_bindgen]
pub struct GameHandle {
    names: Vec<String>,
    counts: [u8; 676],
    graph: LetterGraph,
    required: Option<u8>,
    cpu: CpuKind,
    history: Vec<(String, bool)>,
    human_turn: bool,
    over: bool,
    human_won_flag: bool,
    key_to_names: HashMap<String, Vec<String>>,
    /// `lookup_key(name)` for each Pokémon already played (distinct names, not only edges).
    used_name_keys: HashSet<String>,
}

#[wasm_bindgen]
impl GameHandle {
    /// Create a new game.
    ///
    /// - `agent`: one of `random`, `greedy`, `deadend` (aliases: `hybrid`, `deadendhunter`, `hunter`), `rollout`
    /// - `depth`: minimax depth for rollout/deadend agents
    /// - `rollouts`: random simulations per leaf for rollout/deadend
    /// - `generations`: comma-separated list `1`–`6` or `all` (e.g. `1,3,6`). Empty = all.
    /// - `human_first`: whether the human moves first (after a random safe opening — see below)
    ///
    /// Each game begins with a **random opening** Pokémon whose last letter still allows the
    /// next player at least one legal move (no instant one-move wins from dead-ending letters).
    /// The first configured player (human or CPU) is treated as having played that opening.
    #[wasm_bindgen(constructor)]
    pub fn new(agent: &str, depth: u32, rollouts: u32, generations: &str, human_first: bool) -> Self {
        let gen_list = parse_generation_list(generations).unwrap_or_else(|_| default_generation_list());
        let names = names_for_generations(&gen_list).expect("embedded gens data");
        let counts = counts_from_names(&names);
        let graph = LetterGraph::from_counts(&counts);
        let key_to_names = build_key_to_names(&names);

        let d = depth as usize;
        let r = rollouts as usize;
        let cpu = match agent.to_lowercase().as_str() {
            "random" => CpuKind::Random(RandomAgent),
            "greedy" => CpuKind::Greedy(GreedyAgent),
            "rollout" => CpuKind::Rollout(RolloutAgent::new(d, r)),
            "deadend" | "deadendhunter" | "hunter" | "hybrid" => {
                CpuKind::DeadEnd(HybridAgent::new(d, r))
            }
            _ => CpuKind::DeadEnd(HybridAgent::new(d, r)),
        };

        let mut handle = GameHandle {
            names,
            counts,
            graph,
            required: None,
            cpu,
            history: Vec::new(),
            human_turn: human_first,
            over: false,
            human_won_flag: false,
            key_to_names,
            used_name_keys: HashSet::new(),
        };
        handle.apply_random_forced_opening(human_first);
        handle
    }

    pub fn is_human_turn(&self) -> bool {
        self.human_turn && !self.over
    }

    pub fn is_over(&self) -> bool {
        self.over
    }

    pub fn human_won(&self) -> bool {
        self.human_won_flag
    }

    /// Returns `null` (opening move) or an uppercase single character like `"U"`.
    pub fn required_letter(&self) -> JsValue {
        match self.required {
            None => JsValue::NULL,
            Some(l) => JsValue::from_str(&((b'A' + l) as char).to_string()),
        }
    }

    pub fn cpu_name(&self) -> String {
        self.cpu.display_name().to_string()
    }

    /// Number of Pokémon still in play (not yet used).
    pub fn remaining_count(&self) -> u32 {
        self.counts.iter().map(|&c| c as u32).sum()
    }

    /// Total moves played so far.
    pub fn used_count(&self) -> u32 {
        self.history.len() as u32
    }

    /// Pool names in national dex order for the selected generations (subset of #1–721).
    pub fn pool_names(&self) -> Array {
        let arr = Array::new();
        for name in &self.names {
            arr.push(&JsValue::from_str(name));
        }
        arr
    }

    /// Legal names for the current position, alphabetically sorted.
    pub fn legal_names(&self) -> Array {
        let arr = Array::new();
        let mut names = legal_names_for(self.required, &self.counts, &self.names, &self.used_name_keys);
        names.sort();
        for n in &names {
            arr.push(&JsValue::from_str(n));
        }
        arr
    }

    /// Move history as `Array<{name: string, by_human: boolean}>`.
    pub fn history_json(&self) -> JsValue {
        let v: Vec<HistoryEntry> = self
            .history
            .iter()
            .map(|(n, h)| HistoryEntry {
                name: n.clone(),
                by_human: *h,
            })
            .collect();
        serde_wasm_bindgen::to_value(&v).unwrap_or(JsValue::NULL)
    }

    /// Attempt to play `input` as a human move.
    /// Returns `{ok: boolean, error?: string, name?: string}`.
    pub fn apply_human_move(&mut self, input: &str) -> JsValue {
        if self.over {
            return self.make_result(false, Some("Game is already over"), None);
        }
        if !self.human_turn {
            return self.make_result(false, Some("Not your turn"), None);
        }

        let key = lookup_key(input);
        if key.is_empty() {
            return self.make_result(false, Some("No letters found in that input"), None);
        }

        let Some(candidates) = self.key_to_names.get(&key) else {
            return self.make_result(false, Some("Not in this Pokémon pool"), None);
        };
        let canonical = candidates[0].clone();

        let Some((f, l)) = first_last_letters(&canonical) else {
            return self.make_result(false, Some("Internal error resolving letters"), None);
        };

        if let Some(req) = self.required {
            if f != req {
                let req_char = (b'A' + req) as char;
                let msg = format!("Must start with '{req_char}'");
                return self.make_result(false, Some(&msg), None);
            }
        }

        let key = lookup_key(&canonical);
        if self.used_name_keys.contains(&key) {
            return self.make_result(false, Some("Already used!"), None);
        }
        if self.counts[pair_index(f, l)] == 0 {
            return self.make_result(false, Some("Already used!"), None);
        }

        self.counts[pair_index(f, l)] -= 1;
        self.graph.on_decrement(f, l, &self.counts);
        self.used_name_keys.insert(key);
        self.required = Some(l);
        self.history.push((canonical.clone(), true));
        self.human_turn = false;

        let state = GameState {
            required: self.required,
            counts: &self.counts,
            graph: &self.graph,
        };
        if state.legal_moves().is_empty() {
            self.over = true;
            self.human_won_flag = true;
        }

        self.make_result(true, None, Some(&canonical))
    }

    /// CPU takes its turn. Returns `{name: string|null, lost: boolean}`.
    /// Call this when `is_human_turn()` is false and `is_over()` is false.
    pub fn cpu_take_turn(&mut self) -> JsValue {
        if self.over || self.human_turn {
            let r = CpuMoveResult {
                name: None,
                lost: false,
            };
            return serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL);
        }

        let state = GameState {
            required: self.required,
            counts: &self.counts,
            graph: &self.graph,
        };

        match self.cpu.choose_move(&state) {
            None => {
                self.over = true;
                self.human_won_flag = true;
                let r = CpuMoveResult {
                    name: None,
                    lost: true,
                };
                serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL)
            }
            Some((f, l)) => {
                let name = self.pick_name_for_pair(f, l);
                self.counts[pair_index(f, l)] -= 1;
                self.graph.on_decrement(f, l, &self.counts);
                self.used_name_keys.insert(lookup_key(&name));
                self.required = Some(l);
                self.history.push((name.clone(), false));
                self.human_turn = true;

                let state = GameState {
                    required: self.required,
                    counts: &self.counts,
                    graph: &self.graph,
                };
                if state.legal_moves().is_empty() {
                    self.over = true;
                    self.human_won_flag = false;
                }

                let r = CpuMoveResult {
                    name: Some(name),
                    lost: false,
                };
                serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL)
            }
        }
    }

    /// Returns the CPU's preferred move name without applying it (for the hint feature).
    /// Returns `null` if no legal moves exist.
    pub fn hint(&mut self) -> JsValue {
        let state = GameState {
            required: self.required,
            counts: &self.counts,
            graph: &self.graph,
        };
        match self.cpu.choose_move(&state) {
            None => JsValue::NULL,
            Some((f, l)) => JsValue::from_str(&self.pick_name_for_pair(f, l)),
        }
    }

    // --- private helpers ---

    /// Random opening from [`non_terminal_opening_indices`], played by the nominal first player.
    fn apply_random_forced_opening(&mut self, opener_is_human: bool) {
        let safe = non_terminal_opening_indices(&self.graph, &self.names);
        if safe.is_empty() {
            return;
        }
        let pick = safe[rand::rng().random_range(0..safe.len())];
        let name = self.names[pick].clone();
        let (f, l) = first_last_letters(&name).expect("valid ascii name");
        let idx = pair_index(f, l);
        self.counts[idx] -= 1;
        self.graph.on_decrement(f, l, &self.counts);
        self.used_name_keys.insert(lookup_key(&name));
        self.required = Some(l);
        self.history.push((name, opener_is_human));
        self.human_turn = !opener_is_human;

        let state = GameState {
            required: self.required,
            counts: &self.counts,
            graph: &self.graph,
        };
        if state.legal_moves().is_empty() {
            self.over = true;
            self.human_won_flag = opener_is_human;
        }
    }

    fn pick_name_for_pair(&self, f: u8, l: u8) -> String {
        for name in &self.names {
            if let Some((nf, nl)) = first_last_letters(name) {
                if nf == f && nl == l && !self.used_name_keys.contains(&lookup_key(name)) {
                    return name.clone();
                }
            }
        }
        format!("({}{})", (b'a' + f) as char, (b'a' + l) as char)
    }

    fn make_result(&self, ok: bool, error: Option<&str>, name: Option<&str>) -> JsValue {
        let r = MoveResult {
            ok,
            error: error.map(|e| e.to_string()),
            name: name.map(|n| n.to_string()),
        };
        serde_wasm_bindgen::to_value(&r).unwrap_or(JsValue::NULL)
    }
}
