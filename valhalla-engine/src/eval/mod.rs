//! Evaluation module — positional assessment of game states.
//!
//! The `Evaluator` trait is the frozen interface contract for all evaluators.
//! Bootstrap evaluator is the first implementation; NNUE replaces it later.

pub mod bootstrap;
pub mod king_safety;
pub mod material;
pub mod pawn_structure;
pub mod pst;

pub use bootstrap::{BootstrapEvaluator, EvalBreakdown};

use crate::game_state::GameState;
use crate::types::Score;

/// Frozen evaluator trait. Do not modify this trait after Phase 2.
///
/// Pure positional assessment. No tactical terms — swarm handles all tactics.
/// Returns score from the perspective of `state.side_to_move()`.
pub trait Evaluator: Send + Sync {
    fn evaluate(&self, state: &GameState) -> Score;
}
