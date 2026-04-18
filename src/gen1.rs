//! `(first, last)` bucket counts and Gen 1–only helpers for tests.

use crate::gens::names_for_generations;
use crate::normalize::first_last_letters;

/// Parse Gen 1 names from the shared generations data (national dex #1–151).
pub fn load_gen1_names() -> Vec<String> {
    names_for_generations(&[1]).expect("generation 1 data present")
}

#[inline]
pub fn pair_index(first: u8, last: u8) -> usize {
    debug_assert!(first < 26 && last < 26);
    usize::from(first) * 26 + usize::from(last)
}

/// Build multiset counts for each `(first_letter, last_letter)` pair.
pub fn counts_from_names(names: &[String]) -> [u8; 676] {
    let mut counts = [0u8; 676];
    for name in names {
        let Some((f, l)) = first_last_letters(name) else {
            panic!("name has no ASCII letters: {name:?}");
        };
        let idx = pair_index(f, l);
        counts[idx] = counts[idx].saturating_add(1);
    }
    counts
}

/// Opening position for Gen 1.
pub fn gen1_opening_counts() -> [u8; 676] {
    counts_from_names(&load_gen1_names())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen1_has_151_names() {
        let names = load_gen1_names();
        assert_eq!(names.len(), 151);
    }

    #[test]
    fn total_pokemon_matches_sum_of_counts() {
        let c = gen1_opening_counts();
        let sum: u32 = c.iter().map(|&x| u32::from(x)).sum();
        assert_eq!(sum, 151);
    }
}
