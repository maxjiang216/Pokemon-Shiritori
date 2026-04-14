//! Sensitivity analysis: how often does perturbing one used word flip the W/L outcome?
//!
//! Algorithm per sample at depth D:
//! 1. Play D random moves from the opening → mid-game state.
//! 2. Estimate the W/L label of that state via `rollouts` random completions (majority vote).
//! 3. For each non-zero bucket: perturb by -1 (simulate "that word was played differently"),
//!    re-estimate, and check if the majority flips.
//! 4. `sensitivity(D)` = fraction of perturbations that flip the label.

use crate::gen1::pair_index;

/// Estimate whether the player-to-move wins from `(required, counts)` using random rollouts.
/// Returns `true` if majority of rollouts result in the current player winning.
fn rollout_estimate(required: Option<u8>, counts: &[u8; 676], rollouts: usize) -> bool {
    if rollouts == 0 { return true; } // arbitrary default
    let wins: usize = (0..rollouts)
        .filter(|_| random_game_win(required, counts))
        .count();
    wins * 2 >= rollouts // majority vote
}

/// Random game from `(required, counts)`. Returns `true` if current player wins.
fn random_game_win(required: Option<u8>, counts: &[u8; 676]) -> bool {
    use rand::Rng;
    let mut counts = *counts;
    let mut req = required;
    let mut my_turn = true;
    loop {
        let moves: Vec<(u8, u8)> = match req {
            None => (0u8..26).flat_map(|f| (0u8..26).map(move |t| (f, t)))
                .filter(|&(f, t)| counts[pair_index(f, t)] > 0)
                .collect(),
            Some(l) => (0u8..26).map(|t| (l, t))
                .filter(|&(_, t)| counts[pair_index(l, t)] > 0)
                .collect(),
        };
        if moves.is_empty() { return !my_turn; }
        let (f, t) = moves[rand::rng().random_range(0..moves.len())];
        counts[pair_index(f, t)] -= 1;
        req = Some(t);
        my_turn = !my_turn;
    }
}

/// Measure sensitivity at a specific game depth.
///
/// Returns `(sensitivity, n_valid_samples)` where sensitivity ∈ [0,1].
pub fn sensitivity_at_depth(
    base_counts: &[u8; 676],
    sample_depth: usize,
    n_samples: usize,
    rollouts: usize,
) -> (f64, usize) {
    use rand::Rng;

    let mut total_perturbations = 0usize;
    let mut flipped = 0usize;
    let mut valid_samples = 0usize;

    for _ in 0..n_samples {
        // Step 1: Play `sample_depth` random moves from opening.
        let mut counts = *base_counts;
        let mut req: Option<u8> = None;
        let mut moves_played = 0;

        for _ in 0..sample_depth {
            let moves: Vec<(u8, u8)> = match req {
                None => (0u8..26).flat_map(|f| (0u8..26).map(move |t| (f, t)))
                    .filter(|&(f, t)| counts[pair_index(f, t)] > 0)
                    .collect(),
                Some(l) => (0u8..26).map(|t| (l, t))
                    .filter(|&(_, t)| counts[pair_index(l, t)] > 0)
                    .collect(),
            };
            if moves.is_empty() { break; }
            let (f, t) = moves[rand::rng().random_range(0..moves.len())];
            counts[pair_index(f, t)] -= 1;
            req = Some(t);
            moves_played += 1;
        }

        if moves_played == 0 { continue; } // game ended before we started
        valid_samples += 1;

        // Step 2: Estimate baseline W/L.
        let baseline = rollout_estimate(req, &counts, rollouts);

        // Step 3: Perturb each non-zero bucket by -1 and re-estimate.
        for u in 0u8..26 {
            for v in 0u8..26 {
                let idx = pair_index(u, v);
                if counts[idx] == 0 { continue; }

                // Perturb: pretend this edge was used one more time.
                let mut perturbed = counts;
                perturbed[idx] -= 1;

                let perturbed_label = rollout_estimate(req, &perturbed, rollouts);
                total_perturbations += 1;
                if perturbed_label != baseline {
                    flipped += 1;
                }
            }
        }
    }

    let sensitivity = if total_perturbations == 0 {
        0.0
    } else {
        flipped as f64 / total_perturbations as f64
    };
    (sensitivity, valid_samples)
}

/// Run sensitivity measurement at multiple depths and print a report.
pub fn run_sensitivity_report(
    base_counts: &[u8; 676],
    depths: &[usize],
    n_samples: usize,
    rollouts: usize,
) {
    println!("Sensitivity analysis ({n_samples} samples, {rollouts} rollouts per estimate)");
    println!("{:<10} {:>14} {:>14}", "Depth", "Sensitivity", "Valid samples");
    println!("{}", "-".repeat(42));
    for &d in depths {
        let (s, v) = sensitivity_at_depth(base_counts, d, n_samples, rollouts);
        println!("{:<10} {:>13.2}% {:>14}", d, s * 100.0, v);
    }
}
