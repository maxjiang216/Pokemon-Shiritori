//! Interactive human vs CPU Pokémon Shiritori (name-first CLI).

use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, Write};

use rand::Rng;

use crate::agents::{Agent, GameState};
use crate::gen1::pair_index;
use crate::graph::{non_terminal_opening_indices, LetterGraph};
use crate::normalize::first_last_letters;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Random opening that leaves the opponent at least one legal move (no dead-ending last letters).
fn try_forced_random_opening(
    counts: &mut [u8; 676],
    graph: &mut LetterGraph,
    names: &[String],
    used_name_keys: &mut HashSet<String>,
    opener_is_human: bool,
) -> Option<u8> {
    let safe = non_terminal_opening_indices(graph, names);
    if safe.is_empty() {
        return None;
    }
    let pick = safe[rand::rng().random_range(0..safe.len())];
    let name = &names[pick];
    let (f, l) = first_last_letters(name).expect("valid ascii name");
    let idx = pair_index(f, l);
    counts[idx] -= 1;
    graph.on_decrement(f, l, counts);
    used_name_keys.insert(lookup_key(name));
    if opener_is_human {
        println!("Random opening (you): {name}");
    } else {
        println!("Random opening (CPU): {name}");
    }
    Some(l)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstPlayer {
    Human,
    Cpu,
    Random,
}

/// Run interactive play until the user quits.
pub fn run_play(
    names: Vec<String>,
    base_counts: [u8; 676],
    mut cpu: Box<dyn Agent>,
    first: FirstPlayer,
) {
    let (pair_to_names, key_to_names) = build_indexes(&names);
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    loop {
        let mut counts = base_counts;
        let mut graph = LetterGraph::from_counts(&counts);
        let mut used_name_keys: HashSet<String> = HashSet::new();
        let mut required: Option<u8> = None;
        let human_first = match first {
            FirstPlayer::Human => true,
            FirstPlayer::Cpu => false,
            FirstPlayer::Random => rand::rng().random_bool(0.5),
        };
        let mut human_to_move = human_first;

        print_banner(cpu.name(), names.len(), first, human_first);
        let _ = io::stdout().flush();

        if let Some(l) = try_forced_random_opening(
            &mut counts,
            &mut graph,
            &names,
            &mut used_name_keys,
            human_first,
        ) {
            required = Some(l);
            human_to_move = !human_first;
        }

        let mut game_active = true;
        while game_active {
            if human_to_move {
                let state = GameState {
                    required,
                    counts: &counts,
                    graph: &graph,
                };
                if state.legal_moves().is_empty() {
                    println!("You have no legal move. You lose.");
                    game_active = false;
                    continue;
                }

                loop {
                    print!("Your move: ");
                    let _ = io::stdout().flush();
                    let mut line = String::new();
                    if stdin.read_line(&mut line).is_err() {
                        return;
                    }
                    let line = line.trim();
                    if line.is_empty() {
                        println!("(type a Pokémon name, or: help legal prefer restart quit)");
                        continue;
                    }

                    match parse_command(line) {
                        Some(Cmd::Help) => print_help(),
                        Some(Cmd::Quit) => return,
                        Some(Cmd::Restart) => {
                            game_active = false;
                            break;
                        }
                        Some(Cmd::Legal) => {
                            let mut list = legal_names(required, &counts, &names, &used_name_keys);
                            list.sort();
                            if list.is_empty() {
                                println!("No legal Pokémon names.");
                            } else {
                                println!("Legal names ({}):", list.len());
                                for n in &list {
                                    println!("  {n}");
                                }
                            }
                        }
                        Some(Cmd::Prefer) => match cpu.choose_move(&state) {
                            None => println!("(engine has no legal move from this position)"),
                            Some((f, t)) => {
                                let idx = pair_index(f, t);
                                let labels: Vec<String> = pair_to_names[idx].clone();
                                let fc = (b'a' + f) as char;
                                let tc = (b'a' + t) as char;
                                if labels.is_empty() {
                                    println!(
                                        "Engine prefers edge {fc}→{tc} (no dex label in pool)."
                                    );
                                } else {
                                    println!("Engine prefers: {} ({fc}→{tc})", labels.join(" or "),);
                                }
                            }
                        },
                        None => match resolve_name(line, &key_to_names) {
                            Err(e) => println!("{e}"),
                            Ok((f, l, canonical)) => {
                                if let Some(r) = required {
                                    if f != r {
                                        let rc = (b'a' + r) as char;
                                        println!(
                                            "Must start with '{rc}' (after normalization). Try again."
                                        );
                                        continue;
                                    }
                                }
                                let idx = pair_index(f, l);
                                let key = lookup_key(&canonical);
                                if used_name_keys.contains(&key) {
                                    println!("That Pokémon was already played. Try again.");
                                    continue;
                                }
                                if counts[idx] == 0 {
                                    println!("That Pokémon (edge) is not available. Try again.");
                                    continue;
                                }
                                println!("You: {canonical}");
                                counts[idx] -= 1;
                                graph.on_decrement(f, l, &counts);
                                used_name_keys.insert(key);
                                required = Some(l);
                                human_to_move = false;
                                break;
                            }
                        },
                    }
                }
                if !game_active {
                    break;
                }
            }

            if !human_to_move {
                let state = GameState {
                    required,
                    counts: &counts,
                    graph: &graph,
                };
                match cpu.choose_move(&state) {
                    None => {
                        println!("CPU has no legal move. You win!");
                        game_active = false;
                    }
                    Some((f, t)) => {
                        let idx = pair_index(f, t);
                        let display = pick_unused_name(idx, &pair_to_names, &used_name_keys);
                        let fc = (b'a' + f) as char;
                        let tc = (b'a' + t) as char;
                        match &display {
                            Some(n) => println!("CPU: {n} ({fc}→{tc})"),
                            None => println!("CPU: ({fc}→{tc})"),
                        }
                        counts[idx] -= 1;
                        graph.on_decrement(f, t, &counts);
                        if let Some(n) = &display {
                            used_name_keys.insert(lookup_key(n));
                        }
                        required = Some(t);
                        human_to_move = true;
                    }
                }
            }
        }

        loop {
            print!("[restart or quit] ");
            let _ = io::stdout().flush();
            let mut line = String::new();
            if stdin.read_line(&mut line).is_err() {
                return;
            }
            let line = line.trim();
            match parse_command(line) {
                Some(Cmd::Quit) => return,
                Some(Cmd::Restart) => break,
                None if line.eq_ignore_ascii_case("quit") || line.eq_ignore_ascii_case("exit") => {
                    return
                }
                None if line.eq_ignore_ascii_case("restart")
                    || line.eq_ignore_ascii_case("new") =>
                {
                    break
                }
                _ if line.is_empty() => println!("Type restart or quit."),
                _ => println!("Type restart or quit."),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Indexes & name resolution
// ---------------------------------------------------------------------------

fn lookup_key(s: &str) -> String {
    s.chars()
        .flat_map(|c| c.to_lowercase())
        .filter(|c| c.is_ascii_alphabetic())
        .collect()
}

fn build_indexes(names: &[String]) -> (Vec<Vec<String>>, HashMap<String, Vec<String>>) {
    let mut pair_to_names: Vec<Vec<String>> = vec![Vec::new(); 676];
    let mut key_to_names: HashMap<String, Vec<String>> = HashMap::new();
    for name in names {
        let Some((f, l)) = first_last_letters(name) else {
            continue;
        };
        let idx = pair_index(f, l);
        pair_to_names[idx].push(name.clone());
        let key = lookup_key(name);
        if !key.is_empty() {
            key_to_names.entry(key).or_default().push(name.clone());
        }
    }
    (pair_to_names, key_to_names)
}

fn resolve_name(
    line: &str,
    key_to_names: &HashMap<String, Vec<String>>,
) -> Result<(u8, u8, String), String> {
    let key = lookup_key(line);
    if key.is_empty() {
        return Err("No letters in that input. Try again.".to_string());
    }
    let Some(candidates) = key_to_names.get(&key) else {
        return Err("Unknown Pokémon name (not in the current pool). Try again.".to_string());
    };
    let canonical = &candidates[0];
    if candidates.len() > 1 {
        for c in candidates.iter().skip(1) {
            if first_last_letters(c) != first_last_letters(canonical) {
                return Err(format!(
                    "Ambiguous name {:?}; be more specific (same key maps to different edges).",
                    key
                ));
            }
        }
    }
    let Some((f, l)) = first_last_letters(canonical) else {
        return Err("Internal error resolving letters.".to_string());
    };
    Ok((f, l, canonical.clone()))
}

fn legal_names(
    required: Option<u8>,
    counts: &[u8; 676],
    names: &[String],
    used_name_keys: &HashSet<String>,
) -> Vec<String> {
    let mut out = Vec::new();
    for name in names {
        if used_name_keys.contains(&lookup_key(name)) {
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

fn pick_unused_name(
    idx: usize,
    pair_to_names: &[Vec<String>],
    used_name_keys: &HashSet<String>,
) -> Option<String> {
    pair_to_names.get(idx).and_then(|v| {
        v.iter()
            .find(|n| !used_name_keys.contains(&lookup_key(n)))
            .cloned()
    })
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cmd {
    Help,
    Quit,
    Restart,
    Legal,
    Prefer,
}

fn parse_command(line: &str) -> Option<Cmd> {
    let t = line.trim();
    if t.is_empty() {
        return None;
    }
    let first = t.split_whitespace().next()?;
    let low = first.to_ascii_lowercase();
    match low.as_str() {
        "help" | "?" => Some(Cmd::Help),
        "quit" | "exit" => Some(Cmd::Quit),
        "restart" | "new" => Some(Cmd::Restart),
        "legal" | "hint" | "hints" => Some(Cmd::Legal),
        "prefer" | "engine" => Some(Cmd::Prefer),
        _ => None,
    }
}

fn print_help() {
    println!(
        "Commands:  help   — this message\n\
                  \t  legal  — list legal Pokémon names now\n\
                  \t  prefer — CPU’s preferred move from this position (names + letters)\n\
                  \t  restart — new game\n\
                  \t  quit   — exit\n\
         Or type a Pokémon name (case-insensitive; spaces/punctuation ignored for matching)."
    );
}

fn print_banner(cpu: &str, n: usize, first: FirstPlayer, human_first: bool) {
    let first_s = match first {
        FirstPlayer::Human => "human",
        FirstPlayer::Cpu => "cpu",
        FirstPlayer::Random => "random (this game)",
    };
    let who = if human_first {
        "You move first."
    } else {
        "CPU moves first."
    };
    println!("── Pokémon Shiritori — CPU: {cpu} — {n} Pokémon — --first {first_s}\n{who}\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_key_strips_punctuation() {
        assert_eq!(lookup_key("Mr. Mime"), "mrmime");
        assert_eq!(lookup_key("PIKACHU"), "pikachu");
    }

    #[test]
    fn parse_command_basic() {
        assert_eq!(parse_command("help"), Some(Cmd::Help));
        assert_eq!(parse_command("LEGAL"), Some(Cmd::Legal));
        assert_eq!(parse_command("prefer"), Some(Cmd::Prefer));
        assert_eq!(parse_command("pikachu"), None);
    }
}
