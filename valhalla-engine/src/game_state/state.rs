use crate::board::{starting_position, Board};
/// GameState — the complete state of a four-player chess game.
/// Fixed-size, no heap allocation.
use crate::types::{Player, Square, PLAYERS};
use crate::zobrist;

/// Player status for tracking elimination and DKW.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerStatus {
    /// Player is active and takes turns.
    Active,
    /// Player is eliminated (checkmated or stalemated). Turn skipped.
    Eliminated,
    /// Dead King Walking — pieces are walls, king moves randomly. Turn skipped.
    DKW,
}

/// Game mode — affects promotion ranks and win conditions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode {
    /// Free-For-All: points-based scoring.
    FFA,
    /// Last King Standing: survival only, no points.
    LKS,
}

/// The complete game state. Fixed-size, no heap.
#[derive(Clone)]
pub struct GameState {
    /// The board (mailbox + piece lists).
    pub board: Board,
    /// Whose turn it is.
    pub side_to_move: Player,
    /// Castling rights (8 bits: 2 per player).
    pub castling_rights: u8,
    /// En passant target square (the square the capturing pawn moves to).
    pub en_passant: Option<Square>,
    /// Which player created the en passant opportunity.
    pub ep_pushing_player: Option<Player>,
    /// Half-move clock for 50-move rule.
    pub half_move_clock: u16,
    /// Full move number (increments after Green's turn).
    pub full_move_number: u16,
    /// Total half-moves (plies) played.
    pub ply: u16,
    /// Zobrist hash of the current position.
    pub zobrist_hash: u64,
    /// Status of each player.
    pub player_status: [PlayerStatus; PLAYERS],
    /// FFA scores per player (game scoring, NOT eval centipawns).
    pub ffa_scores: [i16; PLAYERS],
    /// Current game mode.
    pub game_mode: GameMode,
}

impl GameState {
    /// Create a new game state with the standard starting position.
    pub fn new() -> Self {
        let board = starting_position();
        let castling_rights = 0xFF; // All 8 castling rights
        let side_to_move = Player::Red;
        let hash = zobrist::compute_full_hash(&board, side_to_move, castling_rights, None);

        Self {
            board,
            side_to_move,
            castling_rights,
            en_passant: None,
            ep_pushing_player: None,
            half_move_clock: 0,
            full_move_number: 1,
            ply: 0,
            zobrist_hash: hash,
            player_status: [PlayerStatus::Active; PLAYERS],
            ffa_scores: [0; PLAYERS],
            game_mode: GameMode::FFA,
        }
    }

    /// Get the current side to move.
    #[inline]
    pub fn side_to_move(&self) -> Player {
        self.side_to_move
    }

    /// Advance to the next active player (skip eliminated and DKW players).
    pub fn advance_turn(&mut self) {
        let starting = self.side_to_move;
        loop {
            self.side_to_move = self.side_to_move.next();
            if self.player_status[self.side_to_move.index()] == PlayerStatus::Active {
                break;
            }
            // Safety: if all players are eliminated, this would loop forever.
            // In practice, the game ends before that.
            if self.side_to_move == starting {
                break; // No active players — game over
            }
        }
    }

    /// Count active players.
    pub fn active_player_count(&self) -> u8 {
        self.player_status
            .iter()
            .filter(|&&s| s == PlayerStatus::Active)
            .count() as u8
    }

    /// Check if the game is over (fewer than 2 active players).
    pub fn is_game_over(&self) -> bool {
        self.active_player_count() < 2
    }

    /// Check if a specific player is active.
    #[inline]
    pub fn is_active(&self, player: Player) -> bool {
        self.player_status[player.index()] == PlayerStatus::Active
    }

    /// Check if a specific player is DKW.
    #[inline]
    pub fn is_dkw(&self, player: Player) -> bool {
        self.player_status[player.index()] == PlayerStatus::DKW
    }

    /// Check if a specific player is eliminated.
    #[inline]
    pub fn is_eliminated(&self, player: Player) -> bool {
        self.player_status[player.index()] == PlayerStatus::Eliminated
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_state() {
        let state = GameState::new();
        assert_eq!(state.side_to_move, Player::Red);
        assert_eq!(state.castling_rights, 0xFF);
        assert!(state.en_passant.is_none());
        assert_eq!(state.half_move_clock, 0);
        assert_eq!(state.full_move_number, 1);
        assert_eq!(state.ply, 0);
        assert_eq!(state.active_player_count(), 4);
        assert!(!state.is_game_over());
        assert_ne!(state.zobrist_hash, 0);
    }

    #[test]
    fn test_turn_cycling() {
        let mut state = GameState::new();
        assert_eq!(state.side_to_move, Player::Red);
        state.advance_turn();
        assert_eq!(state.side_to_move, Player::Blue);
        state.advance_turn();
        assert_eq!(state.side_to_move, Player::Yellow);
        state.advance_turn();
        assert_eq!(state.side_to_move, Player::Green);
        state.advance_turn();
        assert_eq!(state.side_to_move, Player::Red);
    }

    #[test]
    fn test_skip_eliminated() {
        let mut state = GameState::new();
        state.player_status[Player::Blue.index()] = PlayerStatus::Eliminated;
        assert_eq!(state.side_to_move, Player::Red);
        state.advance_turn();
        assert_eq!(state.side_to_move, Player::Yellow); // Blue skipped
    }

    #[test]
    fn test_skip_dkw() {
        let mut state = GameState::new();
        state.player_status[Player::Yellow.index()] = PlayerStatus::DKW;
        state.side_to_move = Player::Blue;
        state.advance_turn();
        assert_eq!(state.side_to_move, Player::Green); // Yellow skipped
    }

    #[test]
    fn test_active_count() {
        let mut state = GameState::new();
        assert_eq!(state.active_player_count(), 4);
        state.player_status[Player::Red.index()] = PlayerStatus::Eliminated;
        assert_eq!(state.active_player_count(), 3);
        state.player_status[Player::Blue.index()] = PlayerStatus::DKW;
        assert_eq!(state.active_player_count(), 2);
        state.player_status[Player::Yellow.index()] = PlayerStatus::Eliminated;
        assert_eq!(state.active_player_count(), 1);
        assert!(state.is_game_over());
    }

    #[test]
    fn test_hash_matches_recomputation() {
        let state = GameState::new();
        let expected = zobrist::compute_full_hash(
            &state.board,
            state.side_to_move,
            state.castling_rights,
            state.en_passant,
        );
        assert_eq!(state.zobrist_hash, expected);
    }

    #[test]
    fn test_ffa_scores_separate_from_eval() {
        let state = GameState::new();
        // FFA scores are integers for game scoring
        for score in &state.ffa_scores {
            assert_eq!(*score, 0);
        }
    }
}
