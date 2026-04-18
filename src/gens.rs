//! National Pokédex names by generation (English), generations 1–6.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Embedded [`data/gens_1_6_en.json`](../../data/gens_1_6_en.json): keys `"1"` … `"6"`.
static GENS_JSON: &str = include_str!("../data/gens_1_6_en.json");

static GENS_CACHE: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

fn gens_table() -> &'static HashMap<String, Vec<String>> {
    GENS_CACHE.get_or_init(|| serde_json::from_str(GENS_JSON).expect("gens_1_6_en.json valid"))
}

/// Highest supported generation (Kalos / ORAS-era national dex through #721).
pub const MAX_GENERATION: u8 = 6;

/// Parse a comma-separated list like `1,3,6` or `all`. Empty string = all generations.
pub fn parse_generation_list(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("all") {
        return Ok((1..=MAX_GENERATION).collect());
    }
    let mut out = Vec::new();
    for part in s.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let g: u8 = p
            .parse()
            .map_err(|_| format!("invalid generation token: {p:?}"))?;
        if !(1..=MAX_GENERATION).contains(&g) {
            return Err(format!(
                "generation must be 1..={MAX_GENERATION}, got {g}"
            ));
        }
        if !out.contains(&g) {
            out.push(g);
        }
    }
    out.sort_unstable();
    if out.is_empty() {
        return Err("no generations selected".into());
    }
    Ok(out)
}

/// Concatenate national dex blocks in generation order (e.g. `[3, 1]` → Gen 1 then Gen 3).
pub fn names_for_generations(generations: &[u8]) -> Result<Vec<String>, String> {
    let table = gens_table();
    let mut names = Vec::new();
    for &g in generations {
        let key = g.to_string();
        let list = table
            .get(&key)
            .ok_or_else(|| format!("missing generation {g} in data"))?;
        names.extend(list.iter().cloned());
    }
    Ok(names)
}

/// Default pool: generations 1–6 (721 species).
pub fn default_generation_list() -> Vec<u8> {
    (1..=MAX_GENERATION).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_pool_is_721() {
        let n = names_for_generations(&default_generation_list()).unwrap();
        assert_eq!(n.len(), 721);
    }

    #[test]
    fn gen1_only_is_151() {
        let n = names_for_generations(&[1]).unwrap();
        assert_eq!(n.len(), 151);
    }

    #[test]
    fn parse_list() {
        assert_eq!(parse_generation_list("all").unwrap(), (1..=6).collect::<Vec<_>>());
        assert_eq!(parse_generation_list("2,1").unwrap(), vec![1, 2]);
        assert!(parse_generation_list("0").is_err());
        assert!(parse_generation_list("7").is_err());
    }
}
