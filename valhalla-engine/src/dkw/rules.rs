use crate::game_state::{GameState, PlayerStatus};
use crate::types::{
    Move, PieceType, Player, Square, BOARD_SIZE, ELIMINATED_KING_SENTINEL, KING_OFFSETS,
};
/// Dead King Walking (DKW) implementation.
/// When a player resigns or times out:
/// - Their pieces become immovable, uncapturable walls
/// - Their king moves randomly to empty adjacent squares
/// - DKW king CAN be captured/checkmated
use rand::Rng;

/// Eliminate a player via DKW (resignation/timeout).
/// Pieces become walls, king remains live.
pub fn eliminate_player_dkw(state: &mut GameState, player: Player) {
    state.player_status[player.index()] = PlayerStatus::DKW;
    // King square remains — DKW king is still live
    // Castling rights are cleared
    let ks_bit = crate::movegen::castling_bit(player, true);
    let qs_bit = crate::movegen::castling_bit(player, false);
    state.castling_rights &= !(1 << ks_bit);
    state.castling_rights &= !(1 << qs_bit);
}

/// Eliminate a player fully (checkmate/stalemate).
/// Pieces stay on board as walls (like DKW), king is removed.
pub fn eliminate_player_full(state: &mut GameState, player: Player) {
    state.player_status[player.index()] = PlayerStatus::Eliminated;
    // Remove king from tracking
    state.board.king_sq[player.index()] = ELIMINATED_KING_SENTINEL;
    // Clear castling rights
    let ks_bit = crate::movegen::castling_bit(player, true);
    let qs_bit = crate::movegen::castling_bit(player, false);
    state.castling_rights &= !(1 << ks_bit);
    state.castling_rights &= !(1 << qs_bit);
}

/// Generate a random DKW king move.
/// Returns None if no moves available (king is surrounded).
pub fn generate_dkw_king_move(
    state: &GameState,
    player: Player,
    rng: &mut impl Rng,
) -> Option<Move> {
    let king_sq = state.board.king_square(player)?;
    let rank = king_sq.rank() as i8;
    let file = king_sq.file() as i8;

    // Collect all valid empty adjacent squares
    let mut candidates = arrayvec::ArrayVec::<Square, 8>::new();
    for (dr, df) in KING_OFFSETS {
        let nr = rank + dr;
        let nf = file + df;
        if nr < 0 || nr >= BOARD_SIZE as i8 || nf < 0 || nf >= BOARD_SIZE as i8 {
            continue;
        }
        if let Some(to) = Square::from_rank_file(nr as u8, nf as u8) {
            if state.board.get(to).is_none() {
                candidates.push(to);
            }
        }
    }

    if candidates.is_empty() {
        return None;
    }

    let idx = rng.gen_range(0..candidates.len());
    Some(Move::quiet(king_sq, candidates[idx], PieceType::King))
}

/// Process DKW moves for all DKW players.
/// Called BETWEEN turns, BEFORE elimination checks.
pub fn process_dkw_moves(state: &mut GameState, rng: &mut impl Rng) {
    for player in crate::types::ALL_PLAYERS {
        if state.is_dkw(player) {
            if let Some(mv) = generate_dkw_king_move(state, player, rng) {
                // Apply the DKW king move (simple — no captures, no special flags)
                state.board.move_piece(mv.from, mv.to);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_dkw_pieces_immovable() {
        let mut state = GameState::new();
        eliminate_player_dkw(&mut state, Player::Blue);

        assert!(state.is_dkw(Player::Blue));
        // Blue pieces should still be on the board
        assert!(state.board.piece_count[Player::Blue.index()] > 0);
    }

    #[test]
    fn test_dkw_king_random_move() {
        let mut state = GameState::new();
        eliminate_player_dkw(&mut state, Player::Blue);

        let mut rng = StdRng::seed_from_u64(42);
        let mv = generate_dkw_king_move(&state, Player::Blue, &mut rng);
        // Blue king at a7 is surrounded by pieces in starting position,
        // so it might not have moves. Let's clear a square first.
        // Actually in starting position, a7 king is surrounded by:
        // a6(Bishop), a8(Queen), b7(Pawn), b6(Pawn), b8(Pawn)
        // And diagonals that are occupied. So no empty adjacent squares.
        // The king has no moves from starting position as DKW.
        assert!(
            mv.is_none(),
            "DKW king at a7 should have no moves in starting position"
        );
    }

    #[test]
    fn test_dkw_king_has_moves() {
        let mut state = GameState::new();
        // Remove a piece adjacent to Blue king to make space
        let a6 = Square::from_rank_file(5, 0).unwrap(); // a6
        state.board.remove_piece(a6);
        eliminate_player_dkw(&mut state, Player::Blue);

        let mut rng = StdRng::seed_from_u64(42);
        let mv = generate_dkw_king_move(&state, Player::Blue, &mut rng);
        assert!(
            mv.is_some(),
            "DKW king should have a move after clearing a6"
        );
        let mv = mv.unwrap();
        assert_eq!(mv.piece, PieceType::King);
        assert!(
            state.board.get(mv.to).is_none(),
            "DKW king should only move to empty squares"
        );
    }

    #[test]
    fn test_dkw_seeded_deterministic() {
        let mut state = GameState::new();
        let a6 = Square::from_rank_file(5, 0).unwrap();
        state.board.remove_piece(a6);
        eliminate_player_dkw(&mut state, Player::Blue);

        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);
        let mv1 = generate_dkw_king_move(&state, Player::Blue, &mut rng1);
        let mv2 = generate_dkw_king_move(&state, Player::Blue, &mut rng2);
        assert_eq!(mv1, mv2, "Same seed should produce same DKW move");
    }

    #[test]
    fn test_eliminate_full() {
        let mut state = GameState::new();
        eliminate_player_full(&mut state, Player::Blue);

        assert!(state.is_eliminated(Player::Blue));
        assert!(state.board.is_eliminated(Player::Blue));
    }
}
