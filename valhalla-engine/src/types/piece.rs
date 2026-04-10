/// Piece types for four-player chess.
/// PromotedQueen moves like a queen but is worth only 1 FFA point on capture.
use super::player::Player;
use crate::types::constants::*;

/// Chess piece types. PromotedQueen has dual value: 900cp eval, 1 FFA point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
    PromotedQueen = 6,
}

/// A piece with its owning player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ColoredPiece {
    pub player: Player,
    pub piece_type: PieceType,
}

impl ColoredPiece {
    /// Create a new colored piece.
    #[inline]
    pub const fn new(player: Player, piece_type: PieceType) -> Self {
        Self { player, piece_type }
    }
}

impl PieceType {
    /// Whether this piece is a sliding piece (bishop, rook, queen, promoted queen).
    #[inline]
    pub const fn is_slider(self) -> bool {
        matches!(
            self,
            PieceType::Bishop | PieceType::Rook | PieceType::Queen | PieceType::PromotedQueen
        )
    }

    /// FFA capture points for this piece type.
    /// This is the GAME SCORING value, not the eval value.
    #[inline]
    pub const fn ffa_points(self) -> i16 {
        match self {
            PieceType::Pawn => 1,
            PieceType::Knight => 3,
            PieceType::Bishop => 5,
            PieceType::Rook => 5,
            PieceType::Queen => 9,
            PieceType::King => 0,
            PieceType::PromotedQueen => 1, // Moves like queen, worth 1 point
        }
    }

    /// Centipawn eval value for search/NNUE.
    /// This is the EVAL value, separate from FFA points.
    #[inline]
    pub const fn eval_centipawns(self) -> i16 {
        match self {
            PieceType::Pawn => PAWN_EVAL,
            PieceType::Knight => KNIGHT_EVAL,
            PieceType::Bishop => BISHOP_EVAL,
            PieceType::Rook => ROOK_EVAL,
            PieceType::Queen => QUEEN_EVAL,
            PieceType::King => 0, // King has no material value
            PieceType::PromotedQueen => QUEEN_EVAL, // 900cp in eval
        }
    }

    /// Movement directions for sliding pieces. Returns empty slice for non-sliders.
    pub const fn slide_directions(self) -> &'static [(i8, i8)] {
        match self {
            PieceType::Bishop => &[(-1, -1), (-1, 1), (1, -1), (1, 1)],
            PieceType::Rook => &[(-1, 0), (1, 0), (0, -1), (0, 1)],
            PieceType::Queen | PieceType::PromotedQueen => &[
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ],
            _ => &[],
        }
    }

    /// FEN4 character for this piece type (uppercase).
    #[inline]
    pub const fn fen_char(self) -> char {
        match self {
            PieceType::Pawn => 'P',
            PieceType::Knight => 'N',
            PieceType::Bishop => 'B',
            PieceType::Rook => 'R',
            PieceType::Queen => 'Q',
            PieceType::King => 'K',
            PieceType::PromotedQueen => 'W', // W for promoted (1-point) queen
        }
    }

    /// Parse a piece type from FEN4 character.
    #[inline]
    pub const fn from_fen_char(c: char) -> Option<PieceType> {
        match c {
            'P' => Some(PieceType::Pawn),
            'N' => Some(PieceType::Knight),
            'B' => Some(PieceType::Bishop),
            'R' => Some(PieceType::Rook),
            'Q' => Some(PieceType::Queen),
            'K' => Some(PieceType::King),
            'W' => Some(PieceType::PromotedQueen),
            _ => None,
        }
    }
}

/// Knight movement offsets (L-shapes).
pub const KNIGHT_OFFSETS: [(i8, i8); 8] = [
    (-2, -1),
    (-2, 1),
    (-1, -2),
    (-1, 2),
    (1, -2),
    (1, 2),
    (2, -1),
    (2, 1),
];

/// King movement offsets (all 8 adjacent squares).
pub const KING_OFFSETS: [(i8, i8); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promoted_queen_dual_value() {
        // FFA points: 1 (it was a pawn)
        assert_eq!(PieceType::PromotedQueen.ffa_points(), 1);
        // Eval: 900cp (it moves like a queen)
        assert_eq!(PieceType::PromotedQueen.eval_centipawns(), 900);
    }

    #[test]
    fn test_slider_detection() {
        assert!(!PieceType::Pawn.is_slider());
        assert!(!PieceType::Knight.is_slider());
        assert!(PieceType::Bishop.is_slider());
        assert!(PieceType::Rook.is_slider());
        assert!(PieceType::Queen.is_slider());
        assert!(!PieceType::King.is_slider());
        assert!(PieceType::PromotedQueen.is_slider());
    }

    #[test]
    fn test_ffa_points() {
        assert_eq!(PieceType::Pawn.ffa_points(), 1);
        assert_eq!(PieceType::Knight.ffa_points(), 3);
        assert_eq!(PieceType::Bishop.ffa_points(), 5);
        assert_eq!(PieceType::Rook.ffa_points(), 5);
        assert_eq!(PieceType::Queen.ffa_points(), 9);
        assert_eq!(PieceType::King.ffa_points(), 0);
    }

    #[test]
    fn test_slide_directions_count() {
        assert_eq!(PieceType::Bishop.slide_directions().len(), 4);
        assert_eq!(PieceType::Rook.slide_directions().len(), 4);
        assert_eq!(PieceType::Queen.slide_directions().len(), 8);
        assert_eq!(PieceType::PromotedQueen.slide_directions().len(), 8);
        assert_eq!(PieceType::Pawn.slide_directions().len(), 0);
        assert_eq!(PieceType::Knight.slide_directions().len(), 0);
    }

    #[test]
    fn test_fen_char_roundtrip() {
        let pieces = [
            PieceType::Pawn,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
            PieceType::King,
            PieceType::PromotedQueen,
        ];
        for p in pieces {
            assert_eq!(PieceType::from_fen_char(p.fen_char()), Some(p));
        }
    }
}
