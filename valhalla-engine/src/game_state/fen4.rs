//! FEN4 serialization and parsing for four-player chess.
//! Format: `<rank14>/<rank13>/.../<rank1> <side> <castling> <ep> <halfmove>`
//! See 4PC_RULES_REFERENCE.md Section 9.5.

use crate::board::Board;
use crate::types::{ColoredPiece, PieceType, Player, Square, BOARD_SIZE, PLAYERS};
use crate::zobrist;

use super::{GameMode, GameState, PlayerStatus};

/// FEN4 parsing error.
#[derive(Debug)]
pub enum FenError {
    InvalidFormat(String),
    InvalidRank(String),
    InvalidPiece(String),
    InvalidSideToMove(String),
    InvalidCastling(String),
    InvalidEp(String),
}

impl std::fmt::Display for FenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FenError::InvalidFormat(s) => write!(f, "Invalid FEN4 format: {}", s),
            FenError::InvalidRank(s) => write!(f, "Invalid rank: {}", s),
            FenError::InvalidPiece(s) => write!(f, "Invalid piece: {}", s),
            FenError::InvalidSideToMove(s) => write!(f, "Invalid side to move: {}", s),
            FenError::InvalidCastling(s) => write!(f, "Invalid castling: {}", s),
            FenError::InvalidEp(s) => write!(f, "Invalid EP square: {}", s),
        }
    }
}

impl GameState {
    /// Serialize this position to FEN4 string.
    pub fn to_fen4(&self) -> String {
        let mut parts = Vec::new();

        // Board: rank 14 down to rank 1
        let mut rank_strs = Vec::new();
        for rank in (0..BOARD_SIZE).rev() {
            rank_strs.push(serialize_rank(&self.board, rank));
        }
        parts.push(rank_strs.join("/"));

        // Side to move
        parts.push(String::from(match self.side_to_move {
            Player::Red => "r",
            Player::Blue => "b",
            Player::Yellow => "y",
            Player::Green => "g",
        }));

        // Castling rights
        parts.push(serialize_castling(self.castling_rights));

        // EP square
        match self.en_passant {
            Some(sq) => parts.push(sq.to_algebraic()),
            None => parts.push("-".to_string()),
        }

        // Half-move clock
        parts.push(self.half_move_clock.to_string());

        parts.join(" ")
    }

    /// Parse a FEN4 string into a GameState.
    pub fn from_fen4(s: &str) -> Result<GameState, FenError> {
        let fields: Vec<&str> = s.split_whitespace().collect();
        if fields.len() < 4 {
            return Err(FenError::InvalidFormat(
                "Need at least 4 fields".to_string(),
            ));
        }

        // Parse board
        let rank_strs: Vec<&str> = fields[0].split('/').collect();
        if rank_strs.len() != 14 {
            return Err(FenError::InvalidFormat(format!(
                "Expected 14 ranks, got {}",
                rank_strs.len()
            )));
        }

        let mut board = Board::new();
        for (i, rank_str) in rank_strs.iter().enumerate() {
            let rank = 13 - i as u8; // rank 14 first, rank 1 last
            parse_rank(&mut board, rank, rank_str)?;
        }

        // Side to move
        let side_to_move = match fields[1] {
            "r" => Player::Red,
            "b" => Player::Blue,
            "y" => Player::Yellow,
            "g" => Player::Green,
            s => return Err(FenError::InvalidSideToMove(s.to_string())),
        };

        // Castling
        let castling_rights = parse_castling(fields[2])?;

        // EP square
        let en_passant = if fields[3] == "-" {
            None
        } else {
            Some(
                Square::from_algebraic(fields[3])
                    .ok_or_else(|| FenError::InvalidEp(fields[3].to_string()))?,
            )
        };

        // Half-move clock (optional)
        let half_move_clock = if fields.len() > 4 {
            fields[4].parse::<u16>().unwrap_or(0)
        } else {
            0
        };

        let hash = zobrist::compute_full_hash(&board, side_to_move, castling_rights, en_passant);

        Ok(GameState {
            board,
            side_to_move,
            castling_rights,
            en_passant,
            ep_pushing_player: None, // Not encoded in FEN4; inferred if needed
            half_move_clock,
            full_move_number: 1,
            ply: 0,
            zobrist_hash: hash,
            player_status: [PlayerStatus::Active; PLAYERS],
            ffa_scores: [0; PLAYERS],
            game_mode: GameMode::FFA,
        })
    }
}

/// Serialize one rank of the board.
fn serialize_rank(board: &Board, rank: u8) -> String {
    let mut result = String::new();
    let mut empty_count = 0u8;

    for file in 0..BOARD_SIZE {
        if !Square::is_valid(rank, file) {
            // Flush empties before invalid zone
            if empty_count > 0 {
                result.push_str(&empty_count.to_string());
                empty_count = 0;
            }
            result.push_str("xxx");
            // Skip the rest of the 3-wide corner
            // Corners are 3 wide, so skip to file+2 (the loop will increment)
            // But we need to handle each file individually since the loop iterates
            // Actually corners are contiguous 3 files, so we emit xxx for each invalid file
            // That's wrong — xxx represents 3 invalid squares at once.
            // Let's consume all 3 corner files at once.
            // Skip the next 2 invalid files
            // Wait, the loop goes file by file. We need to handle this differently.
            // Let me restructure: emit xxx once for the 3-file corner block.
            break; // Will restructure below
        }

        let idx = rank as usize * BOARD_SIZE as usize + file as usize;
        match board.squares[idx] {
            Some(cp) => {
                if empty_count > 0 {
                    result.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                result.push(cp.player.fen_char());
                result.push(cp.piece_type.fen_char());
            }
            None => {
                empty_count += 1;
            }
        }
    }

    if empty_count > 0 {
        result.push_str(&empty_count.to_string());
    }

    // If we broke out due to corner handling, do the proper version
    if result.contains("xxx") {
        // Redo with proper corner handling
        return serialize_rank_proper(board, rank);
    }

    result
}

/// Properly serialize a rank, handling 3-wide corner blocks.
fn serialize_rank_proper(board: &Board, rank: u8) -> String {
    let mut result = String::new();
    let mut empty_count = 0u8;
    let mut file = 0u8;

    while file < BOARD_SIZE {
        if !Square::is_valid(rank, file) {
            // Flush empties
            if empty_count > 0 {
                result.push_str(&empty_count.to_string());
                empty_count = 0;
            }
            // Emit xxx for the 3-file invalid block
            result.push_str("xxx");
            file += 3; // Skip all 3 invalid files
            continue;
        }

        let idx = rank as usize * BOARD_SIZE as usize + file as usize;
        match board.squares[idx] {
            Some(cp) => {
                if empty_count > 0 {
                    result.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                result.push(cp.player.fen_char());
                result.push(cp.piece_type.fen_char());
            }
            None => {
                empty_count += 1;
            }
        }
        file += 1;
    }

    if empty_count > 0 {
        result.push_str(&empty_count.to_string());
    }

    result
}

/// Serialize castling rights to FEN4 format.
/// Uppercase = kingside, lowercase = queenside.
/// R/r = Red, B/b = Blue, Y/y = Yellow, G/g = Green.
fn serialize_castling(rights: u8) -> String {
    if rights == 0 {
        return "-".to_string();
    }
    let mut result = String::new();
    let chars = [
        (0, 'R'), // Red KS
        (1, 'r'), // Red QS
        (2, 'B'), // Blue KS
        (3, 'b'), // Blue QS
        (4, 'Y'), // Yellow KS
        (5, 'y'), // Yellow QS
        (6, 'G'), // Green KS
        (7, 'g'), // Green QS
    ];
    for (bit, ch) in chars {
        if rights & (1 << bit) != 0 {
            result.push(ch);
        }
    }
    result
}

/// Parse castling rights from FEN4 string.
fn parse_castling(s: &str) -> Result<u8, FenError> {
    if s == "-" {
        return Ok(0);
    }
    let mut rights = 0u8;
    for ch in s.chars() {
        match ch {
            'R' => rights |= 1 << 0,
            'r' => rights |= 1 << 1,
            'B' => rights |= 1 << 2,
            'b' => rights |= 1 << 3,
            'Y' => rights |= 1 << 4,
            'y' => rights |= 1 << 5,
            'G' => rights |= 1 << 6,
            'g' => rights |= 1 << 7,
            _ => return Err(FenError::InvalidCastling(format!("Unknown char: {}", ch))),
        }
    }
    Ok(rights)
}

/// Parse one rank from a FEN4 rank string.
fn parse_rank(board: &mut Board, rank: u8, s: &str) -> Result<(), FenError> {
    let mut file = 0u8;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() && file < BOARD_SIZE {
        let ch = chars[i];

        // Invalid corner marker
        if ch == 'x' {
            if i + 2 < chars.len() && chars[i + 1] == 'x' && chars[i + 2] == 'x' {
                file += 3; // Skip 3 invalid files
                i += 3;
                continue;
            }
            return Err(FenError::InvalidRank(format!(
                "Unexpected 'x' at position {} in rank {}",
                i, rank
            )));
        }

        // Empty square count
        if ch.is_ascii_digit() {
            // Could be 1 or 2 digit number
            let mut num_str = String::new();
            num_str.push(ch);
            if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                num_str.push(chars[i + 1]);
                i += 1;
            }
            let count: u8 = num_str
                .parse()
                .map_err(|_| FenError::InvalidRank(format!("Bad count: {}", num_str)))?;
            file += count;
            i += 1;
            continue;
        }

        // Piece: player char + piece char
        if i + 1 >= chars.len() {
            return Err(FenError::InvalidPiece(format!(
                "Incomplete piece at end of rank {}",
                rank
            )));
        }

        let player_ch = ch;
        let piece_ch = chars[i + 1];

        let player = Player::from_fen_char(player_ch)
            .ok_or_else(|| FenError::InvalidPiece(format!("Bad player: {}", player_ch)))?;
        let piece_type = PieceType::from_fen_char(piece_ch)
            .ok_or_else(|| FenError::InvalidPiece(format!("Bad piece: {}", piece_ch)))?;

        if let Some(sq) = Square::from_rank_file(rank, file) {
            board.place_piece(sq, ColoredPiece::new(player, piece_type));
        }

        file += 1;
        i += 2;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TOTAL_SQUARES;

    #[test]
    fn test_starting_position_fen4_roundtrip() {
        let state = GameState::new();
        let fen = state.to_fen4();

        let parsed = GameState::from_fen4(&fen).expect("Failed to parse FEN4");

        // Verify board matches
        for i in 0..TOTAL_SQUARES {
            assert_eq!(
                parsed.board.squares[i], state.board.squares[i],
                "Board mismatch at square {} in FEN4 roundtrip. FEN: {}",
                i, fen
            );
        }

        assert_eq!(parsed.side_to_move, state.side_to_move);
        assert_eq!(parsed.castling_rights, state.castling_rights);
        assert_eq!(parsed.en_passant, state.en_passant);
    }

    #[test]
    fn test_fen4_string_roundtrip() {
        let state = GameState::new();
        let fen1 = state.to_fen4();
        let parsed = GameState::from_fen4(&fen1).unwrap();
        let fen2 = parsed.to_fen4();
        assert_eq!(fen1, fen2, "FEN4 string should round-trip identically");
    }

    #[test]
    fn test_castling_serialization() {
        assert_eq!(serialize_castling(0xFF), "RrBbYyGg");
        assert_eq!(serialize_castling(0), "-");
        assert_eq!(serialize_castling(0b00000001), "R");
        assert_eq!(serialize_castling(0b00000101), "RB");
    }

    #[test]
    fn test_castling_parse_roundtrip() {
        let rights = 0xFF;
        let s = serialize_castling(rights);
        let parsed = parse_castling(&s).unwrap();
        assert_eq!(parsed, rights);
    }

    #[test]
    fn test_fen4_side_to_move() {
        let state = GameState::new();
        let fen = state.to_fen4();
        assert!(fen.contains(" r "), "Starting FEN should have Red to move");
    }

    #[test]
    fn test_fen4_no_ep() {
        let state = GameState::new();
        let fen = state.to_fen4();
        let fields: Vec<&str> = fen.split_whitespace().collect();
        assert_eq!(fields[3], "-", "Starting position should have no EP");
    }

    #[test]
    fn test_fen4_contains_xxx_for_corners() {
        let state = GameState::new();
        let fen = state.to_fen4();
        // Rank 1 and rank 14 should have xxx at both ends
        let ranks: Vec<&str> = fen.split_whitespace().next().unwrap().split('/').collect();
        // Rank 14 is first, rank 1 is last
        assert!(ranks[0].starts_with("xxx"), "Rank 14 should start with xxx");
        assert!(ranks[0].ends_with("xxx"), "Rank 14 should end with xxx");
        assert!(ranks[13].starts_with("xxx"), "Rank 1 should start with xxx");
        assert!(ranks[13].ends_with("xxx"), "Rank 1 should end with xxx");
    }

    #[test]
    fn test_fen4_hash_matches() {
        let state = GameState::new();
        let fen = state.to_fen4();
        let parsed = GameState::from_fen4(&fen).unwrap();

        let expected = zobrist::compute_full_hash(
            &parsed.board,
            parsed.side_to_move,
            parsed.castling_rights,
            parsed.en_passant,
        );
        assert_eq!(parsed.zobrist_hash, expected);
    }
}
