use super::attack::is_in_check;
use super::castling::generate_castling_moves;
use super::pawns::generate_pawn_moves;
use super::pieces::{generate_king_moves, generate_knight_moves, generate_slider_moves};
use crate::game_state::GameState;
use crate::types::{Move, MoveBuffer, PieceType, Player, MAX_MOVES};
/// Legal move generation.
/// Generates pseudo-legal moves, then filters by king safety.
use arrayvec::ArrayVec;

/// Generate all legal moves for the current side to move.
/// Panics (debug) if called for an eliminated player.
pub fn generate_legal_moves(state: &GameState, buf: &mut ArrayVec<Move, MAX_MOVES>) {
    let player = state.side_to_move();
    debug_assert!(
        state.is_active(player),
        "generate_legal_moves called for non-active player {:?}",
        player
    );

    // Generate all pseudo-legal moves into a temp buffer
    let mut pseudo = ArrayVec::<Move, MAX_MOVES>::new();
    generate_pseudo_legal_moves(state, player, &mut pseudo);

    // Filter: keep only moves where our king is not in check after the move
    for mv in &pseudo {
        if is_legal_move(state, player, *mv) {
            buf.push_move(*mv);
        }
    }
}

/// Generate all pseudo-legal moves for a player (not yet checked for king safety).
fn generate_pseudo_legal_moves(state: &GameState, player: Player, buf: &mut impl MoveBuffer) {
    // Collect pieces first to avoid borrow issues
    let pieces: ArrayVec<(crate::types::Square, PieceType), 32> =
        state.board.pieces_for_player(player).collect();

    for (sq, piece_type) in pieces {
        match piece_type {
            PieceType::Pawn => generate_pawn_moves(state, sq, player, buf),
            PieceType::Knight => generate_knight_moves(state, sq, player, buf),
            PieceType::Bishop | PieceType::Rook | PieceType::Queen | PieceType::PromotedQueen => {
                generate_slider_moves(state, sq, player, piece_type, buf);
            }
            PieceType::King => {
                generate_king_moves(state, sq, player, buf);
            }
        }
    }

    // Castling
    generate_castling_moves(state, player, buf);
}

/// Check if a move is legal (king is not in check after the move).
/// Uses a temporary make/unmake approach.
fn is_legal_move(state: &GameState, player: Player, mv: Move) -> bool {
    let mut test_state = state.clone();
    apply_move_for_legality(&mut test_state, mv);
    !is_in_check(&test_state, player)
}

/// Apply a move to a state for legality checking purposes only.
/// This is a simplified make_move that doesn't handle Zobrist or scoring.
/// It only needs to update the board to check king safety.
fn apply_move_for_legality(state: &mut GameState, mv: Move) {
    if mv.is_en_passant() {
        state.board.move_piece(mv.from, mv.to);
        if let Some(ep_player) = state.ep_pushing_player {
            let (push_dr, push_df) = ep_player.push_direction();
            let cap_r = mv.to.rank() as i8 - push_dr;
            let cap_f = mv.to.file() as i8 - push_df;
            if let Some(cap_sq) = crate::types::Square::from_rank_file(cap_r as u8, cap_f as u8) {
                state.board.remove_piece(cap_sq);
            }
        }
    } else if mv.is_castle() {
        let kingside = mv.is_castle_kingside();
        let player = state.side_to_move();
        let (rook_from, rook_to) = super::castling::get_castle_rook_squares(player, kingside);
        state.board.move_piece(mv.from, mv.to);
        state.board.move_piece(rook_from, rook_to);
    } else if mv.is_promotion() {
        let pawn_piece = state.board.remove_piece(mv.from).unwrap();
        if mv.is_capture() {
            state.board.remove_piece(mv.to);
        }
        let promo_type = mv.promotion.unwrap();
        state.board.place_piece(
            mv.to,
            crate::types::ColoredPiece::new(pawn_piece.player, promo_type),
        );
    } else {
        if mv.is_capture() {
            state.board.remove_piece(mv.to);
        }
        state.board.move_piece(mv.from, mv.to);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position_red_has_20_moves() {
        let state = GameState::new();
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(&state, &mut moves);
        assert_eq!(
            moves.len(),
            20,
            "Red should have 20 legal moves from starting position, got {}. Moves: {:?}",
            moves.len(),
            moves
                .iter()
                .map(|m| format!(
                    "{}->{}({:?})",
                    m.from.to_algebraic(),
                    m.to.to_algebraic(),
                    m.piece
                ))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_all_moves_are_legal() {
        let state = GameState::new();
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(&state, &mut moves);

        for mv in &moves {
            let mut test = state.clone();
            apply_move_for_legality(&mut test, *mv);
            assert!(
                !is_in_check(&test, state.side_to_move()),
                "Move {} -> {} leaves king in check",
                mv.from.to_algebraic(),
                mv.to.to_algebraic()
            );
        }
    }

    #[test]
    fn test_no_moves_to_invalid_squares() {
        let state = GameState::new();
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(&state, &mut moves);

        for mv in &moves {
            assert!(
                mv.from.is_valid_index(),
                "Move source {} is invalid",
                mv.from.index()
            );
            assert!(
                mv.to.is_valid_index(),
                "Move target {} is invalid",
                mv.to.index()
            );
        }
    }
}
