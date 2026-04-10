//! Named constants for the 4-player chess engine.
//! Source: 4PC_RULES_REFERENCE.md Section 14.

/// Board dimension (ranks and files).
pub const BOARD_SIZE: u8 = 14;

/// Total squares on the 14x14 grid.
pub const TOTAL_SQUARES: usize = 196;

/// Number of playable squares (196 - 36 invalid corners).
pub const VALID_SQUARES: usize = 160;

/// Number of invalid corner squares (four 3x3 corners).
pub const INVALID_CORNER_COUNT: usize = 36;

/// Size of each corner dead zone.
pub const CORNER_SIZE: u8 = 3;

/// Number of players.
pub const PLAYERS: usize = 4;

/// Starting pieces per player (8 back rank + 8 pawns).
pub const PIECES_PER_PLAYER: usize = 16;

/// Maximum pieces per player including promotions.
pub const MAX_PIECES_PER_PLAYER: usize = 32;

/// Sentinel value for king square when player is eliminated.
pub const ELIMINATED_KING_SENTINEL: u8 = 255;

/// Number of castling rights bits (2 per player x 4 players).
pub const CASTLING_RIGHTS_BITS: u8 = 8;

/// Maximum legal moves in a single position.
pub const MAX_MOVES: usize = 256;

// --- FFA Scoring Constants ---

/// FFA points for checkmating a king.
pub const CHECKMATE_POINTS: i16 = 20;

/// FFA points for self-stalemate.
pub const STALEMATE_POINTS: i16 = 20;

/// Point lead needed to claim win (FFA).
pub const CLAIM_WIN_THRESHOLD: i16 = 21;

// --- Eval Constants (centipawns) ---

/// Pawn value in centipawns.
pub const PAWN_EVAL: i16 = 100;

/// Knight value in centipawns.
pub const KNIGHT_EVAL: i16 = 300;

/// Bishop value in centipawns.
pub const BISHOP_EVAL: i16 = 350;

/// Rook value in centipawns.
pub const ROOK_EVAL: i16 = 500;

/// Queen value in centipawns.
pub const QUEEN_EVAL: i16 = 900;

/// Maximum half-moves before forced draw.
pub const MAX_GAME_LENGTH: u16 = 1024;
