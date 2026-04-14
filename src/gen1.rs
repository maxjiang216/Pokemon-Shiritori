//! Generation I English names and `(first, last)` bucket counts.

use crate::normalize::first_last_letters;

/// Embedded [`data/gen1_en.json`](../../data/gen1_en.json) (national dex order).
pub static GEN1_JSON: &str = include_str!("../data/gen1_en.json");

/// Parse embedded JSON into owned names.
pub fn load_gen1_names() -> Vec<String> {
    serde_json::from_str(GEN1_JSON).expect("gen1_en.json must be a JSON array of strings")
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
