use crate::game_state::GameState;
/// Pawn move generation for all four player orientations.
/// Handles: push, double push, captures, en passant, promotion.
/// EN PASSANT uses board scan, NOT player.prev() (ADR-012).
use crate::types::{Move, MoveBuffer, PieceType, Player, Square, BOARD_SIZE};

/// Generate all pseudo-legal pawn moves for a pawn at the given square.
pub fn generate_pawn_moves(
    state: &GameState,
    from: Square,
    player: Player,
    buf: &mut impl MoveBuffer,
) {
    let rank = from.rank();
    let file = from.file();
    let (push_dr, push_df) = player.push_direction();

    // --- Single push ---
    let push_r = rank as i8 + push_dr;
    let push_f = file as i8 + push_df;

    if push_r >= 0 && push_r < BOARD_SIZE as i8 && push_f >= 0 && push_f < BOARD_SIZE as i8 {
        if let Some(push_sq) = Square::from_rank_file(push_r as u8, push_f as u8) {
            if state.board.get(push_sq).is_none() {
                if is_promotion(player, push_r as u8, push_f as u8) {
                    // Promotion push
                    add_promotions(from, push_sq, None, buf);
                } else {
                    buf.push_move(Move::quiet(from, push_sq, PieceType::Pawn));

                    // --- Double push (only from starting position) ---
                    if player.is_pawn_on_start(rank, file) {
                        let dbl_r = push_r + push_dr;
                        let dbl_f = push_f + push_df;
                        if dbl_r >= 0
                            && dbl_r < BOARD_SIZE as i8
                            && dbl_f >= 0
                            && dbl_f < BOARD_SIZE as i8
                        {
                            if let Some(dbl_sq) = Square::from_rank_file(dbl_r as u8, dbl_f as u8) {
                                if state.board.get(dbl_sq).is_none() {
                                    buf.push_move(Move::double_push(from, dbl_sq));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // --- Captures ---
    for (cap_dr, cap_df) in player.capture_directions() {
        let cap_r = rank as i8 + cap_dr;
        let cap_f = file as i8 + cap_df;
        if cap_r < 0 || cap_r >= BOARD_SIZE as i8 || cap_f < 0 || cap_f >= BOARD_SIZE as i8 {
            continue;
        }
        if let Some(cap_sq) = Square::from_rank_file(cap_r as u8, cap_f as u8) {
            // Normal capture
            if let Some(target) = state.board.get(cap_sq) {
                if target.player != player
                    && can_pawn_capture(state, target.player, target.piece_type)
                {
                    if is_promotion(player, cap_r as u8, cap_f as u8) {
                        add_promotions(from, cap_sq, Some(target.piece_type), buf);
                    } else {
                        buf.push_move(Move::capture(
                            from,
                            cap_sq,
                            PieceType::Pawn,
                            target.piece_type,
                        ));
                    }
                }
            }

            // En passant capture
            if let Some(ep_sq) = state.en_passant {
                if cap_sq == ep_sq {
                    buf.push_move(Move::en_passant(from, cap_sq));
                }
            }
        }
    }
}

/// Check if moving to (rank, file) is a promotion for this player.
#[inline]
fn is_promotion(player: Player, rank: u8, file: u8) -> bool {
    player.is_promotion_square(rank, file)
}

/// Add all 4 promotion moves (PromotedQueen, Knight, Bishop, Rook).
fn add_promotions(
    from: Square,
    to: Square,
    captured: Option<PieceType>,
    buf: &mut impl MoveBuffer,
) {
    let promos = [
        PieceType::PromotedQueen,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
    ];
    match captured {
        Some(cap) => {
            for promo in promos {
                buf.push_move(Move::promotion_capture(from, to, cap, promo));
            }
        }
        None => {
            for promo in promos {
                buf.push_move(Move::promotion(from, to, promo));
            }
        }
    }
}

/// Check if a pawn can capture this target. DKW non-king pieces are uncapturable.
#[inline]
fn can_pawn_capture(state: &GameState, target_player: Player, target_piece: PieceType) -> bool {
    if state.is_dkw(target_player) {
        target_piece == PieceType::King
    } else {
        true
    }
}
