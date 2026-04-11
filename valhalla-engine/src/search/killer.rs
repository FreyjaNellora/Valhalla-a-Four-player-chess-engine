//! Killer move heuristic — remembers quiet moves that caused beta cutoffs.
//!
//! Two killer slots per ply. Fixed-size array, no heap.

use crate::types::Move;

/// Maximum search depth in plies (depth 12 + 8 extension plies = 20 rounds = 80 plies, but
/// with 4 players we measure in full plies from root, so 52 is generous headroom).
pub const MAX_PLY: usize = 52;

/// Stores two killer moves per ply.
pub struct KillerTable {
    killers: [[Move; 2]; MAX_PLY],
}

impl KillerTable {
    /// Create a new empty killer table.
    pub fn new() -> Self {
        Self {
            killers: [[Move::NULL; 2]; MAX_PLY],
        }
    }

    /// Store a killer move at the given ply.
    /// If the move is already killer[0], do nothing.
    /// Otherwise shift killer[0] to killer[1] and store new move at killer[0].
    #[inline]
    pub fn store(&mut self, ply: usize, mv: Move) {
        if ply >= MAX_PLY {
            return;
        }
        if self.killers[ply][0] == mv {
            return;
        }
        self.killers[ply][1] = self.killers[ply][0];
        self.killers[ply][0] = mv;
    }

    /// Check if a move is a killer at the given ply.
    /// Returns `Some(0)` for primary killer, `Some(1)` for secondary, `None` if not a killer.
    #[inline]
    pub fn is_killer(&self, ply: usize, mv: Move) -> Option<u8> {
        if ply >= MAX_PLY {
            return None;
        }
        if !self.killers[ply][0].is_null() && self.killers[ply][0] == mv {
            return Some(0);
        }
        if !self.killers[ply][1].is_null() && self.killers[ply][1] == mv {
            return Some(1);
        }
        None
    }

    /// Clear all killers (e.g., for a new search).
    pub fn clear(&mut self) {
        self.killers = [[Move::NULL; 2]; MAX_PLY];
    }
}

impl Default for KillerTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PieceType, Square};

    fn mv_a() -> Move {
        Move::quiet(Square(31), Square(45), PieceType::Knight)
    }
    fn mv_b() -> Move {
        Move::quiet(Square(50), Square(64), PieceType::Bishop)
    }
    fn mv_c() -> Move {
        Move::quiet(Square(70), Square(84), PieceType::Rook)
    }

    #[test]
    fn test_store_and_check() {
        let mut kt = KillerTable::new();
        kt.store(0, mv_a());
        assert_eq!(kt.is_killer(0, mv_a()), Some(0));
        assert_eq!(kt.is_killer(0, mv_b()), None);
    }

    #[test]
    fn test_two_killers() {
        let mut kt = KillerTable::new();
        kt.store(0, mv_a());
        kt.store(0, mv_b());
        assert_eq!(kt.is_killer(0, mv_b()), Some(0)); // newest is primary
        assert_eq!(kt.is_killer(0, mv_a()), Some(1)); // previous shifted to secondary
    }

    #[test]
    fn test_third_killer_evicts_oldest() {
        let mut kt = KillerTable::new();
        kt.store(0, mv_a());
        kt.store(0, mv_b());
        kt.store(0, mv_c());
        assert_eq!(kt.is_killer(0, mv_c()), Some(0));
        assert_eq!(kt.is_killer(0, mv_b()), Some(1));
        assert_eq!(kt.is_killer(0, mv_a()), None); // evicted
    }

    #[test]
    fn test_duplicate_store_no_shift() {
        let mut kt = KillerTable::new();
        kt.store(0, mv_a());
        kt.store(0, mv_b());
        kt.store(0, mv_b()); // duplicate
        assert_eq!(kt.is_killer(0, mv_b()), Some(0));
        assert_eq!(kt.is_killer(0, mv_a()), Some(1)); // not evicted
    }

    #[test]
    fn test_different_plies_independent() {
        let mut kt = KillerTable::new();
        kt.store(0, mv_a());
        kt.store(1, mv_b());
        assert_eq!(kt.is_killer(0, mv_a()), Some(0));
        assert_eq!(kt.is_killer(0, mv_b()), None);
        assert_eq!(kt.is_killer(1, mv_b()), Some(0));
        assert_eq!(kt.is_killer(1, mv_a()), None);
    }

    #[test]
    fn test_clear() {
        let mut kt = KillerTable::new();
        kt.store(0, mv_a());
        kt.clear();
        assert_eq!(kt.is_killer(0, mv_a()), None);
    }

    #[test]
    fn test_out_of_bounds_safe() {
        let mut kt = KillerTable::new();
        kt.store(MAX_PLY + 5, mv_a()); // should not panic
        assert_eq!(kt.is_killer(MAX_PLY + 5, mv_a()), None);
    }
}
