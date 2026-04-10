//! Player representation for four-player chess.
//! Turn order: Red -> Blue -> Yellow -> Green (clockwise).

/// The four players, indexed 0-3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Player {
    Red = 0,
    Blue = 1,
    Yellow = 2,
    Green = 3,
}

/// All players in turn order, for iteration.
pub const ALL_PLAYERS: [Player; 4] = [Player::Red, Player::Blue, Player::Yellow, Player::Green];

impl Player {
    /// Returns the next player in turn order (R->B->Y->G->R).
    #[inline]
    pub const fn next(self) -> Player {
        match self {
            Player::Red => Player::Blue,
            Player::Blue => Player::Yellow,
            Player::Yellow => Player::Green,
            Player::Green => Player::Red,
        }
    }

    /// Returns the previous player in turn order (R<-B<-Y<-G<-R).
    #[inline]
    pub const fn prev(self) -> Player {
        match self {
            Player::Red => Player::Green,
            Player::Blue => Player::Red,
            Player::Yellow => Player::Blue,
            Player::Green => Player::Yellow,
        }
    }

    /// Returns this player's index (0-3).
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Creates a Player from an index (0-3). Returns None if invalid.
    #[inline]
    pub const fn from_index(index: u8) -> Option<Player> {
        match index {
            0 => Some(Player::Red),
            1 => Some(Player::Blue),
            2 => Some(Player::Yellow),
            3 => Some(Player::Green),
            _ => None,
        }
    }

    /// Pawn push direction as (rank_delta, file_delta).
    /// Red pushes North (+rank), Blue pushes East (+file),
    /// Yellow pushes South (-rank), Green pushes West (-file).
    #[inline]
    pub const fn push_direction(self) -> (i8, i8) {
        match self {
            Player::Red => (1, 0),
            Player::Blue => (0, 1),
            Player::Yellow => (-1, 0),
            Player::Green => (0, -1),
        }
    }

    /// Pawn capture directions as two (rank_delta, file_delta) pairs.
    /// Each capture is the push direction combined with one perpendicular step.
    #[inline]
    pub const fn capture_directions(self) -> [(i8, i8); 2] {
        match self {
            Player::Red => [(1, 1), (1, -1)],      // NE, NW
            Player::Blue => [(1, 1), (-1, 1)],     // NE, SE
            Player::Yellow => [(-1, 1), (-1, -1)], // SE, SW
            Player::Green => [(1, -1), (-1, -1)],  // NW, SW
        }
    }

    /// The starting rank index (for Red/Yellow) or file index (for Blue/Green)
    /// where this player's pawns begin. Used for double-step validation.
    #[inline]
    pub const fn pawn_start_rank_or_file(self) -> u8 {
        match self {
            Player::Red => 1,     // rank 2 (index 1)
            Player::Blue => 1,    // file b (index 1)
            Player::Yellow => 12, // rank 13 (index 12)
            Player::Green => 12,  // file m (index 12)
        }
    }

    /// The rank/file index where this player's pawns promote (FFA mode).
    #[inline]
    pub const fn promotion_rank_or_file(self) -> u8 {
        match self {
            Player::Red => 8,    // rank 9 (index 8)
            Player::Blue => 8,   // file i (index 8)
            Player::Yellow => 5, // rank 6 (index 5)
            Player::Green => 5,  // file f (index 5)
        }
    }

    /// Check if a pawn at (rank, file) is on this player's starting rank/file.
    #[inline]
    pub const fn is_pawn_on_start(self, rank: u8, file: u8) -> bool {
        match self {
            Player::Red => rank == 1 && file >= 3 && file <= 10,
            Player::Blue => file == 1 && rank >= 3 && rank <= 10,
            Player::Yellow => rank == 12 && file >= 3 && file <= 10,
            Player::Green => file == 12 && rank >= 3 && rank <= 10,
        }
    }

    /// Check if a pawn at (rank, file) is on this player's promotion rank/file (FFA).
    #[inline]
    pub const fn is_promotion_square(self, rank: u8, file: u8) -> bool {
        match self {
            Player::Red | Player::Yellow => rank == self.promotion_rank_or_file(),
            Player::Blue | Player::Green => file == self.promotion_rank_or_file(),
        }
    }

    /// Returns the FEN4 character for this player.
    #[inline]
    pub const fn fen_char(self) -> char {
        match self {
            Player::Red => 'r',
            Player::Blue => 'b',
            Player::Yellow => 'y',
            Player::Green => 'g',
        }
    }

    /// Creates a Player from FEN4 character.
    #[inline]
    pub const fn from_fen_char(c: char) -> Option<Player> {
        match c {
            'r' => Some(Player::Red),
            'b' => Some(Player::Blue),
            'y' => Some(Player::Yellow),
            'g' => Some(Player::Green),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_order() {
        assert_eq!(Player::Red.next(), Player::Blue);
        assert_eq!(Player::Blue.next(), Player::Yellow);
        assert_eq!(Player::Yellow.next(), Player::Green);
        assert_eq!(Player::Green.next(), Player::Red);
    }

    #[test]
    fn test_prev_order() {
        assert_eq!(Player::Red.prev(), Player::Green);
        assert_eq!(Player::Blue.prev(), Player::Red);
        assert_eq!(Player::Yellow.prev(), Player::Blue);
        assert_eq!(Player::Green.prev(), Player::Yellow);
    }

    #[test]
    fn test_index_roundtrip() {
        for p in ALL_PLAYERS {
            assert_eq!(Player::from_index(p.index() as u8), Some(p));
        }
        assert_eq!(Player::from_index(4), None);
    }

    #[test]
    fn test_red_push_direction() {
        assert_eq!(Player::Red.push_direction(), (1, 0));
    }

    #[test]
    fn test_blue_push_direction() {
        assert_eq!(Player::Blue.push_direction(), (0, 1));
    }

    #[test]
    fn test_yellow_push_direction() {
        assert_eq!(Player::Yellow.push_direction(), (-1, 0));
    }

    #[test]
    fn test_green_push_direction() {
        assert_eq!(Player::Green.push_direction(), (0, -1));
    }

    #[test]
    fn test_red_capture_directions() {
        assert_eq!(Player::Red.capture_directions(), [(1, 1), (1, -1)]);
    }

    #[test]
    fn test_blue_capture_directions() {
        assert_eq!(Player::Blue.capture_directions(), [(1, 1), (-1, 1)]);
    }

    #[test]
    fn test_yellow_capture_directions() {
        assert_eq!(Player::Yellow.capture_directions(), [(-1, 1), (-1, -1)]);
    }

    #[test]
    fn test_green_capture_directions() {
        assert_eq!(Player::Green.capture_directions(), [(1, -1), (-1, -1)]);
    }

    #[test]
    fn test_red_pawn_on_start() {
        assert!(Player::Red.is_pawn_on_start(1, 3)); // d2
        assert!(Player::Red.is_pawn_on_start(1, 10)); // k2
        assert!(!Player::Red.is_pawn_on_start(1, 2)); // c2 (invalid area)
        assert!(!Player::Red.is_pawn_on_start(2, 3)); // d3 (not start rank)
    }

    #[test]
    fn test_blue_pawn_on_start() {
        assert!(Player::Blue.is_pawn_on_start(3, 1)); // b4
        assert!(Player::Blue.is_pawn_on_start(10, 1)); // b11
        assert!(!Player::Blue.is_pawn_on_start(2, 1)); // b3 (invalid area)
        assert!(!Player::Blue.is_pawn_on_start(3, 2)); // c4 (not start file)
    }

    #[test]
    fn test_yellow_pawn_on_start() {
        assert!(Player::Yellow.is_pawn_on_start(12, 3)); // d13
        assert!(Player::Yellow.is_pawn_on_start(12, 10)); // k13
        assert!(!Player::Yellow.is_pawn_on_start(11, 3)); // d12 (not start rank)
    }

    #[test]
    fn test_green_pawn_on_start() {
        assert!(Player::Green.is_pawn_on_start(3, 12)); // m4
        assert!(Player::Green.is_pawn_on_start(10, 12)); // m11
        assert!(!Player::Green.is_pawn_on_start(3, 11)); // l4 (not start file)
    }

    #[test]
    fn test_red_promotion() {
        assert!(Player::Red.is_promotion_square(8, 5)); // rank 9, any valid file
        assert!(!Player::Red.is_promotion_square(7, 5));
    }

    #[test]
    fn test_blue_promotion() {
        assert!(Player::Blue.is_promotion_square(5, 8)); // any valid rank, file i
        assert!(!Player::Blue.is_promotion_square(5, 7));
    }

    #[test]
    fn test_yellow_promotion() {
        assert!(Player::Yellow.is_promotion_square(5, 5)); // rank 6
        assert!(!Player::Yellow.is_promotion_square(6, 5));
    }

    #[test]
    fn test_green_promotion() {
        assert!(Player::Green.is_promotion_square(5, 5)); // file f
        assert!(!Player::Green.is_promotion_square(5, 6));
    }

    #[test]
    fn test_fen_char_roundtrip() {
        for p in ALL_PLAYERS {
            assert_eq!(Player::from_fen_char(p.fen_char()), Some(p));
        }
    }
}
