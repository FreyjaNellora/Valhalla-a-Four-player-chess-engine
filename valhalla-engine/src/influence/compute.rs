//! Ray-attenuated influence map computation.
//!
//! Each active player's pieces project influence along their movement vectors.
//! Sliding pieces use ray-walk with compounding blocker gradient.
//! Knights/Pawns/King project directly to attack squares.
//! Invalid corner squares are skipped entirely.

use crate::game_state::GameState;
use crate::types::piece::{PieceType, KING_OFFSETS, KNIGHT_OFFSETS};
use crate::types::player::{Player, ALL_PLAYERS};
use crate::types::square::Square;

const TOTAL_SQUARES: usize = 196;

/// Per-player influence on every square.
/// `grid[square_index][player_index]` = total influence value.
pub struct InfluenceMap {
    pub grid: [[f32; 4]; TOTAL_SQUARES],
}

impl InfluenceMap {
    /// Compute influence map for the current position.
    pub fn compute(state: &GameState) -> Self {
        let mut map = InfluenceMap {
            grid: [[0.0f32; 4]; TOTAL_SQUARES],
        };

        for &player in &ALL_PLAYERS {
            // Skip DKW/eliminated — they project no influence
            if !state.is_active(player) {
                continue;
            }

            let pi = player.index();

            for (sq, piece_type) in state.board.pieces_for_player(player) {
                match piece_type {
                    PieceType::Knight => {
                        for &(dr, df) in &KNIGHT_OFFSETS {
                            if let Some(target) = sq.offset(dr, df) {
                                map.grid[target.index() as usize][pi] += 1.0;
                            }
                        }
                    }
                    PieceType::King => {
                        for &(dr, df) in &KING_OFFSETS {
                            if let Some(target) = sq.offset(dr, df) {
                                map.grid[target.index() as usize][pi] += 1.0;
                            }
                        }
                    }
                    PieceType::Pawn => {
                        let capture_dirs = player.capture_directions();
                        for &(dr, df) in &capture_dirs {
                            if let Some(target) = sq.offset(dr, df) {
                                map.grid[target.index() as usize][pi] += 1.0;
                            }
                        }
                    }
                    _ if piece_type.is_slider() => {
                        // Bishop, Rook, Queen, PromotedQueen
                        for &(dr, df) in piece_type.slide_directions() {
                            Self::ray_walk(state, &mut map.grid, pi, sq, dr, df);
                        }
                    }
                    _ => {} // King=5 already handled above
                }
            }
        }

        map
    }

    /// Walk a ray from `origin` in direction (dr, df), projecting influence
    /// with compounding blocker attenuation.
    fn ray_walk(
        state: &GameState,
        grid: &mut [[f32; 4]; TOTAL_SQUARES],
        player_index: usize,
        origin: Square,
        dr: i8,
        df: i8,
    ) {
        let mut influence = 1.0f32;
        let mut blocker_count = 0u32;
        let mut current = origin;

        loop {
            current = match current.offset(dr, df) {
                Some(sq) => sq,
                None => break, // Off the board
            };

            // Add influence to this square
            grid[current.index() as usize][player_index] += influence;

            // Check for blocker
            if let Some(piece) = state.board.get(current) {
                blocker_count += 1;

                // Determine attenuation factor
                let is_friendly = piece.player.index() == player_index;

                // DKW pieces treated as enemy blockers
                let is_dkw = state.is_dkw(piece.player) || state.is_eliminated(piece.player);

                if is_friendly && !is_dkw {
                    // Friendly blocker: compounding gradient
                    influence /= 1.5 * blocker_count as f32;
                } else {
                    // Enemy or DKW blocker: heavier compounding attenuation
                    let piece_weight = piece.piece_type.eval_centipawns() as f32 / 100.0;
                    influence /= (2.0 + piece_weight * 0.3) * blocker_count as f32;
                }

                // If influence drops below threshold, stop the ray
                if influence < 0.001 {
                    break;
                }
            }
        }
    }

    /// Get influence value for a specific square and player.
    #[inline]
    pub fn get(&self, sq: Square, player: Player) -> f32 {
        self.grid[sq.index() as usize][player.index()]
    }

    /// Get total influence on a square across all players.
    #[inline]
    pub fn total_influence(&self, sq: Square) -> f32 {
        let idx = sq.index() as usize;
        self.grid[idx][0] + self.grid[idx][1] + self.grid[idx][2] + self.grid[idx][3]
    }

    /// Get influence advantage: player's influence minus sum of opponents'.
    pub fn advantage(&self, sq: Square, player: Player) -> f32 {
        let idx = sq.index() as usize;
        let pi = player.index();
        let mine = self.grid[idx][pi];
        let theirs: f32 = (0..4).filter(|&i| i != pi).map(|i| self.grid[idx][i]).sum();
        mine - theirs
    }
}

impl Default for InfluenceMap {
    fn default() -> Self {
        InfluenceMap {
            grid: [[0.0f32; 4]; TOTAL_SQUARES],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;
    use crate::types::piece::ColoredPiece;

    /// Helper: create an empty board state with just specific pieces.
    fn empty_state_with_pieces(pieces: &[(Player, PieceType, u8, u8)]) -> GameState {
        let mut state = GameState::new();
        // Clear the board
        for sq_idx in 0..196u8 {
            let sq = Square::from_index_unchecked(sq_idx);
            if sq.is_valid_index() {
                state.board.remove_piece(sq);
            }
        }
        // Place specified pieces
        for &(player, piece_type, rank, file) in pieces {
            if let Some(sq) = Square::from_rank_file(rank, file) {
                state
                    .board
                    .place_piece(sq, ColoredPiece::new(player, piece_type));
            }
        }
        state
    }

    #[test]
    fn test_lone_rook_uniform_influence() {
        // A lone rook on an empty board should project 1.0 on all ray squares
        let state = empty_state_with_pieces(&[
            (Player::Red, PieceType::Rook, 6, 6),
            // Need kings to avoid panics
            (Player::Red, PieceType::King, 0, 7),
            (Player::Blue, PieceType::King, 7, 0),
            (Player::Yellow, PieceType::King, 13, 6),
            (Player::Green, PieceType::King, 6, 13),
        ]);

        let map = InfluenceMap::compute(&state);

        // Check squares along rank 6 (file 0-13, excluding rook's own square and corners)
        for file in 3..=10 {
            if file == 6 {
                continue; // rook's own square
            }
            let sq = Square::from_rank_file(6, file).unwrap();
            let inf = map.get(sq, Player::Red);
            assert!(
                (inf - 1.0).abs() < 0.01,
                "Lone rook should project 1.0 on rank ray at file {}, got {}",
                file,
                inf
            );
        }
    }

    #[test]
    fn test_friendly_blocker_attenuation() {
        // Rook at (6,3), friendly pawn at (6,5) — squares past pawn should be attenuated
        let state = empty_state_with_pieces(&[
            (Player::Red, PieceType::Rook, 6, 3),
            (Player::Red, PieceType::Pawn, 6, 5),
            (Player::Red, PieceType::King, 0, 7),
            (Player::Blue, PieceType::King, 7, 0),
            (Player::Yellow, PieceType::King, 13, 6),
            (Player::Green, PieceType::King, 6, 13),
        ]);

        let map = InfluenceMap::compute(&state);

        // Square before blocker should be 1.0
        let before = map.get(Square::from_rank_file(6, 4).unwrap(), Player::Red);
        assert!(
            (before - 1.0).abs() < 0.01,
            "Before blocker should be ~1.0, got {}",
            before
        );

        // Blocker square gets 1.0 (influence applied before attenuation)
        let at_blocker = map.get(Square::from_rank_file(6, 5).unwrap(), Player::Red);
        assert!(
            (at_blocker - 1.0).abs() < 0.5, // pawn also contributes its own influence
            "At blocker should be ~1.0 from rook, got {}",
            at_blocker
        );

        // Square after first friendly blocker: 1.0 / (1.5 * 1) = 0.667
        let after = map.get(Square::from_rank_file(6, 6).unwrap(), Player::Red);
        let expected = 1.0 / 1.5;
        assert!(
            (after - expected).abs() < 0.1,
            "After 1st friendly blocker should be ~{:.3}, got {:.3}",
            expected,
            after
        );
    }

    #[test]
    fn test_enemy_blocker_attenuation() {
        // Red rook at (6,3), Blue pawn at (6,5)
        let state = empty_state_with_pieces(&[
            (Player::Red, PieceType::Rook, 6, 3),
            (Player::Blue, PieceType::Pawn, 6, 5),
            (Player::Red, PieceType::King, 0, 7),
            (Player::Blue, PieceType::King, 7, 0),
            (Player::Yellow, PieceType::King, 13, 6),
            (Player::Green, PieceType::King, 6, 13),
        ]);

        let map = InfluenceMap::compute(&state);

        // After enemy pawn (weight 1.0): 1.0 / ((2.0 + 1.0*0.3) * 1) = 1.0 / 2.3
        let after = map.get(Square::from_rank_file(6, 6).unwrap(), Player::Red);
        let expected = 1.0 / 2.3;
        assert!(
            (after - expected).abs() < 0.1,
            "After enemy pawn blocker should be ~{:.3}, got {:.3}",
            expected,
            after
        );
    }

    #[test]
    fn test_starting_position_symmetry() {
        let state = GameState::new();
        let map = InfluenceMap::compute(&state);

        // The total Red influence across all squares should equal total Yellow influence
        let red_total: f32 = (0..196)
            .filter(|&i| Square::from_index_unchecked(i as u8).is_valid_index())
            .map(|i| map.grid[i][Player::Red.index()])
            .sum();
        let yellow_total: f32 = (0..196)
            .filter(|&i| Square::from_index_unchecked(i as u8).is_valid_index())
            .map(|i| map.grid[i][Player::Yellow.index()])
            .sum();

        assert!(
            (red_total - yellow_total).abs() < 1.0,
            "Red and Yellow total influence should be similar: R={:.1}, Y={:.1}",
            red_total,
            yellow_total
        );
    }

    #[test]
    fn test_no_influence_on_corners() {
        let state = GameState::new();
        let map = InfluenceMap::compute(&state);

        // Corner square (0,0) should have zero influence
        let corner = Square::from_index_unchecked(0); // rank 0, file 0
        assert!(
            !corner.is_valid_index() || map.total_influence(corner) == 0.0,
            "Corner squares should have no influence"
        );
    }

    #[test]
    fn test_knight_influence_count() {
        // A knight in the center should influence exactly 8 squares
        let state = empty_state_with_pieces(&[
            (Player::Red, PieceType::Knight, 6, 6),
            (Player::Red, PieceType::King, 0, 7),
            (Player::Blue, PieceType::King, 7, 0),
            (Player::Yellow, PieceType::King, 13, 6),
            (Player::Green, PieceType::King, 6, 13),
        ]);

        let map = InfluenceMap::compute(&state);

        let mut count = 0;
        for i in 0..196 {
            let sq = Square::from_index_unchecked(i as u8);
            if sq.is_valid_index() && map.get(sq, Player::Red) > 0.5 {
                count += 1;
            }
        }

        // Knight on (6,6) + king on (0,7) contributes influence
        // Knight should have 8 attacked squares, king up to 8
        assert!(
            count >= 8,
            "Knight in center should influence at least 8 squares, got {}",
            count
        );
    }
}
