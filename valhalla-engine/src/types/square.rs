/// Square representation for the 14x14 four-player chess board.
/// Index = rank_index * 14 + file_index. Range: 0..195.
/// 36 squares are invalid (four 3x3 corners).
use crate::types::constants::{BOARD_SIZE, TOTAL_SQUARES, VALID_SQUARES};

/// A square on the 14x14 board, stored as a single byte index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Square(pub u8);

impl Square {
    /// Create a square from rank and file indices (0-based).
    /// Returns None if the square is invalid (corner dead zone or out of bounds).
    #[inline]
    pub const fn from_rank_file(rank: u8, file: u8) -> Option<Square> {
        if !Self::is_valid(rank, file) {
            return None;
        }
        Some(Square(rank * BOARD_SIZE + file))
    }

    /// Create a square from raw index. No validity check.
    #[inline]
    pub const fn from_index_unchecked(index: u8) -> Square {
        Square(index)
    }

    /// Raw index (0..195).
    #[inline]
    pub const fn index(self) -> u8 {
        self.0
    }

    /// Rank index (0..13). Rank 1 = index 0, Rank 14 = index 13.
    #[inline]
    pub const fn rank(self) -> u8 {
        self.0 / BOARD_SIZE
    }

    /// File index (0..13). File a = index 0, File n = index 13.
    #[inline]
    pub const fn file(self) -> u8 {
        self.0 % BOARD_SIZE
    }

    /// Check if a (rank, file) coordinate is a valid playable square.
    /// Invalid: out of bounds, or in any of the four 3x3 corner dead zones.
    #[inline]
    pub const fn is_valid(rank: u8, file: u8) -> bool {
        if rank >= BOARD_SIZE || file >= BOARD_SIZE {
            return false;
        }
        // Four 3x3 corners are invalid
        let in_sw = rank < 3 && file < 3;
        let in_se = rank < 3 && file > 10;
        let in_nw = rank > 10 && file < 3;
        let in_ne = rank > 10 && file > 10;
        !(in_sw || in_se || in_nw || in_ne)
    }

    /// Check if this square's index corresponds to a valid square.
    #[inline]
    pub fn is_valid_index(self) -> bool {
        IS_VALID[self.0 as usize]
    }

    /// Offset this square by (rank_delta, file_delta).
    /// Returns None if the result is out of bounds or an invalid corner square.
    #[inline]
    pub const fn offset(self, rank_delta: i8, file_delta: i8) -> Option<Square> {
        let new_rank = self.rank() as i8 + rank_delta;
        let new_file = self.file() as i8 + file_delta;
        if new_rank < 0
            || new_rank >= BOARD_SIZE as i8
            || new_file < 0
            || new_file >= BOARD_SIZE as i8
        {
            return None;
        }
        Square::from_rank_file(new_rank as u8, new_file as u8)
    }

    /// Convert to algebraic notation (e.g., "d1", "a7", "n14").
    pub fn to_algebraic(self) -> String {
        let file_char = (b'a' + self.file()) as char;
        let rank_num = self.rank() + 1;
        format!("{}{}", file_char, rank_num)
    }

    /// Parse from algebraic notation (e.g., "d1", "a7", "n14").
    pub fn from_algebraic(s: &str) -> Option<Square> {
        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return None;
        }
        let file_char = bytes[0];
        if !(b'a'..=b'n').contains(&file_char) {
            return None;
        }
        let file = file_char - b'a';
        let rank_str = &s[1..];
        let rank_num: u8 = rank_str.parse().ok()?;
        if !(1..=14).contains(&rank_num) {
            return None;
        }
        let rank = rank_num - 1;
        Square::from_rank_file(rank, file)
    }
}

/// Pre-computed validity lookup table for all 196 square indices.
pub static IS_VALID: [bool; TOTAL_SQUARES] = {
    let mut table = [false; TOTAL_SQUARES];
    let mut i: u8 = 0;
    while i < TOTAL_SQUARES as u8 {
        let rank = i / BOARD_SIZE;
        let file = i % BOARD_SIZE;
        table[i as usize] = Square::is_valid(rank, file);
        i += 1;
    }
    table
};

/// Pre-computed list of all 160 valid square indices.
pub static VALID_SQUARES_LIST: [Square; VALID_SQUARES] = {
    let mut list = [Square(0); VALID_SQUARES];
    let mut count = 0;
    let mut i: u8 = 0;
    while i < TOTAL_SQUARES as u8 {
        let rank = i / BOARD_SIZE;
        let file = i % BOARD_SIZE;
        if Square::is_valid(rank, file) {
            list[count] = Square(i);
            count += 1;
        }
        i += 1;
    }
    list
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_square_count() {
        let count = IS_VALID.iter().filter(|&&v| v).count();
        assert_eq!(count, VALID_SQUARES);
    }

    #[test]
    fn test_valid_squares_list_length() {
        // Verify all entries in the list are valid
        for sq in &VALID_SQUARES_LIST {
            assert!(sq.is_valid_index(), "Square {} should be valid", sq.0);
        }
    }

    // --- Invalid corner tests (all 36) ---

    #[test]
    fn test_sw_corner_invalid() {
        // a1-c3 (files 0-2, ranks 0-2)
        for rank in 0..3u8 {
            for file in 0..3u8 {
                assert!(
                    !Square::is_valid(rank, file),
                    "SW corner ({},{}) should be invalid",
                    rank,
                    file
                );
            }
        }
    }

    #[test]
    fn test_se_corner_invalid() {
        // l1-n3 (files 11-13, ranks 0-2)
        for rank in 0..3u8 {
            for file in 11..14u8 {
                assert!(
                    !Square::is_valid(rank, file),
                    "SE corner ({},{}) should be invalid",
                    rank,
                    file
                );
            }
        }
    }

    #[test]
    fn test_nw_corner_invalid() {
        // a12-c14 (files 0-2, ranks 11-13)
        for rank in 11..14u8 {
            for file in 0..3u8 {
                assert!(
                    !Square::is_valid(rank, file),
                    "NW corner ({},{}) should be invalid",
                    rank,
                    file
                );
            }
        }
    }

    #[test]
    fn test_ne_corner_invalid() {
        // l12-n14 (files 11-13, ranks 11-13)
        for rank in 11..14u8 {
            for file in 11..14u8 {
                assert!(
                    !Square::is_valid(rank, file),
                    "NE corner ({},{}) should be invalid",
                    rank,
                    file
                );
            }
        }
    }

    #[test]
    fn test_center_valid() {
        // Center squares should all be valid
        for rank in 3..11u8 {
            for file in 3..11u8 {
                assert!(
                    Square::is_valid(rank, file),
                    "Center ({},{}) should be valid",
                    rank,
                    file
                );
            }
        }
    }

    #[test]
    fn test_rank_file_roundtrip() {
        for sq in &VALID_SQUARES_LIST {
            let rank = sq.rank();
            let file = sq.file();
            let reconstructed = Square::from_rank_file(rank, file).unwrap();
            assert_eq!(*sq, reconstructed);
        }
    }

    #[test]
    fn test_algebraic_roundtrip() {
        for sq in &VALID_SQUARES_LIST {
            let alg = sq.to_algebraic();
            let parsed = Square::from_algebraic(&alg).unwrap();
            assert_eq!(*sq, parsed, "Failed roundtrip for {}", alg);
        }
    }

    #[test]
    fn test_specific_squares() {
        // Red king: h1 = file 7, rank 0
        let h1 = Square::from_algebraic("h1").unwrap();
        assert_eq!(h1.index(), 7);

        // Blue king: a7 = file 0, rank 6
        let a7 = Square::from_algebraic("a7").unwrap();
        assert_eq!(a7.index(), 84);

        // Yellow king: g14 = file 6, rank 13
        let g14 = Square::from_algebraic("g14").unwrap();
        assert_eq!(g14.index(), 188);

        // Green king: n8 = file 13, rank 7
        let n8 = Square::from_algebraic("n8").unwrap();
        assert_eq!(n8.index(), 111);
    }

    #[test]
    fn test_offset_valid() {
        let d2 = Square::from_algebraic("d2").unwrap();
        // Red pawn push: d2 -> d3
        let d3 = d2.offset(1, 0).unwrap();
        assert_eq!(d3.to_algebraic(), "d3");
    }

    #[test]
    fn test_offset_into_corner_returns_none() {
        // b4 offset to (-1, -1) would be a3 which is still valid,
        // but a1 is invalid
        let d1 = Square::from_algebraic("d1").unwrap();
        // d1 offset (-1, 0) goes out of bounds (rank -1)
        assert!(d1.offset(-1, 0).is_none());
    }

    #[test]
    fn test_offset_into_invalid_corner() {
        // d3 is valid. Offset to (-1, -1) = c2 which is invalid (SW corner)
        let d3 = Square::from_algebraic("d3").unwrap();
        assert!(d3.offset(-1, -1).is_none());
    }

    #[test]
    fn test_out_of_bounds() {
        assert!(!Square::is_valid(14, 0));
        assert!(!Square::is_valid(0, 14));
    }

    #[test]
    fn test_invalid_algebraic() {
        assert!(Square::from_algebraic("a1").is_none()); // SW corner
        assert!(Square::from_algebraic("n14").is_none()); // NE corner
        assert!(Square::from_algebraic("z1").is_none()); // bad file
        assert!(Square::from_algebraic("a0").is_none()); // bad rank
        assert!(Square::from_algebraic("a15").is_none()); // bad rank
    }
}
