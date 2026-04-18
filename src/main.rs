//! CLI for the Pokémon Shiritori solver and tournament.
//!
//! Subcommands:
//!   solve        [--count N]                      Exact solver (warns if N > 15)
//!   tournament   [--count N] [--depth D]          Agent tournament
//!                [--rollouts R] [--games G]
//!   sensitivity  [--count N] [--samples S]        Sensitivity curve
//!                [--rollouts R]
//!   play         [--cpu NAME] [--first human|cpu|random]  Interactive vs CPU
//!                [--gens LIST] [--count N] [--depth D] [--rollouts R]

use std::collections::HashMap;

use pokemon_shiritori::agents::{
    Agent, ExactAgent, GreedyAgent, HybridAgent, RandomAgent, RolloutAgent,
};
use pokemon_shiritori::analysis::run_sensitivity_report;
use pokemon_shiritori::gen1::counts_from_names;
use pokemon_shiritori::gens::{names_for_generations, parse_generation_list};
use pokemon_shiritori::graph::LetterGraph;
use pokemon_shiritori::normalize::first_last_letters;
use pokemon_shiritori::play::{run_play, FirstPlayer};
use pokemon_shiritori::solver::{can_win, first_player_wins_after_opening_edge};
use pokemon_shiritori::terminal_stats::{
    count_openings_with_terminal_reply, count_terminal_chains, TerminalPool,
};
use pokemon_shiritori::tournament::round_robin;
use rustc_hash::FxHashMap;

// ---------------------------------------------------------------------------
// Argument parsing (no external dep — simple key=value args)
// ---------------------------------------------------------------------------

/// Build the name pool from `--gens` (default all 1–6) and optional `--count` prefix cap.
fn pool_names_from_opts(opts: &HashMap<String, String>) -> Vec<String> {
    let gens_str = opts.get("gens").map(|s| s.as_str()).unwrap_or("all");
    let gens = match parse_generation_list(gens_str) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Invalid --gens: {e}");
            std::process::exit(1);
        }
    };
    let mut names = names_for_generations(&gens).expect("embedded dex data");
    if let Some(c) = opts.get("count").and_then(|s| s.parse::<usize>().ok()) {
        names.truncate(c.min(names.len()));
    }
    names
}

fn parse_args(args: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        if args[i].starts_with("--") {
            let key = args[i][2..].to_string();
            if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                map.insert(key, args[i + 1].clone());
                i += 2;
            } else {
                map.insert(key, "true".to_string());
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Subcommand: solve
// ---------------------------------------------------------------------------

fn cmd_solve(opts: &HashMap<String, String>) {
    let names = pool_names_from_opts(opts);
    let count = names.len();

    if count > 15 {
        eprintln!(
            "Warning: --count {count} may produce a very large search space. \
             Exact solving of full Gen 1 is not guaranteed to terminate quickly. \
             Consider --count 15 or less for guaranteed termination."
        );
    }

    println!("Solving with {} Pokémon…", names.len());
    for n in &names {
        println!("  {n}");
    }

    let mut counts = counts_from_names(&names);
    let mut graph = LetterGraph::from_counts(&counts);
    let mut memo: FxHashMap<(u8, u64), bool> = FxHashMap::default();

    let result = can_win(None, &mut counts, &mut graph, &mut memo, usize::MAX);
    match result {
        Some(true) => println!("\nResult: First player WINS under optimal play."),
        Some(false) => println!("\nResult: First player LOSES under optimal play."),
        None => println!("\nResult: Unknown (depth limit reached)."),
    }
    println!("Transposition table entries: {}", memo.len());
}

// ---------------------------------------------------------------------------
// Subcommand: tournament
// ---------------------------------------------------------------------------

fn cmd_tournament(opts: &HashMap<String, String>) {
    let names = pool_names_from_opts(opts);
    let count = names.len();
    let depth: usize = opts.get("depth").and_then(|s| s.parse().ok()).unwrap_or(4);
    let rollouts: usize = opts
        .get("rollouts")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let games: u32 = opts.get("games").and_then(|s| s.parse().ok()).unwrap_or(1);

    let base_counts = counts_from_names(&names);

    println!(
        "Tournament: {} Pokémon, {} starting words × 2 sides × {} game(s) per start",
        names.len(),
        names.len(),
        games
    );
    println!("Rollout depth: {depth}, rollouts per leaf: {rollouts}\n");

    let mut agents: Vec<Box<dyn Agent>> = vec![
        Box::new(RandomAgent),
        Box::new(GreedyAgent),
        Box::new(RolloutAgent::new(depth, rollouts)),
        Box::new(HybridAgent::new(depth, rollouts)),
    ];
    if count <= 15 {
        agents.push(Box::new(ExactAgent::new()));
        println!("(ExactAgent included since --count ≤ 15)");
    }

    let result = round_robin(&names, &mut agents, &base_counts, games);
    println!("\nWin% table (row beats column):\n");
    result.print_table();
}

// ---------------------------------------------------------------------------
// Subcommand: sensitivity
// ---------------------------------------------------------------------------

fn cmd_sensitivity(opts: &HashMap<String, String>) {
    let names = pool_names_from_opts(opts);
    let samples: usize = opts
        .get("samples")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let rollouts: usize = opts
        .get("rollouts")
        .and_then(|s| s.parse().ok())
        .unwrap_or(40);

    let depths_str = opts
        .get("depths")
        .map(|s| s.as_str())
        .unwrap_or("10,30,60,100,140");
    let depths: Vec<usize> = depths_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let base_counts = counts_from_names(&names);

    println!("Using {} Pokémon.", names.len());
    run_sensitivity_report(&base_counts, &depths, samples, rollouts);
}

// ---------------------------------------------------------------------------
// Subcommand: stats — optimal opening classification + optional chain counts
// ---------------------------------------------------------------------------

fn cmd_stats(opts: &HashMap<String, String>) {
    let names = pool_names_from_opts(opts);
    let max_k: u32 = opts.get("max-k").and_then(|s| s.parse().ok()).unwrap_or(9);
    let terminal_chains = opts.get("terminal-chains").map(|s| s != "false").unwrap_or(false);
    let depth_limit: usize = opts
        .get("depth-limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(usize::MAX);

    let mut counts = counts_from_names(&names);
    let mut graph = LetterGraph::from_counts(&counts);
    let mut memo: rustc_hash::FxHashMap<(u8, u64), bool> = rustc_hash::FxHashMap::default();
    let pool = TerminalPool::from_names(&names);

    println!(
        "Shiritori stats — {} Pokémon (national dex prefix), English letters as in normalize\n",
        pool.n
    );
    println!(
        "Optimal play on the (first→last) letter multigraph — names that share the same edge have the same outcome.\n"
    );

    let t0 = std::time::Instant::now();
    let mut win = Vec::new();
    let mut lose = Vec::new();
    let mut unknown = Vec::new();
    for name in names.iter() {
        let (f, l) = first_last_letters(name).expect("ascii");
        match first_player_wins_after_opening_edge(
            &mut counts,
            &mut graph,
            &mut memo,
            f,
            l,
            depth_limit,
        ) {
            Some(true) => win.push(name.as_str()),
            Some(false) => lose.push(name.as_str()),
            None => unknown.push(name.as_str()),
        }
    }
    let elapsed = t0.elapsed();

    println!(
        "First player wins with opening: {} / {}  (loses: {}, unknown: {})  [{:.3?}, memo entries: {}]",
        win.len(),
        pool.n,
        lose.len(),
        unknown.len(),
        elapsed,
        memo.len()
    );
    println!("\nWinning openings (N-positions for player to move first):");
    println!("  {}", win.join(", "));
    println!("\nLosing openings (P-positions — opponent wins under optimal play):");
    println!("  {}", lose.join(", "));
    if !unknown.is_empty() {
        println!("\nUnknown (depth limit / incomplete):");
        println!("  {}", unknown.join(", "));
    }

    let (m2_openings, m2_pairs) = count_openings_with_terminal_reply(&pool);
    println!("\n(Reference) One-move terminal reply exists for opponent:");
    println!("  openings with ≥1 such reply: {m2_openings} / {}", pool.n);
    println!("  ordered (opening → reply) pairs: {m2_pairs}");

    if terminal_chains {
        println!("\nmk — ordered chains of length k ending in a one-move terminal (expensive):");
        for k in 1..=max_k {
            let t1 = std::time::Instant::now();
            let c = count_terminal_chains(&pool, k);
            println!("  m{k}: {c}  ({:.3?})", t1.elapsed());
        }
    }
}

// ---------------------------------------------------------------------------
// Subcommand: play (interactive human vs CPU)
// ---------------------------------------------------------------------------

fn make_cpu_agent(
    cpu: &str,
    count: usize,
    depth: usize,
    rollouts: usize,
) -> Result<Box<dyn Agent>, String> {
    match cpu.to_lowercase().as_str() {
        "random" => Ok(Box::new(RandomAgent)),
        "greedy" => Ok(Box::new(GreedyAgent)),
        "deadend" | "deadendhunter" | "hunter" | "hybrid" => {
            Ok(Box::new(HybridAgent::new(depth, rollouts)))
        }
        "rollout" => Ok(Box::new(RolloutAgent::new(depth, rollouts))),
        "exact" => {
            if count <= 15 {
                Ok(Box::new(ExactAgent::new()))
            } else {
                Err(format!(
                    "Exact agent is only available with --count ≤ 15 (got {count}). \
                     Use --cpu deadend, rollout, greedy, or another agent."
                ))
            }
        }
        _ => Err(format!(
            "Unknown --cpu {cpu:?}. Use: random, greedy, deadend, rollout, hybrid, exact"
        )),
    }
}

fn cmd_play(opts: &HashMap<String, String>) {
    let names = pool_names_from_opts(opts);
    let count = names.len();
    let depth: usize = opts.get("depth").and_then(|s| s.parse().ok()).unwrap_or(4);
    let rollouts: usize = opts
        .get("rollouts")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let cpu_str = opts.get("cpu").map(|s| s.as_str()).unwrap_or("deadend");
    let first_str = opts.get("first").map(|s| s.as_str()).unwrap_or("human");

    let base_counts = counts_from_names(&names);

    let cpu = match make_cpu_agent(cpu_str, count, depth, rollouts) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let first = match first_str.to_lowercase().as_str() {
        "human" | "you" | "player" => FirstPlayer::Human,
        "cpu" | "computer" | "bot" => FirstPlayer::Cpu,
        "random" => FirstPlayer::Random,
        _ => {
            eprintln!("Unknown --first {first_str:?}. Use: human, cpu, random");
            std::process::exit(1);
        }
    };

    run_play(names, base_counts, cpu, first);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let raw: Vec<String> = std::env::args().collect();
    let subcmd = raw.get(1).map(|s| s.as_str()).unwrap_or("help");
    let opts = parse_args(&raw[2.min(raw.len())..]);

    match subcmd {
        "solve" => cmd_solve(&opts),
        "tournament" => cmd_tournament(&opts),
        "sensitivity" => cmd_sensitivity(&opts),
        "play" => cmd_play(&opts),
        "stats" => cmd_stats(&opts),
        "cpus" | "agents" => cmd_cpus(),
        _ => {
            println!("Pokémon Shiritori Solver\n");
            println!("Usage: pokemon-shiritori <subcommand> [options]\n");
            println!("Subcommands:");
            println!("  solve        [--gens LIST] [--count N]");
            println!("               Exact solver. Warns if pool size > 15.");
            println!("               --gens: comma-separated 1–6 or `all` (default all). --count caps pool size.");
            println!();
            println!("  tournament   [--gens LIST] [--count N] [--depth D] [--rollouts R] [--games G]");
            println!("               Run all-agent tournament.");
            println!("               Defaults: gens=all, depth=4, rollouts=30, games=1");
            println!();
            println!(
                "  sensitivity  [--gens LIST] [--count N] [--depths D1,D2,...] [--samples S] [--rollouts R]"
            );
            println!("               Measure outcome sensitivity at various game depths.");
            println!("               Defaults: depths=10,30,60,100,140, samples=100, rollouts=40");
            println!();
            println!("  play         [--cpu NAME] [--first human|cpu|random]");
            println!("               [--gens LIST] [--count N] [--depth D] [--rollouts R]");
            println!("               Interactive human vs CPU.");
            println!("               In-game commands: help  legal  prefer  restart  quit");
            println!("               Defaults: cpu=deadend, first=human, gens=all, depth=4, rollouts=30");
            println!();
            println!("  stats        [--gens LIST] [--count N] [--terminal-chains] [--max-k K] [--depth-limit N]");
            println!("               Optimal win/lose per opening (exact on letter multigraph);");
            println!("               optional mk terminal-chain counts. depth-limit=∞ by default.");
            println!();
            println!("  cpus         List all CPU agents with descriptions.");
        }
    }
}

fn cmd_cpus() {
    println!("Available CPU agents (use with --cpu NAME):\n");
    println!("  random         Uniform random over legal moves.");
    println!("  greedy         Minimize opponent's out-degree at the landing letter.");
    println!("  deadend        Retrograde + SCC check first; rollout fallback.");
    println!("               aliases: hybrid, deadendhunter, hunter");
    println!("  rollout        Exact minimax to depth D, then R random rollouts per leaf.");
    println!("  exact          Full memoized minimax — only usable with --count ≤ 15.");
    println!();
    println!("Tournament lineup: Random, Greedy, Rollout, DeadEnd (hybrid engine).");
    println!("  DeadEnd uses O(26²) retrograde analysis plus rollout search when needed.");
}
