//! Optimal Pokémon Shiritori solver — Gen 1 English names.

pub mod agents;
pub mod analysis;
pub mod gen1;
pub mod graph;
pub mod normalize;
pub mod solver;
pub mod tournament;

// play.rs uses std::io (stdin) which is unavailable in WASM.
#[cfg(not(target_arch = "wasm32"))]
pub mod play;

// WASM API — compiled only when the `wasm` feature is enabled.
#[cfg(feature = "wasm")]
pub mod wasm;
