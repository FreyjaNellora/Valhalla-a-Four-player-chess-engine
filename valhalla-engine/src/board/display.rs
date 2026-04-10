use super::repr::Board;
use crate::types::{Square, BOARD_SIZE};
/// ASCII display for the board.
use std::fmt;

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "    a   b   c   d   e   f   g   h   i   j   k   l   m   n"
        )?;
        for rank in (0..BOARD_SIZE).rev() {
            write!(f, "{:>2}  ", rank + 1)?;
            for file in 0..BOARD_SIZE {
                if !Square::is_valid(rank, file) {
                    write!(f, ".   ")?;
                } else {
                    let idx = rank as usize * BOARD_SIZE as usize + file as usize;
                    match self.squares[idx] {
                        Some(cp) => {
                            let player_char = cp.player.fen_char();
                            let piece_char = cp.piece_type.fen_char();
                            write!(f, "{}{} ", player_char, piece_char)?;
                        }
                        None => write!(f, "    ")?,
                    }
                }
            }
            writeln!(f, " {:>2}", rank + 1)?;
        }
        writeln!(
            f,
            "    a   b   c   d   e   f   g   h   i   j   k   l   m   n"
        )?;
        Ok(())
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
