use super::GameState;
use crate::movegen::{castling_bit, get_castle_rook_squares, ROOK_START_SQUARES};
/// Make/unmake move implementation.
/// make_move applies a move and returns MoveUndo for reversal.
/// unmake_move restores the exact prior state.
use crate::types::{ColoredPiece, Move, MoveUndo, PieceType, Player, Square};
use crate::zobrist;

impl GameState {
    /// Apply a move and return undo information.
    /// Updates board, Zobrist hash, castling rights, EP, turn, clocks, scores.
    pub fn make_move(&mut self, mv: Move) -> MoveUndo {
        let player = self.side_to_move;

        // Save undo state
        let undo = MoveUndo {
            captured_piece: if mv.is_en_passant() {
                // For EP, the captured piece is not on mv.to
                self.get_ep_captured_piece()
            } else {
                self.board.get(mv.to)
            },
            castling_rights: self.castling_rights,
            en_passant: self.en_passant,
            ep_pushing_player: self.ep_pushing_player,
            half_move_clock: self.half_move_clock,
            zobrist_hash: self.zobrist_hash,
            ffa_scores: self.ffa_scores,
        };

        // Clear old EP from hash
        if let Some(old_ep) = self.en_passant {
            zobrist::toggle_ep(&mut self.zobrist_hash, Some(old_ep), None);
        }

        // Remove old side-to-move from hash
        zobrist::toggle_side(&mut self.zobrist_hash, player, player);

        // Handle the move
        if mv.is_en_passant() {
            self.do_en_passant(mv, player);
        } else if mv.is_castle() {
            self.do_castle(mv, player);
        } else if mv.is_promotion() {
            self.do_promotion(mv, player);
        } else {
            self.do_normal_move(mv, player);
        }

        // Update en passant
        if mv.is_double_push() {
            let (push_dr, push_df) = player.push_direction();
            // EP target is the intermediate square
            let ep_r = mv.from.rank() as i8 + push_dr;
            let ep_f = mv.from.file() as i8 + push_df;
            let ep_sq = Square::from_rank_file(ep_r as u8, ep_f as u8).unwrap();
            self.en_passant = Some(ep_sq);
            self.ep_pushing_player = Some(player);
            zobrist::toggle_ep(&mut self.zobrist_hash, None, Some(ep_sq));
        } else {
            self.en_passant = None;
            self.ep_pushing_player = None;
        }

        // Update castling rights
        self.update_castling_rights(mv, player);

        // Update half-move clock
        if mv.piece == PieceType::Pawn || mv.is_capture() {
            self.half_move_clock = 0;
        } else {
            self.half_move_clock += 1;
        }

        // Advance turn
        let old_side = self.side_to_move;
        self.advance_turn();
        zobrist::toggle_side(&mut self.zobrist_hash, old_side, self.side_to_move);

        // Increment ply
        self.ply += 1;
        if self.side_to_move == Player::Red {
            self.full_move_number += 1;
        }

        undo
    }

    /// Unmake a move, restoring the exact prior state.
    pub fn unmake_move(&mut self, mv: Move, undo: MoveUndo) {
        // Restore ply
        self.ply -= 1;
        if self.side_to_move == Player::Red {
            self.full_move_number -= 1;
        }

        // Restore side to move (reverse advance_turn)
        // We know which player made the move from the piece's owner
        self.side_to_move = self.find_previous_active_player();

        let player = self.side_to_move;

        // Undo the board changes
        if mv.is_en_passant() {
            self.undo_en_passant(mv, player, &undo);
        } else if mv.is_castle() {
            self.undo_castle(mv, player);
        } else if mv.is_promotion() {
            self.undo_promotion(mv, player, &undo);
        } else {
            self.undo_normal_move(mv, &undo);
        }

        // Restore saved state
        self.castling_rights = undo.castling_rights;
        self.en_passant = undo.en_passant;
        self.ep_pushing_player = undo.ep_pushing_player;
        self.half_move_clock = undo.half_move_clock;
        self.zobrist_hash = undo.zobrist_hash;
        self.ffa_scores = undo.ffa_scores;
    }

    // --- Internal helpers ---

    fn do_normal_move(&mut self, mv: Move, player: Player) {
        // Remove captured piece
        if mv.is_capture() {
            if let Some(cap) = self.board.get(mv.to) {
                zobrist::toggle_piece(&mut self.zobrist_hash, cap.player, cap.piece_type, mv.to);
                self.board.remove_piece(mv.to);
                // FFA score
                if self.game_mode == super::GameMode::FFA {
                    self.ffa_scores[player.index()] += cap.piece_type.ffa_points();
                }
            }
        }

        // Move the piece
        zobrist::toggle_piece(&mut self.zobrist_hash, player, mv.piece, mv.from);
        zobrist::toggle_piece(&mut self.zobrist_hash, player, mv.piece, mv.to);
        self.board.move_piece(mv.from, mv.to);
    }

    fn do_en_passant(&mut self, mv: Move, player: Player) {
        // Find and remove the captured pawn (board scan)
        if let Some(ep_player) = self.ep_pushing_player {
            let (push_dr, push_df) = ep_player.push_direction();
            let cap_r = mv.to.rank() as i8 - push_dr;
            let cap_f = mv.to.file() as i8 - push_df;
            if let Some(cap_sq) = Square::from_rank_file(cap_r as u8, cap_f as u8) {
                if let Some(cap_piece) = self.board.get(cap_sq) {
                    zobrist::toggle_piece(
                        &mut self.zobrist_hash,
                        cap_piece.player,
                        cap_piece.piece_type,
                        cap_sq,
                    );
                    self.board.remove_piece(cap_sq);
                    if self.game_mode == super::GameMode::FFA {
                        self.ffa_scores[player.index()] += PieceType::Pawn.ffa_points();
                    }
                }
            }
        }

        // Move the pawn
        zobrist::toggle_piece(&mut self.zobrist_hash, player, PieceType::Pawn, mv.from);
        zobrist::toggle_piece(&mut self.zobrist_hash, player, PieceType::Pawn, mv.to);
        self.board.move_piece(mv.from, mv.to);
    }

    fn do_castle(&mut self, mv: Move, player: Player) {
        let kingside = mv.is_castle_kingside();
        let (rook_from, rook_to) = get_castle_rook_squares(player, kingside);

        // Move king
        zobrist::toggle_piece(&mut self.zobrist_hash, player, PieceType::King, mv.from);
        zobrist::toggle_piece(&mut self.zobrist_hash, player, PieceType::King, mv.to);
        self.board.move_piece(mv.from, mv.to);

        // Move rook
        zobrist::toggle_piece(&mut self.zobrist_hash, player, PieceType::Rook, rook_from);
        zobrist::toggle_piece(&mut self.zobrist_hash, player, PieceType::Rook, rook_to);
        self.board.move_piece(rook_from, rook_to);
    }

    fn do_promotion(&mut self, mv: Move, player: Player) {
        let promo_type = mv.promotion.unwrap();

        // Handle capture
        if mv.is_capture() {
            if let Some(cap) = self.board.get(mv.to) {
                zobrist::toggle_piece(&mut self.zobrist_hash, cap.player, cap.piece_type, mv.to);
                self.board.remove_piece(mv.to);
                if self.game_mode == super::GameMode::FFA {
                    self.ffa_scores[player.index()] += cap.piece_type.ffa_points();
                }
            }
        }

        // Remove pawn
        zobrist::toggle_piece(&mut self.zobrist_hash, player, PieceType::Pawn, mv.from);
        self.board.remove_piece(mv.from);

        // Place promoted piece
        zobrist::toggle_piece(&mut self.zobrist_hash, player, promo_type, mv.to);
        self.board
            .place_piece(mv.to, ColoredPiece::new(player, promo_type));
    }

    fn undo_normal_move(&mut self, mv: Move, undo: &MoveUndo) {
        // Move piece back
        self.board.move_piece(mv.to, mv.from);

        // Restore captured piece
        if let Some(cap) = undo.captured_piece {
            self.board.place_piece(mv.to, cap);
        }
    }

    fn undo_en_passant(&mut self, mv: Move, _player: Player, undo: &MoveUndo) {
        // Move pawn back
        self.board.move_piece(mv.to, mv.from);

        // Restore captured pawn
        if let Some(cap) = undo.captured_piece {
            // Figure out where the captured pawn was
            if let Some(ep_player) = undo.ep_pushing_player {
                let (push_dr, push_df) = ep_player.push_direction();
                let cap_r = mv.to.rank() as i8 - push_dr;
                let cap_f = mv.to.file() as i8 - push_df;
                if let Some(cap_sq) = Square::from_rank_file(cap_r as u8, cap_f as u8) {
                    self.board.place_piece(cap_sq, cap);
                }
            }
        }
    }

    fn undo_castle(&mut self, mv: Move, player: Player) {
        let kingside = mv.is_castle_kingside();
        let (rook_from, rook_to) = get_castle_rook_squares(player, kingside);

        // Move king back
        self.board.move_piece(mv.to, mv.from);
        // Move rook back
        self.board.move_piece(rook_to, rook_from);
    }

    fn undo_promotion(&mut self, mv: Move, player: Player, undo: &MoveUndo) {
        // Remove promoted piece
        self.board.remove_piece(mv.to);

        // Restore pawn
        self.board
            .place_piece(mv.from, ColoredPiece::new(player, PieceType::Pawn));

        // Restore captured piece
        if let Some(cap) = undo.captured_piece {
            self.board.place_piece(mv.to, cap);
        }
    }

    fn update_castling_rights(&mut self, mv: Move, player: Player) {
        let old_rights = self.castling_rights;

        // King move clears both castling rights for that player
        if mv.piece == PieceType::King {
            let ks_bit = castling_bit(player, true);
            let qs_bit = castling_bit(player, false);
            self.castling_rights &= !(1 << ks_bit);
            self.castling_rights &= !(1 << qs_bit);
        }

        // Rook move or capture clears the relevant castling right
        for &(sq_idx, player_idx, is_ks) in &ROOK_START_SQUARES {
            let bit = castling_bit(Player::from_index(player_idx as u8).unwrap(), is_ks);
            // Rook moved from its starting square
            if mv.from.index() == sq_idx {
                self.castling_rights &= !(1 << bit);
            }
            // Rook captured on its starting square
            if mv.is_capture() && mv.to.index() == sq_idx {
                self.castling_rights &= !(1 << bit);
            }
        }

        // Update Zobrist for changed castling bits
        let changed = old_rights ^ self.castling_rights;
        for bit in 0..8u8 {
            if changed & (1 << bit) != 0 {
                zobrist::toggle_castling_bit(&mut self.zobrist_hash, bit);
            }
        }
    }

    /// Get the EP captured piece for undo (the pawn that double-pushed).
    fn get_ep_captured_piece(&self) -> Option<ColoredPiece> {
        let ep_sq = self.en_passant?;
        let ep_player = self.ep_pushing_player?;
        let (push_dr, push_df) = ep_player.push_direction();
        let cap_r = ep_sq.rank() as i8 - push_dr;
        let cap_f = ep_sq.file() as i8 - push_df;
        let cap_sq = Square::from_rank_file(cap_r as u8, cap_f as u8)?;
        self.board.get(cap_sq)
    }

    /// Find the previous active player (reverse of advance_turn).
    fn find_previous_active_player(&self) -> Player {
        let mut p = self.side_to_move.prev();
        for _ in 0..4 {
            if self.player_status[p.index()] == super::PlayerStatus::Active {
                return p;
            }
            p = p.prev();
        }
        self.side_to_move.prev() // Fallback (shouldn't happen in normal play)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::generate_legal_moves;
    use crate::types::MAX_MOVES;
    use arrayvec::ArrayVec;

    #[test]
    fn test_make_unmake_roundtrip_starting() {
        let state = GameState::new();
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(&state, &mut moves);

        for mv in &moves {
            let mut test = state.clone();
            let original_hash = test.zobrist_hash;
            let undo = test.make_move(*mv);
            test.unmake_move(*mv, undo);

            assert_eq!(
                test.zobrist_hash,
                original_hash,
                "Hash mismatch after make/unmake of {} -> {}",
                mv.from.to_algebraic(),
                mv.to.to_algebraic()
            );
            assert_eq!(test.castling_rights, state.castling_rights);
            assert_eq!(test.en_passant, state.en_passant);
            assert_eq!(test.side_to_move, state.side_to_move);
            assert_eq!(test.half_move_clock, state.half_move_clock);

            // Verify board is identical
            for i in 0..crate::types::TOTAL_SQUARES {
                assert_eq!(
                    test.board.squares[i],
                    state.board.squares[i],
                    "Board mismatch at square {} after make/unmake of {} -> {}",
                    i,
                    mv.from.to_algebraic(),
                    mv.to.to_algebraic()
                );
            }
        }
    }

    #[test]
    fn test_make_move_advances_turn() {
        let mut state = GameState::new();
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(&state, &mut moves);

        let mv = moves[0];
        let _undo = state.make_move(mv);
        assert_eq!(state.side_to_move, Player::Blue);
    }

    #[test]
    fn test_make_move_hash_matches_full() {
        let mut state = GameState::new();
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(&state, &mut moves);

        let mv = moves[0];
        let _undo = state.make_move(mv);

        let expected = zobrist::compute_full_hash(
            &state.board,
            state.side_to_move,
            state.castling_rights,
            state.en_passant,
        );
        assert_eq!(
            state.zobrist_hash, expected,
            "Incremental hash after make_move should match full recomputation"
        );
    }

    #[test]
    fn test_double_push_sets_ep() {
        let mut state = GameState::new();
        // Find a double push move (e.g., d2->d4)
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(&state, &mut moves);

        let dp = moves.iter().find(|m| m.is_double_push()).unwrap();
        let _undo = state.make_move(*dp);

        assert!(
            state.en_passant.is_some(),
            "EP square should be set after double push"
        );
    }

    #[test]
    fn test_ep_clears_after_next_move() {
        let mut state = GameState::new();
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(&state, &mut moves);

        // Make a double push
        let dp = *moves.iter().find(|m| m.is_double_push()).unwrap();
        let _undo1 = state.make_move(dp);
        assert!(state.en_passant.is_some());

        // Make any Blue move
        let mut blue_moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(&state, &mut blue_moves);
        let _undo2 = state.make_move(blue_moves[0]);

        // EP should be cleared (1-ply rule)
        assert!(
            state.en_passant.is_none(),
            "EP should clear after next move"
        );
    }
}
