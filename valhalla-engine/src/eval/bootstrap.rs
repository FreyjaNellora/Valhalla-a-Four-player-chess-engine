//! Bootstrap evaluator — purely positional assessment.
//!
//! Four components: material balance, PST, king safety, pawn structure.
//! NO tactical terms. Swarm handles all tactics (ADR-008).
//! Exposes `evaluate_breakdown()` for labeled diagnostics alongside
//! the frozen `Evaluator` trait.

use crate::eval::king_safety;
use crate::eval::material;
use crate::eval::pawn_structure;
use crate::eval::pst;
use crate::eval::Evaluator;
use crate::game_state::GameState;
use crate::types::{Player, Score, ALL_PLAYERS};

/// Labeled component breakdown for diagnostics and observer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalBreakdown {
    pub material: Score,
    pub pst: Score,
    pub king_safety: Score,
    pub pawn_structure: Score,
    pub total: Score,
}

/// Bootstrap evaluator: material + PST + king safety + pawn structure.
///
/// Deterministic, no tactical terms, < 1μs target.
pub struct BootstrapEvaluator;

impl BootstrapEvaluator {
    /// Create a new bootstrap evaluator.
    pub fn new() -> Self {
        Self
    }

    /// Full component breakdown for diagnostics/observer.
    /// Not part of the frozen `Evaluator` trait.
    pub fn evaluate_breakdown(&self, state: &GameState) -> EvalBreakdown {
        let stm = state.side_to_move();

        let mat = material::material_balance(state);
        let pst_score = pst_relative(state, stm);
        let king = king_safety_relative(state, stm);
        let pawns = pawn_structure_relative(state, stm);

        let total = mat + pst_score + king + pawns;

        EvalBreakdown {
            material: mat,
            pst: pst_score,
            king_safety: king,
            pawn_structure: pawns,
            total,
        }
    }
}

impl Default for BootstrapEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for BootstrapEvaluator {
    fn evaluate(&self, state: &GameState) -> Score {
        self.evaluate_breakdown(state).total
    }
}

/// PST score: side_to_move's PST sum minus average of active opponents'.
fn pst_relative(state: &GameState, stm: Player) -> Score {
    let my_pst = pst::pst_sum(state, stm);

    let mut opp_total: i32 = 0;
    let mut opp_count: i32 = 0;
    for &p in &ALL_PLAYERS {
        if p != stm && state.is_active(p) {
            opp_total += pst::pst_sum(state, p);
            opp_count += 1;
        }
    }

    if opp_count == 0 {
        return my_pst;
    }

    my_pst - opp_total / opp_count
}

/// King safety: side_to_move's king safety minus average of active opponents'.
fn king_safety_relative(state: &GameState, stm: Player) -> Score {
    let my_king = king_safety::king_safety(state, stm);

    let mut opp_total: Score = 0;
    let mut opp_count: i32 = 0;
    for &p in &ALL_PLAYERS {
        if p != stm && state.is_active(p) {
            opp_total += king_safety::king_safety(state, p);
            opp_count += 1;
        }
    }

    if opp_count == 0 {
        return my_king;
    }

    my_king - opp_total / opp_count as Score
}

/// Pawn structure: side_to_move's score minus average of active opponents'.
fn pawn_structure_relative(state: &GameState, stm: Player) -> Score {
    let my_pawns = pawn_structure::pawn_structure(state, stm);

    let mut opp_total: Score = 0;
    let mut opp_count: i32 = 0;
    for &p in &ALL_PLAYERS {
        if p != stm && state.is_active(p) {
            opp_total += pawn_structure::pawn_structure(state, p);
            opp_count += 1;
        }
    }

    if opp_count == 0 {
        return my_pawns;
    }

    my_pawns - opp_total / opp_count as Score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;

    #[test]
    fn test_starting_position_near_zero() {
        let eval = BootstrapEvaluator::new();
        let state = GameState::new();
        let breakdown = eval.evaluate_breakdown(&state);

        // Symmetric starting position — each component should be 0
        assert_eq!(breakdown.material, 0, "Material should be 0 at start");
        assert_eq!(breakdown.pst, 0, "PST should be 0 at start");
        assert_eq!(breakdown.king_safety, 0, "King safety should be 0 at start");
        assert_eq!(
            breakdown.pawn_structure, 0,
            "Pawn structure should be 0 at start"
        );
        assert_eq!(breakdown.total, 0, "Total should be 0 at start");
    }

    #[test]
    fn test_evaluate_matches_breakdown_total() {
        let eval = BootstrapEvaluator::new();
        let state = GameState::new();
        let score = eval.evaluate(&state);
        let breakdown = eval.evaluate_breakdown(&state);
        assert_eq!(score, breakdown.total);
    }

    #[test]
    fn test_deterministic() {
        let eval = BootstrapEvaluator::new();
        let state = GameState::new();
        let score1 = eval.evaluate(&state);
        let score2 = eval.evaluate(&state);
        assert_eq!(score1, score2, "Evaluator must be deterministic");
    }

    #[test]
    fn test_losing_material_is_negative() {
        let eval = BootstrapEvaluator::new();
        let mut state = GameState::new();

        // Remove a Red pawn
        let pawn_sq = state
            .board
            .pieces_for_player(Player::Red)
            .find(|(_, pt)| *pt == crate::types::PieceType::Pawn)
            .map(|(sq, _)| sq)
            .expect("Red should have pawns");
        state.board.remove_piece(pawn_sq);

        // Red to move — should see negative eval
        let score = eval.evaluate(&state);
        assert!(
            score < 0,
            "Red should have negative eval after losing a pawn, got {}",
            score
        );
    }

    #[test]
    fn test_breakdown_components_labeled() {
        let eval = BootstrapEvaluator::new();
        let state = GameState::new();
        let bd = eval.evaluate_breakdown(&state);

        // Verify all fields are accessible and sum to total
        assert_eq!(
            bd.material + bd.pst + bd.king_safety + bd.pawn_structure,
            bd.total
        );
    }
}
