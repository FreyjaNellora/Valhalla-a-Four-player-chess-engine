//! Score type for evaluation results.
//! Centipawn score from the perspective of the side to move.

/// Centipawn score. Positive = good for side_to_move, negative = bad.
pub type Score = i32;

/// Score for checkmate (winning side).
pub const SCORE_MATE: Score = 100_000;

/// Score for a drawn position.
pub const SCORE_DRAW: Score = 0;

/// Upper bound sentinel. Use for alpha-beta initialization.
pub const SCORE_INFINITY: Score = i32::MAX;

/// Lower bound sentinel. `MIN + 1` avoids overflow on negation.
pub const SCORE_NEG_INFINITY: Score = i32::MIN + 1;
