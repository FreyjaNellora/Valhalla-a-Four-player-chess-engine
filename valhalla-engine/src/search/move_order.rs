//! Move ordering — single-pass scoring and sorting.
//!
//! Priority: TT move > killer[0] > killer[1] > captures (MVV-LVA) > history > quiet.
//! All moves are scored in one pass, then sorted descending.

use arrayvec::ArrayVec;

use crate::types::{Move, MAX_MOVES};

use super::history::HistoryTable;
use super::killer::KillerTable;

/// A move paired with its ordering score.
#[derive(Clone, Copy)]
pub struct ScoredMove {
    pub mv: Move,
    pub score: i32,
}

/// Score thresholds for move ordering priority bands.
/// Order: TT move > good captures > killers > bad captures > history > quiet.
const TT_MOVE_SCORE: i32 = i32::MAX;
const KILLER_0_SCORE: i32 = 50_000;
const KILLER_1_SCORE: i32 = 49_000;
/// Base score for captures: victim_value * 100 - attacker_value + CAPTURE_BASE.
/// Good captures (PxQ = 90000-100+100000 = 189900) beat killers.
/// Bad captures (QxP = 10000-900+100000 = 109100) still beat killers.
const CAPTURE_BASE: i32 = 100_000;

/// Score and sort a list of legal moves for search ordering.
///
/// Returns moves sorted by score descending (best first).
pub fn score_moves(
    moves: &ArrayVec<Move, MAX_MOVES>,
    tt_move: Option<Move>,
    killers: &KillerTable,
    history: &HistoryTable,
    ply: usize,
) -> ArrayVec<ScoredMove, MAX_MOVES> {
    let mut scored = ArrayVec::<ScoredMove, MAX_MOVES>::new();

    for &mv in moves.iter() {
        let score = if tt_move == Some(mv) {
            TT_MOVE_SCORE
        } else if mv.is_capture() {
            // MVV-LVA: most valuable victim, least valuable attacker
            let victim_val = mv.captured.map(|p| p.eval_centipawns() as i32).unwrap_or(0);
            let attacker_val = mv.piece.eval_centipawns() as i32;
            CAPTURE_BASE + victim_val * 100 - attacker_val
        } else if let Some(slot) = killers.is_killer(ply, mv) {
            if slot == 0 {
                KILLER_0_SCORE
            } else {
                KILLER_1_SCORE
            }
        } else {
            // History heuristic for quiet moves
            history.get(mv.from.index(), mv.to.index())
        };

        scored.push(ScoredMove { mv, score });
    }

    // Sort descending by score
    scored.sort_unstable_by(|a, b| b.score.cmp(&a.score));
    scored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::history::HistoryTable;
    use crate::search::killer::KillerTable;
    use crate::types::{PieceType, Square};

    fn quiet(from: u8, to: u8) -> Move {
        Move::quiet(Square(from), Square(to), PieceType::Knight)
    }

    fn cap(from: u8, to: u8, piece: PieceType, victim: PieceType) -> Move {
        Move::capture(Square(from), Square(to), piece, victim)
    }

    #[test]
    fn test_tt_move_first() {
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        let tt_mv = quiet(10, 20);
        moves.push(quiet(30, 40));
        moves.push(tt_mv);
        moves.push(quiet(50, 60));

        let kt = KillerTable::new();
        let ht = HistoryTable::new();
        let scored = score_moves(&moves, Some(tt_mv), &kt, &ht, 0);

        assert_eq!(scored[0].mv, tt_mv);
    }

    #[test]
    fn test_capture_before_quiet() {
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        moves.push(quiet(10, 20));
        let capture = cap(30, 40, PieceType::Pawn, PieceType::Queen);
        moves.push(capture);

        let kt = KillerTable::new();
        let ht = HistoryTable::new();
        let scored = score_moves(&moves, None, &kt, &ht, 0);

        assert_eq!(scored[0].mv, capture);
    }

    #[test]
    fn test_mvv_lva_ordering() {
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        // Pawn takes queen (great)
        let pxq = cap(10, 20, PieceType::Pawn, PieceType::Queen);
        // Queen takes pawn (bad)
        let qxp = cap(30, 40, PieceType::Queen, PieceType::Pawn);
        moves.push(qxp);
        moves.push(pxq);

        let kt = KillerTable::new();
        let ht = HistoryTable::new();
        let scored = score_moves(&moves, None, &kt, &ht, 0);

        assert_eq!(scored[0].mv, pxq); // PxQ first (higher MVV-LVA)
    }

    #[test]
    fn test_killer_before_quiet_after_capture() {
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        let killer = quiet(10, 20);
        let normal = quiet(30, 40);
        let capture = cap(50, 60, PieceType::Pawn, PieceType::Rook);

        moves.push(normal);
        moves.push(killer);
        moves.push(capture);

        let mut kt = KillerTable::new();
        kt.store(0, killer);
        let ht = HistoryTable::new();

        let scored = score_moves(&moves, None, &kt, &ht, 0);

        assert_eq!(scored[0].mv, capture); // captures first
        assert_eq!(scored[1].mv, killer); // killer second
        assert_eq!(scored[2].mv, normal); // quiet last
    }

    #[test]
    fn test_history_ordering_for_quiet() {
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        let good = quiet(10, 20);
        let bad = quiet(30, 40);
        moves.push(bad);
        moves.push(good);

        let kt = KillerTable::new();
        let mut ht = HistoryTable::new();
        ht.update(10, 20, 8); // high history for good move

        let scored = score_moves(&moves, None, &kt, &ht, 0);
        assert_eq!(scored[0].mv, good);
    }

    #[test]
    fn test_full_priority_order() {
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        let tt_mv = quiet(10, 20);
        let killer = quiet(30, 40);
        let capture = cap(50, 60, PieceType::Knight, PieceType::Bishop);
        let history_mv = quiet(70, 80);
        let plain = quiet(90, 100);

        moves.push(plain);
        moves.push(history_mv);
        moves.push(capture);
        moves.push(killer);
        moves.push(tt_mv);

        let mut kt = KillerTable::new();
        kt.store(0, killer);
        let mut ht = HistoryTable::new();
        ht.update(70, 80, 4); // +16

        let scored = score_moves(&moves, Some(tt_mv), &kt, &ht, 0);

        assert_eq!(scored[0].mv, tt_mv); // TT first
        assert_eq!(scored[1].mv, capture); // capture second
        assert_eq!(scored[2].mv, killer); // killer third
        assert_eq!(scored[3].mv, history_mv); // history fourth
        assert_eq!(scored[4].mv, plain); // plain last
    }
}
