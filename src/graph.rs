//! Structural analysis of the 26-node letter graph derived from `(first, last)` edge counts.
//!
//! Provides:
//! - `LetterGraph`: adjacency bitmasks + cached out-degree per letter
//! - `reachable_from`: transitive reachability bitmask (BFS on support graph)
//! - `sccs`: Tarjan SCC condensation → SCC ids + sink-SCC bitmask
//! - `retrograde`: backward induction W/L labels on the 26-node graph
//!
//! All methods operate in O(26²) or better — negligible compared with any search.

use crate::gen1::pair_index;

/// Adjacency structure of the 26-letter support graph.
///
/// `successors[l]` is a bitmask (bit `v` set ↔ `cnt[l][v] > 0`).
/// `out_degree[l]` = total count of remaining edges *starting* at `l`.
#[derive(Clone, Debug)]
pub struct LetterGraph {
    /// Bitmask: bit v set iff cnt[l][v] > 0.
    pub successors: [u32; 26],
    /// Total edge count starting at each letter: Σ_v cnt[l][v].
    pub out_degree: [u32; 26],
}

impl LetterGraph {
    /// Build from a `counts[u*26+v]` matrix.
    pub fn from_counts(counts: &[u8; 676]) -> Self {
        let mut successors = [0u32; 26];
        let mut out_degree = [0u32; 26];
        for u in 0u8..26 {
            for v in 0u8..26 {
                let c = counts[pair_index(u, v)] as u32;
                if c > 0 {
                    successors[u as usize] |= 1 << v;
                    out_degree[u as usize] += c;
                }
            }
        }
        LetterGraph {
            successors,
            out_degree,
        }
    }

    /// Update after decrementing `counts[f][l]` by 1.
    /// `counts` must already reflect the new (decremented) value.
    pub fn on_decrement(&mut self, f: u8, l: u8, counts: &[u8; 676]) {
        let fi = f as usize;
        self.out_degree[fi] -= 1;
        if counts[pair_index(f, l)] == 0 {
            self.successors[fi] &= !(1 << l);
        }
    }

    /// Update after incrementing `counts[f][l]` by 1 (undo a move).
    /// `counts` must already reflect the new (incremented) value.
    pub fn on_increment(&mut self, f: u8, l: u8) {
        let fi = f as usize;
        self.out_degree[fi] += 1;
        self.successors[fi] |= 1 << l;
    }

    /// Transitive reachability from `start` via remaining edges.
    /// Returns a bitmask over 26 letters.  `start` itself is always included.
    pub fn reachable_from(&self, start: u8) -> u32 {
        let mut visited = 0u32;
        let mut queue = 1u32 << start;
        while queue != 0 {
            let bit = queue & queue.wrapping_neg(); // lowest set bit
            queue &= !bit;
            let node = bit.trailing_zeros() as usize;
            if visited & bit != 0 {
                continue;
            }
            visited |= bit;
            // Add successors not yet visited
            queue |= self.successors[node] & !visited;
        }
        visited
    }

    /// Tarjan SCC decomposition on the 26-node support graph.
    ///
    /// Returns `(scc_id, sink_mask)` where:
    /// - `scc_id[l]` = SCC index for letter `l` (0-based, arbitrary order)
    /// - `sink_mask` = bitmask of letters belonging to a sink SCC
    ///   (an SCC with no outgoing edges to letters in other SCCs)
    pub fn sccs(&self) -> ([u8; 26], u32) {
        let mut index_counter = 0u32;
        let mut stack: Vec<u8> = Vec::new();
        let mut on_stack = [false; 26];
        let mut index = [u32::MAX; 26];
        let mut lowlink = [0u32; 26];
        let mut scc_id = [u8::MAX; 26];
        let mut scc_count = 0u8;

        fn strongconnect(
            v: u8,
            successors: &[u32; 26],
            index_counter: &mut u32,
            stack: &mut Vec<u8>,
            on_stack: &mut [bool; 26],
            index: &mut [u32; 26],
            lowlink: &mut [u32; 26],
            scc_id: &mut [u8; 26],
            scc_count: &mut u8,
        ) {
            index[v as usize] = *index_counter;
            lowlink[v as usize] = *index_counter;
            *index_counter += 1;
            stack.push(v);
            on_stack[v as usize] = true;

            let mut succ_mask = successors[v as usize];
            while succ_mask != 0 {
                let bit = succ_mask & succ_mask.wrapping_neg();
                succ_mask &= !bit;
                let w = bit.trailing_zeros() as u8;
                if index[w as usize] == u32::MAX {
                    strongconnect(
                        w,
                        successors,
                        index_counter,
                        stack,
                        on_stack,
                        index,
                        lowlink,
                        scc_id,
                        scc_count,
                    );
                    lowlink[v as usize] = lowlink[v as usize].min(lowlink[w as usize]);
                } else if on_stack[w as usize] {
                    lowlink[v as usize] = lowlink[v as usize].min(index[w as usize]);
                }
            }

            if lowlink[v as usize] == index[v as usize] {
                loop {
                    let w = stack.pop().unwrap();
                    on_stack[w as usize] = false;
                    scc_id[w as usize] = *scc_count;
                    if w == v {
                        break;
                    }
                }
                *scc_count += 1;
            }
        }

        for v in 0u8..26 {
            if index[v as usize] == u32::MAX {
                strongconnect(
                    v,
                    &self.successors,
                    &mut index_counter,
                    &mut stack,
                    &mut on_stack,
                    &mut index,
                    &mut lowlink,
                    &mut scc_id,
                    &mut scc_count,
                );
            }
        }

        // Find sink SCCs: SCCs with no edge leaving to a *different* SCC.
        let mut sink_scc = vec![true; scc_count as usize];
        for u in 0u8..26 {
            let mut succ_mask = self.successors[u as usize];
            while succ_mask != 0 {
                let bit = succ_mask & succ_mask.wrapping_neg();
                succ_mask &= !bit;
                let v = bit.trailing_zeros() as u8;
                if scc_id[u as usize] != scc_id[v as usize] {
                    sink_scc[scc_id[u as usize] as usize] = false;
                }
            }
        }

        let mut sink_mask = 0u32;
        for l in 0u8..26 {
            if scc_id[l as usize] != u8::MAX && sink_scc[scc_id[l as usize] as usize] {
                sink_mask |= 1 << l;
            }
        }

        (scc_id, sink_mask)
    }

    /// Retrograde propagation on the 26-node letter graph.
    ///
    /// Returns per-letter game-theoretic label:
    /// - `Some(false)` = this letter is a **losing** start for the player to move
    /// - `Some(true)`  = this letter is a **winning** start for the player to move
    /// - `None`        = unknown (only possible if graph has deep cycles)
    ///
    /// Note: this operates on the *support* graph (presence of any edge), not on
    /// multiplicities.  It gives exact answers only when the support-level result
    /// is sufficient (e.g., out-degree-0 letters, or when all paths lead to
    /// labeled nodes).  It may leave cyclic regions unknown.
    pub fn retrograde(&self) -> [Option<bool>; 26] {
        let mut label = [None::<bool>; 26];

        // Seed: letters with no successors → immediate loss for player to move.
        for l in 0..26usize {
            if self.successors[l] == 0 {
                label[l] = Some(false); // Loser
            }
        }

        // Iterative propagation until stable.
        let mut changed = true;
        while changed {
            changed = false;
            for l in 0..26usize {
                if label[l].is_some() {
                    continue;
                }
                let mut succ_mask = self.successors[l];
                if succ_mask == 0 {
                    continue;
                }
                // If any successor is Loser → l is Winner
                let mut has_loser_succ = false;
                let mut all_succs_labeled_winner = true;
                while succ_mask != 0 {
                    let bit = succ_mask & succ_mask.wrapping_neg();
                    succ_mask &= !bit;
                    let v = bit.trailing_zeros() as usize;
                    match label[v] {
                        Some(false) => {
                            has_loser_succ = true;
                        }
                        Some(true) => {} // winner successor
                        None => {
                            all_succs_labeled_winner = false;
                        }
                    }
                }
                if has_loser_succ {
                    label[l] = Some(true);
                    changed = true;
                } else if all_succs_labeled_winner {
                    label[l] = Some(false);
                    changed = true;
                }
            }
        }

        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gen1::gen1_opening_counts;

    #[test]
    fn dead_letters_have_no_successors() {
        let counts = gen1_opening_counts();
        let g = LetterGraph::from_counts(&counts);
        // q=16, u=20, x=23, y=24 have no Gen1 starters
        for dead in [b'q' - b'a', b'u' - b'a', b'x' - b'a', b'y' - b'a'] {
            assert_eq!(
                g.successors[dead as usize],
                0,
                "letter {} should have no successors",
                (b'a' + dead) as char
            );
            assert_eq!(g.out_degree[dead as usize], 0);
        }
    }

    #[test]
    fn reachability_includes_start() {
        let counts = gen1_opening_counts();
        let g = LetterGraph::from_counts(&counts);
        let reach = g.reachable_from(b'p' - b'a');
        assert!(reach & (1 << (b'p' - b'a')) != 0);
    }

    #[test]
    fn retrograde_labels_dead_letters_as_losers() {
        let counts = gen1_opening_counts();
        let g = LetterGraph::from_counts(&counts);
        let labels = g.retrograde();
        // q, u, x, y have no starters → immediate loss
        for dead in [b'q' - b'a', b'u' - b'a', b'x' - b'a', b'y' - b'a'] {
            assert_eq!(
                labels[dead as usize],
                Some(false),
                "letter {} should be labeled Loser",
                (b'a' + dead) as char
            );
        }
        // Letters that can reach a dead letter directly are winners
        // e.g. if any Pokémon ends in q/u/x/y, its first letter should be a winner
    }

    #[test]
    fn on_decrement_updates_graph() {
        let mut counts = gen1_opening_counts();
        let mut g = LetterGraph::from_counts(&counts);
        // Ivysaur: i→r (only i-starter)
        let i = (b'i' - b'a') as u8;
        let r = (b'r' - b'a') as u8;
        let old_od = g.out_degree[i as usize];
        counts[pair_index(i, r)] -= 1;
        g.on_decrement(i, r, &counts);
        assert_eq!(g.out_degree[i as usize], old_od - 1);
        // If count hits 0, bit should be cleared
        if counts[pair_index(i, r)] == 0 {
            assert_eq!(g.successors[i as usize] & (1 << r), 0);
        }
    }

    #[test]
    fn scc_sink_mask_nonempty() {
        let counts = gen1_opening_counts();
        let g = LetterGraph::from_counts(&counts);
        let (_, sink_mask) = g.sccs();
        // There must be at least one sink SCC (dead letters form singletons)
        assert_ne!(sink_mask, 0);
    }
}
