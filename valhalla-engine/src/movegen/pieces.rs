use crate::game_state::GameState;
/// Pseudo-legal move generation for non-pawn pieces.
/// Knight, Bishop, Rook, Queen, PromotedQueen, King.
use crate::types::{
    Move, MoveBuffer, PieceType, Player, Square, BOARD_SIZE, KING_OFFSETS, KNIGHT_OFFSETS,
};

/// Generate pseudo-legal moves for a knight at the given square.
pub fn generate_knight_moves(
    state: &GameState,
    from: Square,
    player: Player,
    buf: &mut impl MoveBuffer,
) {
    let rank = from.rank() as i8;
    let file = from.file() as i8;

    for (dr, df) in KNIGHT_OFFSETS {
        let nr = rank + dr;
        let nf = file + df;
        if nr < 0 || nr >= BOARD_SIZE as i8 || nf < 0 || nf >= BOARD_SIZE as i8 {
            continue;
        }
        if let Some(to) = Square::from_rank_file(nr as u8, nf as u8) {
            match state.board.get(to) {
                None => {
                    buf.push_move(Move::quiet(from, to, PieceType::Knight));
                }
                Some(cp) => {
                    if cp.player != player && can_capture(state, cp.player, cp.piece_type) {
                        buf.push_move(Move::capture(from, to, PieceType::Knight, cp.piece_type));
                    }
                }
            }
        }
    }
}

/// Generate pseudo-legal moves for a king at the given square (no castling).
pub fn generate_king_moves(
    state: &GameState,
    from: Square,
    player: Player,
    buf: &mut impl MoveBuffer,
) {
    let rank = from.rank() as i8;
    let file = from.file() as i8;

    for (dr, df) in KING_OFFSETS {
        let nr = rank + dr;
        let nf = file + df;
        if nr < 0 || nr >= BOARD_SIZE as i8 || nf < 0 || nf >= BOARD_SIZE as i8 {
            continue;
        }
        if let Some(to) = Square::from_rank_file(nr as u8, nf as u8) {
            match state.board.get(to) {
                None => {
                    buf.push_move(Move::quiet(from, to, PieceType::King));
                }
                Some(cp) => {
                    if cp.player != player && can_capture(state, cp.player, cp.piece_type) {
                        buf.push_move(Move::capture(from, to, PieceType::King, cp.piece_type));
                    }
                }
            }
        }
    }
}

/// Generate pseudo-legal moves for a sliding piece (bishop, rook, queen, promoted queen).
pub fn generate_slider_moves(
    state: &GameState,
    from: Square,
    player: Player,
    piece_type: PieceType,
    buf: &mut impl MoveBuffer,
) {
    let rank = from.rank() as i8;
    let file = from.file() as i8;

    for &(dr, df) in piece_type.slide_directions() {
        let mut r = rank + dr;
        let mut f = file + df;
        while r >= 0 && r < BOARD_SIZE as i8 && f >= 0 && f < BOARD_SIZE as i8 {
            match Square::from_rank_file(r as u8, f as u8) {
                Some(to) => {
                    match state.board.get(to) {
                        None => {
                            buf.push_move(Move::quiet(from, to, piece_type));
                        }
                        Some(cp) => {
                            if cp.player != player && can_capture(state, cp.player, cp.piece_type) {
                                buf.push_move(Move::capture(from, to, piece_type, cp.piece_type));
                            }
                            break; // Blocked (whether capture or friendly)
                        }
                    }
                }
                None => break, // Invalid corner square — ray stops
            }
            r += dr;
            f += df;
        }
    }
}

/// Check if a piece can be captured. DKW non-king pieces are uncapturable walls.
#[inline]
fn can_capture(state: &GameState, target_player: Player, target_piece: PieceType) -> bool {
    if state.is_dkw(target_player) {
        // Only DKW king can be captured
        target_piece == PieceType::King
    } else {
        true
    }
}
