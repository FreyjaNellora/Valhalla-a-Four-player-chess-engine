//! Search module — game tree exploration.
//!
//! The `Searcher` trait is the frozen interface contract for all searchers.
//! OPPS implements it first; the phase-separated hybrid wraps it later.

use crate::game_state::GameState;
use crate::types::{Move, Score};

/// Result of a completed search.
pub struct SearchResult {
    /// The best move found.
    pub best_move: Move,
    /// Evaluation score from the root player's perspective.
    pub score: Score,
    /// Search depth reached (must be divisible by 4 for final results).
    pub depth: u32,
    /// Total nodes visited during search.
    pub nodes: u64,
}

/// Frozen searcher trait. Do not modify this trait after Phase 2.
///
/// `depth` must be divisible by 4 for valid final results.
/// `&mut self` allows internal state (TT, statistics, history tables).
pub trait Searcher {
    fn search(&mut self, state: &GameState, depth: u32) -> SearchResult;
}
