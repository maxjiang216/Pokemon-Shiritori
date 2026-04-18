//! Combinatorics for Gen 1 pools:
//!
//! - **Forced line** — [`forced_terminal_game_length`]: fix the opening; the game is *uniquely
//!   determined* only if every turn has 0 or 1 legal move until someone cannot move (never ≥2).
//! - **Terminal chains** — [`count_terminal_chains`]: count ordered length-`k` paths whose last
//!   move is one-move terminal (optional; expensive for large `k`).
//!
//! Hot paths use fixed arrays, 151-bit occupancy, and O(1) legal counts via a remaining
//! first-letter histogram (backtracked on recursion).

use crate::gen1::load_gen1_names;
use crate::normalize::first_last_letters;

/// 151 Pokémon → 192 bits (three limbs; high bits unused).
#[derive(Clone, Copy)]
struct Bits151([u64; 3]);

impl Bits151 {
    const fn new() -> Self {
        Self([0, 0, 0])
    }

    #[inline(always)]
    fn word_idx(i: u8) -> usize {
        i as usize / 64
    }

    #[inline(always)]
    fn bit(i: u8) -> u64 {
        1u64 << (i as u32 % 64)
    }

    #[inline(always)]
    fn test(self, i: u8) -> bool {
        (self.0[Self::word_idx(i)] & Self::bit(i)) != 0
    }

    #[inline(always)]
    fn set(&mut self, i: u8) {
        self.0[Self::word_idx(i)] |= Self::bit(i);
    }

    #[inline(always)]
    fn clear(&mut self, i: u8) {
        self.0[Self::word_idx(i)] &= !Self::bit(i);
    }
}

/// Precomputed pool for one `count` (national dex prefix).
pub struct TerminalPool {
    pub n: usize,
    /// `first[i]`, `last[i]` in 0..26
    pub first: Vec<u8>,
    pub last: Vec<u8>,
    /// Indices `i` with `first[i] == letter`
    pub by_start: [Vec<u8>; 26],
    /// Total Pokémon whose name starts with each letter (full pool).
    pub start_hist: [u8; 26],
}

impl TerminalPool {
    /// National dex prefix (`count` first English names).
    pub fn from_count(count: usize) -> Self {
        let names = load_gen1_names();
        let n = count.min(names.len());
        Self::from_names(&names[..n])
    }

    /// Ordered name list (e.g. a `--count` prefix of Gen 1).
    pub fn from_names(names: &[String]) -> Self {
        let n = names.len();
        let mut first = Vec::with_capacity(n);
        let mut last = Vec::with_capacity(n);
        let mut by_start: [Vec<u8>; 26] = std::array::from_fn(|_| Vec::new());
        let mut start_hist = [0u8; 26];

        for (idx, name) in names.iter().enumerate() {
            let (f, l) = first_last_letters(name).expect("ascii name");
            let i = idx as u8;
            first.push(f);
            last.push(l);
            by_start[usize::from(f)].push(i);
            start_hist[usize::from(f)] = start_hist[usize::from(f)].saturating_add(1);
        }

        Self {
            n,
            first,
            last,
            by_start,
            start_hist,
        }
    }
}

/// Ordered chains of length `k` whose last Pokémon is terminal for the next player.
#[inline]
pub fn count_terminal_chains(pool: &TerminalPool, k: u32) -> u64 {
    if k == 0 {
        return 0;
    }
    if k == 1 {
        let mut total = 0u64;
        for i in 0..pool.n {
            let i = i as u8;
            if terminal_after_play_opening(pool, i) {
                total += 1;
            }
        }
        return total;
    }

    let mut rem = pool.start_hist;
    let mut used = Bits151::new();
    let mut total = 0u64;
    for open in 0..pool.n {
        let open = open as u8;
        let f = usize::from(pool.first[usize::from(open)]);
        rem[f] -= 1;
        used.set(open);
        total += extensions(
            pool,
            &mut used,
            &mut rem,
            pool.last[usize::from(open)],
            k - 1,
        );
        used.clear(open);
        rem[f] += 1;
    }
    total
}

#[inline(always)]
fn terminal_after_play_opening(pool: &TerminalPool, i: u8) -> bool {
    let f = usize::from(pool.first[usize::from(i)]);
    let l = usize::from(pool.last[usize::from(i)]);
    let mut rem = pool.start_hist;
    rem[f] -= 1;
    rem[l] == 0
}

/// `need_more` additional Pokémon including the terminal one; next must start with `req`.
#[inline(always)]
fn extensions(
    pool: &TerminalPool,
    used: &mut Bits151,
    rem: &mut [u8; 26],
    req: u8,
    need_more: u32,
) -> u64 {
    let req_usize = usize::from(req);
    if rem[req_usize] == 0 {
        return 0;
    }

    if need_more == 1 {
        let mut c = 0u64;
        for &ti in &pool.by_start[req_usize] {
            if used.test(ti) {
                continue;
            }
            let f = usize::from(pool.first[usize::from(ti)]);
            let l = usize::from(pool.last[usize::from(ti)]);
            rem[f] -= 1;
            if rem[l] == 0 {
                c += 1;
            }
            rem[f] += 1;
        }
        return c;
    }

    let mut total = 0u64;
    for &ti in &pool.by_start[req_usize] {
        if used.test(ti) {
            continue;
        }
        let f = usize::from(pool.first[usize::from(ti)]);
        rem[f] -= 1;
        used.set(ti);
        let last = pool.last[usize::from(ti)];
        total += extensions(pool, used, rem, last, need_more - 1);
        used.clear(ti);
        rem[f] += 1;
    }
    total
}

/// Indices of Pokémon that end the game in one play from the full pool (opponent has no reply).
pub fn immediate_terminal_indices(pool: &TerminalPool) -> Vec<usize> {
    let mut out = Vec::new();
    for i in 0..pool.n {
        if terminal_after_play_opening(pool, i as u8) {
            out.push(i);
        }
    }
    out
}

/// If this opening makes every turn **forced** (the player to move always has 0 or 1 legal move,
/// never 2+), simulate until someone cannot move and return the total number of Pokémon played.
///
/// - `Some(len)` — unique play line ends after `len` moves (last mover wins; next player has 0 legal).
/// - `None` — before the game ended, some player had **≥2** legal moves (not uniquely determined).
pub fn forced_terminal_game_length(pool: &TerminalPool, opening: usize) -> Option<usize> {
    debug_assert!(opening < pool.n);
    let o = opening as u8;
    let mut rem = pool.start_hist;
    let mut used = Bits151::new();
    rem[usize::from(pool.first[usize::from(o)])] -= 1;
    used.set(o);
    let mut len = 1usize;
    let mut req = pool.last[usize::from(o)];

    loop {
        let c = usize::from(rem[usize::from(req)]);
        if c == 0 {
            return Some(len);
        }
        if c >= 2 {
            return None;
        }
        // Exactly one unused Pokémon starts with `req`.
        let mut t = None;
        for &i in &pool.by_start[usize::from(req)] {
            if used.test(i) {
                continue;
            }
            t = Some(i);
            break;
        }
        let t = t.expect("rem[req]==1 implies one unused index");
        rem[usize::from(pool.first[usize::from(t)])] -= 1;
        used.set(t);
        len += 1;
        req = pool.last[usize::from(t)];
    }
}

/// Group openings by `forced_terminal_game_length` (only those with `Some`).
pub fn openings_by_forced_length(pool: &TerminalPool) -> (Vec<Vec<usize>>, usize) {
    let mut by_len: Vec<Vec<usize>> = Vec::new();
    let mut branched = 0usize;
    for o in 0..pool.n {
        match forced_terminal_game_length(pool, o) {
            Some(len) => {
                if len >= by_len.len() {
                    by_len.resize(len + 1, Vec::new());
                }
                by_len[len].push(o);
            }
            None => branched += 1,
        }
    }
    (by_len, branched)
}

/// Openings where the opponent has at least one reply that ends the game on the next turn.
/// Returns `(openings_with_some_terminal_reply, ordered_opening_reply_pairs)`.
pub fn count_openings_with_terminal_reply(pool: &TerminalPool) -> (usize, u64) {
    let mut openings = 0usize;
    let mut pairs = 0u64;
    for o in 0..pool.n {
        let o = o as u8;
        let req = pool.last[usize::from(o)];
        let mut any = false;
        let mut rem = pool.start_hist;
        rem[usize::from(pool.first[usize::from(o)])] -= 1;

        for &ti in &pool.by_start[usize::from(req)] {
            if ti == o {
                continue;
            }
            let f = usize::from(pool.first[usize::from(ti)]);
            let l = usize::from(pool.last[usize::from(ti)]);
            rem[f] -= 1;
            let term = rem[l] == 0;
            rem[f] += 1;
            if term {
                any = true;
                pairs += 1;
            }
        }

        if any {
            openings += 1;
        }
    }
    (openings, pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen1_full_matches_known_small_k() {
        let pool = TerminalPool::from_count(151);
        assert_eq!(count_terminal_chains(&pool, 1), 13);
        assert_eq!(count_terminal_chains(&pool, 2), 67);
        assert_eq!(count_terminal_chains(&pool, 3), 423);
        assert_eq!(count_terminal_chains(&pool, 4), 2730);
        assert_eq!(count_terminal_chains(&pool, 5), 16940);
    }

    #[test]
    fn forced_length_1_matches_immediate_terminals() {
        let pool = TerminalPool::from_count(151);
        let m1 = immediate_terminal_indices(&pool);
        let (by_len, _) = openings_by_forced_length(&pool);
        assert_eq!(by_len.get(1).map(|v| v.len()).unwrap_or(0), m1.len());
        for &i in &m1 {
            assert_eq!(forced_terminal_game_length(&pool, i), Some(1));
        }
    }

    #[test]
    fn forced_partition_covers_all_openings() {
        let pool = TerminalPool::from_count(151);
        let (by_len, branched) = openings_by_forced_length(&pool);
        let mut sum = 0usize;
        for v in &by_len {
            sum += v.len();
        }
        assert_eq!(sum + branched, pool.n);
    }
}
