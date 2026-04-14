//! CLI for the Pokémon Shiritori solver and tournament.
//!
//! Subcommands:
//!   solve        [--count N]                      Exact solver (warns if N > 15)
//!   tournament   [--count N] [--depth D]          Agent tournament
//!                [--rollouts R] [--games G]
//!   sensitivity  [--count N] [--samples S]        Sensitivity curve
//!                [--rollouts R]
//!   play         [--cpu NAME] [--first human|cpu|random]  Interactive vs CPU
//!                [--count N] [--depth D] [--rollouts R]

use std::collections::HashMap;

use pokemon_shiritori::agents::{
    Agent, DeadEndHunter, ExactAgent, GreedyAgent, HybridAgent, RandomAgent, RolloutAgent,
};
use pokemon_shiritori::analysis::run_sensitivity_report;
use pokemon_shiritori::gen1::{counts_from_names, load_gen1_names};
use pokemon_shiritori::graph::LetterGraph;
use pokemon_shiritori::play::{run_play, FirstPlayer};
use pokemon_shiritori::solver::can_win;
use pokemon_shiritori::tournament::round_robin;
use rustc_hash::FxHashMap;

// ---------------------------------------------------------------------------
// Argument parsing (no external dep — simple key=value args)
// ---------------------------------------------------------------------------

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
    let all_names = load_gen1_names();
    let count: usize = opts
        .get("count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(all_names.len());

    if count > 15 {
        eprintln!(
            "Warning: --count {count} may produce a very large search space. \
             Exact solving of full Gen 1 is not guaranteed to terminate quickly. \
             Consider --count 15 or less for guaranteed termination."
        );
    }

    let names: Vec<String> = all_names.into_iter().take(count).collect();
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
    let all_names = load_gen1_names();
    let count: usize = opts
        .get("count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(all_names.len());
    let depth: usize = opts.get("depth").and_then(|s| s.parse().ok()).unwrap_or(4);
    let rollouts: usize = opts
        .get("rollouts")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let games: u32 = opts.get("games").and_then(|s| s.parse().ok()).unwrap_or(1);

    let names: Vec<String> = all_names.into_iter().take(count).collect();
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
        Box::new(DeadEndHunter),
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
    let all_names = load_gen1_names();
    let count: usize = opts
        .get("count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(all_names.len());
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

    let names: Vec<String> = all_names.into_iter().take(count).collect();
    let base_counts = counts_from_names(&names);

    println!("Using {} Pokémon.", names.len());
    run_sensitivity_report(&base_counts, &depths, samples, rollouts);
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
        "deadend" | "deadendhunter" | "hunter" => Ok(Box::new(DeadEndHunter)),
        "rollout" => Ok(Box::new(RolloutAgent::new(depth, rollouts))),
        "hybrid" => Ok(Box::new(HybridAgent::new(depth, rollouts))),
        "exact" => {
            if count <= 15 {
                Ok(Box::new(ExactAgent::new()))
            } else {
                Err(format!(
                    "Exact agent is only available with --count ≤ 15 (got {count}). \
                     Use --cpu hybrid, rollout, greedy, or another agent."
                ))
            }
        }
        _ => Err(format!(
            "Unknown --cpu {cpu:?}. Use: random, greedy, deadend, rollout, hybrid, exact"
        )),
    }
}

fn cmd_play(opts: &HashMap<String, String>) {
    let all_names = load_gen1_names();
    let count: usize = opts
        .get("count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(all_names.len());
    let depth: usize = opts.get("depth").and_then(|s| s.parse().ok()).unwrap_or(4);
    let rollouts: usize = opts
        .get("rollouts")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let cpu_str = opts.get("cpu").map(|s| s.as_str()).unwrap_or("hybrid");
    let first_str = opts.get("first").map(|s| s.as_str()).unwrap_or("human");

    let names: Vec<String> = all_names.into_iter().take(count).collect();
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
        _ => {
            println!("Pokémon Shiritori Solver\n");
            println!("Usage: pokemon-shiritori <subcommand> [options]\n");
            println!("Subcommands:");
            println!("  solve        [--count N]");
            println!("               Exact solver. Warns if N > 15.");
            println!();
            println!("  tournament   [--count N] [--depth D] [--rollouts R] [--games G]");
            println!("               Run all-agent tournament.");
            println!("               Defaults: count=151, depth=4, rollouts=30, games=1");
            println!();
            println!(
                "  sensitivity  [--count N] [--depths D1,D2,...] [--samples S] [--rollouts R]"
            );
            println!("               Measure outcome sensitivity at various game depths.");
            println!("               Defaults: depths=10,30,60,100,140, samples=100, rollouts=40");
            println!();
            println!("  play         [--cpu NAME] [--first human|cpu|random]");
            println!("               [--count N] [--depth D] [--rollouts R]");
            println!("               Interactive human vs CPU. Commands: help legal prefer restart quit.");
            println!(
                "               Defaults: cpu=hybrid, first=human, count=151, depth=4, rollouts=30"
            );
        }
    }
}
