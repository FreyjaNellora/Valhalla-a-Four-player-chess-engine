use crate::game_state::GameState;
/// Attack Query API (ADR-001).
/// Nothing above Phase 1 should read board.squares[] directly.
/// Uses the "super-piece" approach: check backwards from target square.
use crate::types::{PieceType, Player, Square, ALL_PLAYERS, BOARD_SIZE, KNIGHT_OFFSETS};

/// Check if a square is attacked by any piece of the given player.
pub fn is_square_attacked_by(state: &GameState, sq: Square, attacker: Player) -> bool {
    let board = &state.board;
    let rank = sq.rank() as i8;
    let file = sq.file() as i8;

    // 1. Pawn attacks: check if attacker has a pawn that can capture onto sq.
    //    We reverse the attacker's capture directions to find where a pawn would be.
    let captures = attacker.capture_directions();
    for (dr, df) in captures {
        // A pawn at (rank - dr, file - df) would capture to (rank, file)
        let pr = rank - dr;
        let pf = file - df;
        if pr >= 0 && pr < BOARD_SIZE as i8 && pf >= 0 && pf < BOARD_SIZE as i8 {
            if let Some(from_sq) = Square::from_rank_file(pr as u8, pf as u8) {
                if let Some(piece) = board.get(from_sq) {
                    if piece.player == attacker && piece.piece_type == PieceType::Pawn {
                        return true;
                    }
                }
            }
        }
    }

    // 2. Knight attacks: check all 8 L-shape offsets
    for (dr, df) in KNIGHT_OFFSETS {
        let nr = rank + dr;
        let nf = file + df;
        if nr >= 0 && nr < BOARD_SIZE as i8 && nf >= 0 && nf < BOARD_SIZE as i8 {
            if let Some(from_sq) = Square::from_rank_file(nr as u8, nf as u8) {
                if let Some(piece) = board.get(from_sq) {
                    if piece.player == attacker && piece.piece_type == PieceType::Knight {
                        return true;
                    }
                }
            }
        }
    }

    // 3. King attacks: check all 8 adjacent squares
    for dr in -1..=1i8 {
        for df in -1..=1i8 {
            if dr == 0 && df == 0 {
                continue;
            }
            let kr = rank + dr;
            let kf = file + df;
            if kr >= 0 && kr < BOARD_SIZE as i8 && kf >= 0 && kf < BOARD_SIZE as i8 {
                if let Some(from_sq) = Square::from_rank_file(kr as u8, kf as u8) {
                    if let Some(piece) = board.get(from_sq) {
                        if piece.player == attacker && piece.piece_type == PieceType::King {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // 4. Slider attacks: ray-walk in all 8 directions.
    //    Check for bishop/queen on diagonals, rook/queen on orthogonals.
    //    PromotedQueen moves like queen.
    let diagonals: [(i8, i8); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
    let orthogonals: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    // Diagonal rays (bishop, queen, promoted queen)
    for (dr, df) in diagonals {
        let mut r = rank + dr;
        let mut f = file + df;
        while r >= 0 && r < BOARD_SIZE as i8 && f >= 0 && f < BOARD_SIZE as i8 {
            if let Some(ray_sq) = Square::from_rank_file(r as u8, f as u8) {
                if let Some(piece) = board.get(ray_sq) {
                    if piece.player == attacker {
                        match piece.piece_type {
                            PieceType::Bishop | PieceType::Queen | PieceType::PromotedQueen => {
                                return true
                            }
                            _ => break, // Blocked by own non-slider piece
                        }
                    } else {
                        break; // Blocked by enemy piece
                    }
                }
                // Empty valid square, continue ray
            } else {
                break; // Invalid corner, ray stops
            }
            r += dr;
            f += df;
        }
    }

    // Orthogonal rays (rook, queen, promoted queen)
    for (dr, df) in orthogonals {
        let mut r = rank + dr;
        let mut f = file + df;
        while r >= 0 && r < BOARD_SIZE as i8 && f >= 0 && f < BOARD_SIZE as i8 {
            if let Some(ray_sq) = Square::from_rank_file(r as u8, f as u8) {
                if let Some(piece) = board.get(ray_sq) {
                    if piece.player == attacker {
                        match piece.piece_type {
                            PieceType::Rook | PieceType::Queen | PieceType::PromotedQueen => {
                                return true
                            }
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
            r += dr;
            f += df;
        }
    }

    false
}

/// Check which players attack a given square. Returns [bool; 4].
pub fn attackers_of(state: &GameState, sq: Square) -> [bool; 4] {
    let mut result = [false; 4];
    for player in ALL_PLAYERS {
        result[player.index()] = is_square_attacked_by(state, sq, player);
    }
    result
}

/// Check if a player's king is in check (attacked by ANY of the 3 opponents).
/// Three-way check obligation: all THREE opponents must be considered.
pub fn is_in_check(state: &GameState, player: Player) -> bool {
    let king_sq = match state.board.king_square(player) {
        Some(sq) => sq,
        None => return false, // Eliminated player has no king
    };

    for opponent in ALL_PLAYERS {
        if opponent == player {
            continue;
        }
        // Only active and DKW players can give check
        // (DKW king can still be in the way, and DKW pieces block but don't attack —
        //  actually DKW pieces are walls, they don't attack. Only active players attack.)
        if !state.is_active(opponent) {
            continue;
        }
        if is_square_attacked_by(state, king_sq, opponent) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::types::ColoredPiece;

    fn empty_state() -> GameState {
        let mut state = GameState::new();
        state.board = Board::new();
        // Reset king squares since board is empty
        for i in 0..4 {
            state.board.king_sq[i] = crate::types::ELIMINATED_KING_SENTINEL;
        }
        state
    }

    #[test]
    fn test_starting_position_no_check() {
        let state = GameState::new();
        for player in ALL_PLAYERS {
            assert!(
                !is_in_check(&state, player),
                "Player {:?} should not be in check at start",
                player
            );
        }
    }

    #[test]
    fn test_rook_attack_horizontal() {
        let mut state = empty_state();
        let rook_sq = Square::from_algebraic("d4").unwrap();
        state
            .board
            .place_piece(rook_sq, ColoredPiece::new(Player::Red, PieceType::Rook));

        // Rook attacks along rank 4
        let target = Square::from_algebraic("h4").unwrap();
        assert!(is_square_attacked_by(&state, target, Player::Red));
    }

    #[test]
    fn test_rook_attack_blocked() {
        let mut state = empty_state();
        let rook_sq = Square::from_algebraic("d4").unwrap();
        let blocker = Square::from_algebraic("f4").unwrap();
        state
            .board
            .place_piece(rook_sq, ColoredPiece::new(Player::Red, PieceType::Rook));
        state
            .board
            .place_piece(blocker, ColoredPiece::new(Player::Blue, PieceType::Pawn));

        // Rook is blocked by blocker
        let beyond = Square::from_algebraic("h4").unwrap();
        assert!(!is_square_attacked_by(&state, beyond, Player::Red));
        // But attacks the blocker itself
        assert!(is_square_attacked_by(&state, blocker, Player::Red));
    }

    #[test]
    fn test_bishop_attack_diagonal() {
        let mut state = empty_state();
        let sq = Square::from_algebraic("e5").unwrap();
        state
            .board
            .place_piece(sq, ColoredPiece::new(Player::Red, PieceType::Bishop));

        let target = Square::from_algebraic("h8").unwrap();
        assert!(is_square_attacked_by(&state, target, Player::Red));
    }

    #[test]
    fn test_knight_attack() {
        let mut state = empty_state();
        let sq = Square::from_algebraic("e5").unwrap();
        state
            .board
            .place_piece(sq, ColoredPiece::new(Player::Red, PieceType::Knight));

        // Knight L-shapes from e5 (rank 4, file 4)
        let targets = ["d7", "f7", "g6", "g4", "f3", "d3"];
        for t in targets {
            if let Some(target) = Square::from_algebraic(t) {
                assert!(
                    is_square_attacked_by(&state, target, Player::Red),
                    "Knight should attack {}",
                    t
                );
            }
        }
        // Knight does not attack adjacent
        let not_target = Square::from_algebraic("e6").unwrap();
        assert!(!is_square_attacked_by(&state, not_target, Player::Red));
    }

    #[test]
    fn test_red_pawn_attack() {
        let mut state = empty_state();
        let sq = Square::from_algebraic("e4").unwrap();
        state
            .board
            .place_piece(sq, ColoredPiece::new(Player::Red, PieceType::Pawn));

        // Red pawn captures NE and NW: (rank+1, file+1) and (rank+1, file-1)
        let ne = Square::from_algebraic("f5").unwrap();
        let nw = Square::from_algebraic("d5").unwrap();
        assert!(is_square_attacked_by(&state, ne, Player::Red));
        assert!(is_square_attacked_by(&state, nw, Player::Red));
        // Does not attack forward
        let fwd = Square::from_algebraic("e5").unwrap();
        assert!(!is_square_attacked_by(&state, fwd, Player::Red));
    }

    #[test]
    fn test_blue_pawn_attack() {
        let mut state = empty_state();
        let sq = Square::from_algebraic("d5").unwrap();
        state
            .board
            .place_piece(sq, ColoredPiece::new(Player::Blue, PieceType::Pawn));

        // Blue pawn captures NE (+1,+1) and SE (-1,+1)
        let ne = Square::from_algebraic("e6").unwrap();
        let se = Square::from_algebraic("e4").unwrap();
        assert!(is_square_attacked_by(&state, ne, Player::Blue));
        assert!(is_square_attacked_by(&state, se, Player::Blue));
    }

    #[test]
    fn test_yellow_pawn_attack() {
        let mut state = empty_state();
        let sq = Square::from_algebraic("e10").unwrap();
        state
            .board
            .place_piece(sq, ColoredPiece::new(Player::Yellow, PieceType::Pawn));

        // Yellow pawn captures SE (-1,+1) and SW (-1,-1)
        let se = Square::from_algebraic("f9").unwrap();
        let sw = Square::from_algebraic("d9").unwrap();
        assert!(is_square_attacked_by(&state, se, Player::Yellow));
        assert!(is_square_attacked_by(&state, sw, Player::Yellow));
    }

    #[test]
    fn test_green_pawn_attack() {
        let mut state = empty_state();
        let sq = Square::from_algebraic("j5").unwrap();
        state
            .board
            .place_piece(sq, ColoredPiece::new(Player::Green, PieceType::Pawn));

        // Green pawn captures NW (+1,-1) and SW (-1,-1)
        let nw = Square::from_algebraic("i6").unwrap();
        let sw = Square::from_algebraic("i4").unwrap();
        assert!(is_square_attacked_by(&state, nw, Player::Green));
        assert!(is_square_attacked_by(&state, sw, Player::Green));
    }

    #[test]
    fn test_three_opponent_check() {
        let mut state = empty_state();
        // Place Red king
        let king_sq = Square::from_algebraic("h5").unwrap();
        state
            .board
            .place_piece(king_sq, ColoredPiece::new(Player::Red, PieceType::King));

        // Blue attacks from the left with a rook
        let blue_rook = Square::from_algebraic("d5").unwrap();
        state
            .board
            .place_piece(blue_rook, ColoredPiece::new(Player::Blue, PieceType::Rook));

        assert!(is_in_check(&state, Player::Red));

        // Remove blue rook, add yellow bishop attacking diagonally
        state.board.remove_piece(blue_rook);
        let yellow_bishop = Square::from_algebraic("f7").unwrap();
        state.board.place_piece(
            yellow_bishop,
            ColoredPiece::new(Player::Yellow, PieceType::Bishop),
        );

        assert!(is_in_check(&state, Player::Red));
    }

    #[test]
    fn test_king_attack() {
        let mut state = empty_state();
        let sq = Square::from_algebraic("e5").unwrap();
        state
            .board
            .place_piece(sq, ColoredPiece::new(Player::Red, PieceType::King));

        let adj = Square::from_algebraic("e6").unwrap();
        assert!(is_square_attacked_by(&state, adj, Player::Red));
        let far = Square::from_algebraic("e7").unwrap();
        assert!(!is_square_attacked_by(&state, far, Player::Red));
    }

    #[test]
    fn test_slider_blocked_by_corner() {
        let mut state = empty_state();
        // Place a bishop on d4 (rank 3, file 3)
        let sq = Square::from_algebraic("d4").unwrap();
        state
            .board
            .place_piece(sq, ColoredPiece::new(Player::Red, PieceType::Bishop));

        // Diagonal NE from d4: e5 is valid and attacked
        let e5 = Square::from_algebraic("e5").unwrap();
        assert!(is_square_attacked_by(&state, e5, Player::Red));

        // Diagonal SW from d4: c3 is invalid (SW corner: rank 2, file 2).
        // The ray stops immediately at the invalid square.
        // So the bishop only attacks in 3 diagonal directions from d4
        // (NE, NW via d4 has no room NW either — actually NW from d4 is c5 which IS valid)
        let c5 = Square::from_algebraic("c5").unwrap();
        assert!(
            is_square_attacked_by(&state, c5, Player::Red),
            "Bishop should attack c5 (NW)"
        );
    }
}
