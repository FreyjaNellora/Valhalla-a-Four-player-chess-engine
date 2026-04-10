use super::constants::MAX_MOVES;
use super::piece::{ColoredPiece, PieceType};
use super::player::Player;
use super::square::Square;
/// Move representation, undo information, and move buffer trait.
/// All types are fixed-size with no heap allocation.
use arrayvec::ArrayVec;

/// Bitflags for move classification.
pub struct MoveFlags;

impl MoveFlags {
    pub const QUIET: u8 = 0;
    pub const CAPTURE: u8 = 1;
    pub const DOUBLE_PUSH: u8 = 2;
    pub const EN_PASSANT: u8 = 4;
    pub const CASTLE_KINGSIDE: u8 = 8;
    pub const CASTLE_QUEENSIDE: u8 = 16;
    pub const PROMOTION: u8 = 32;
}

/// A chess move. Fixed-size, no heap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: PieceType,
    pub captured: Option<PieceType>,
    pub promotion: Option<PieceType>,
    pub flags: u8,
}

impl Move {
    /// Null/sentinel move.
    pub const NULL: Move = Move {
        from: Square(0),
        to: Square(0),
        piece: PieceType::Pawn,
        captured: None,
        promotion: None,
        flags: 0,
    };

    /// Create a quiet (non-capture) move.
    #[inline]
    pub const fn quiet(from: Square, to: Square, piece: PieceType) -> Self {
        Self {
            from,
            to,
            piece,
            captured: None,
            promotion: None,
            flags: MoveFlags::QUIET,
        }
    }

    /// Create a capture move.
    #[inline]
    pub const fn capture(from: Square, to: Square, piece: PieceType, captured: PieceType) -> Self {
        Self {
            from,
            to,
            piece,
            captured: Some(captured),
            promotion: None,
            flags: MoveFlags::CAPTURE,
        }
    }

    /// Create a double pawn push.
    #[inline]
    pub const fn double_push(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            piece: PieceType::Pawn,
            captured: None,
            promotion: None,
            flags: MoveFlags::DOUBLE_PUSH,
        }
    }

    /// Create an en passant capture.
    #[inline]
    pub const fn en_passant(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            piece: PieceType::Pawn,
            captured: Some(PieceType::Pawn),
            promotion: None,
            flags: MoveFlags::EN_PASSANT | MoveFlags::CAPTURE,
        }
    }

    /// Create a castling move (king's move; rook is handled in make_move).
    #[inline]
    pub const fn castle_kingside(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            piece: PieceType::King,
            captured: None,
            promotion: None,
            flags: MoveFlags::CASTLE_KINGSIDE,
        }
    }

    /// Create a queenside castling move.
    #[inline]
    pub const fn castle_queenside(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            piece: PieceType::King,
            captured: None,
            promotion: None,
            flags: MoveFlags::CASTLE_QUEENSIDE,
        }
    }

    /// Create a promotion move (non-capture).
    #[inline]
    pub const fn promotion(from: Square, to: Square, promo_piece: PieceType) -> Self {
        Self {
            from,
            to,
            piece: PieceType::Pawn,
            captured: None,
            promotion: Some(promo_piece),
            flags: MoveFlags::PROMOTION,
        }
    }

    /// Create a promotion-capture move.
    #[inline]
    pub const fn promotion_capture(
        from: Square,
        to: Square,
        captured: PieceType,
        promo_piece: PieceType,
    ) -> Self {
        Self {
            from,
            to,
            piece: PieceType::Pawn,
            captured: Some(captured),
            promotion: Some(promo_piece),
            flags: MoveFlags::PROMOTION | MoveFlags::CAPTURE,
        }
    }

    #[inline]
    pub const fn is_capture(self) -> bool {
        self.flags & MoveFlags::CAPTURE != 0
    }

    #[inline]
    pub const fn is_en_passant(self) -> bool {
        self.flags & MoveFlags::EN_PASSANT != 0
    }

    #[inline]
    pub const fn is_castle(self) -> bool {
        self.flags & (MoveFlags::CASTLE_KINGSIDE | MoveFlags::CASTLE_QUEENSIDE) != 0
    }

    #[inline]
    pub const fn is_castle_kingside(self) -> bool {
        self.flags & MoveFlags::CASTLE_KINGSIDE != 0
    }

    #[inline]
    pub const fn is_castle_queenside(self) -> bool {
        self.flags & MoveFlags::CASTLE_QUEENSIDE != 0
    }

    #[inline]
    pub const fn is_promotion(self) -> bool {
        self.flags & MoveFlags::PROMOTION != 0
    }

    #[inline]
    pub const fn is_double_push(self) -> bool {
        self.flags & MoveFlags::DOUBLE_PUSH != 0
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.from.0 == 0 && self.to.0 == 0 && self.flags == 0
    }
}

/// Everything needed to unmake a move. Fixed-size, no heap.
#[derive(Clone, Copy, Debug)]
pub struct MoveUndo {
    /// The piece (with color) that was on the target square before the move.
    pub captured_piece: Option<ColoredPiece>,
    /// Castling rights before the move (8 bits).
    pub castling_rights: u8,
    /// En passant target square before the move.
    pub en_passant: Option<Square>,
    /// Which player created the en passant opportunity.
    pub ep_pushing_player: Option<Player>,
    /// Half-move clock before the move (50-move rule).
    pub half_move_clock: u16,
    /// Zobrist hash before the move (for direct restore on unmake).
    pub zobrist_hash: u64,
}

/// Trait for collecting generated moves. Implemented for ArrayVec.
pub trait MoveBuffer {
    fn push_move(&mut self, mv: Move);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl MoveBuffer for ArrayVec<Move, MAX_MOVES> {
    #[inline]
    fn push_move(&mut self, mv: Move) {
        self.push(mv);
    }

    #[inline]
    fn len(&self) -> usize {
        ArrayVec::len(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_no_heap() {
        // Move should be small and stack-allocated
        assert!(
            std::mem::size_of::<Move>() <= 16,
            "Move is {} bytes",
            std::mem::size_of::<Move>()
        );
    }

    #[test]
    fn test_move_undo_no_heap() {
        // MoveUndo should be small and stack-allocated
        assert!(
            std::mem::size_of::<MoveUndo>() <= 32,
            "MoveUndo is {} bytes",
            std::mem::size_of::<MoveUndo>()
        );
    }

    #[test]
    fn test_quiet_move() {
        let mv = Move::quiet(Square(7), Square(21), PieceType::King);
        assert!(!mv.is_capture());
        assert!(!mv.is_castle());
        assert!(!mv.is_promotion());
        assert!(!mv.is_en_passant());
        assert!(!mv.is_double_push());
    }

    #[test]
    fn test_capture_move() {
        let mv = Move::capture(Square(7), Square(21), PieceType::Rook, PieceType::Pawn);
        assert!(mv.is_capture());
        assert_eq!(mv.captured, Some(PieceType::Pawn));
    }

    #[test]
    fn test_castle_flags() {
        let ks = Move::castle_kingside(Square(7), Square(9));
        assert!(ks.is_castle());
        assert!(ks.is_castle_kingside());
        assert!(!ks.is_castle_queenside());

        let qs = Move::castle_queenside(Square(7), Square(5));
        assert!(qs.is_castle());
        assert!(!qs.is_castle_kingside());
        assert!(qs.is_castle_queenside());
    }

    #[test]
    fn test_promotion_move() {
        let mv = Move::promotion(Square(17), Square(31), PieceType::PromotedQueen);
        assert!(mv.is_promotion());
        assert!(!mv.is_capture());
        assert_eq!(mv.promotion, Some(PieceType::PromotedQueen));
    }

    #[test]
    fn test_en_passant_move() {
        let mv = Move::en_passant(Square(17), Square(31));
        assert!(mv.is_en_passant());
        assert!(mv.is_capture());
        assert_eq!(mv.captured, Some(PieceType::Pawn));
    }

    #[test]
    fn test_double_push() {
        let mv = Move::double_push(Square(17), Square(45));
        assert!(mv.is_double_push());
        assert!(!mv.is_capture());
    }

    #[test]
    fn test_null_move() {
        assert!(Move::NULL.is_null());
    }

    #[test]
    fn test_move_buffer() {
        let mut buf = ArrayVec::<Move, MAX_MOVES>::new();
        assert!(buf.is_empty());
        buf.push_move(Move::quiet(Square(7), Square(21), PieceType::King));
        assert_eq!(buf.len(), 1);
    }
}
