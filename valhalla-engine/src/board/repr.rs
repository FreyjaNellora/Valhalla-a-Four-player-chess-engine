/// Board representation for 14x14 four-player chess.
/// Fixed-size arrays only — no Vec or Box in this struct.
use crate::types::{
    ColoredPiece, PieceType, Player, Square, ALL_PLAYERS, ELIMINATED_KING_SENTINEL,
    MAX_PIECES_PER_PLAYER, PLAYERS, TOTAL_SQUARES,
};

/// The chess board. Mailbox representation with piece lists for fast iteration.
#[derive(Clone)]
pub struct Board {
    /// Mailbox: what's on each square. None = empty (or invalid corner).
    pub squares: [Option<ColoredPiece>; TOTAL_SQUARES],
    /// King square index per player. ELIMINATED_KING_SENTINEL (255) if eliminated.
    pub king_sq: [u8; PLAYERS],
    /// Piece list per player: array of squares where that player has pieces.
    pub piece_list: [[Square; MAX_PIECES_PER_PLAYER]; PLAYERS],
    /// Number of pieces per player in the piece list.
    pub piece_count: [u8; PLAYERS],
}

impl Board {
    /// Create an empty board.
    pub fn new() -> Self {
        Self {
            squares: [None; TOTAL_SQUARES],
            king_sq: [ELIMINATED_KING_SENTINEL; PLAYERS],
            piece_list: [[Square(0); MAX_PIECES_PER_PLAYER]; PLAYERS],
            piece_count: [0; PLAYERS],
        }
    }

    /// Get the piece on a square.
    #[inline]
    pub fn get(&self, sq: Square) -> Option<ColoredPiece> {
        self.squares[sq.index() as usize]
    }

    /// Place a piece on a square. Updates mailbox, piece list, and king tracking.
    pub fn place_piece(&mut self, sq: Square, piece: ColoredPiece) {
        debug_assert!(sq.is_valid_index(), "Cannot place piece on invalid square");
        debug_assert!(
            self.squares[sq.index() as usize].is_none(),
            "Square {} already occupied",
            sq.index()
        );

        self.squares[sq.index() as usize] = Some(piece);
        let pi = piece.player.index();

        // Track king
        if piece.piece_type == PieceType::King {
            self.king_sq[pi] = sq.index();
        }

        // Add to piece list
        let count = self.piece_count[pi] as usize;
        self.piece_list[pi][count] = sq;
        self.piece_count[pi] += 1;
    }

    /// Remove a piece from a square. Updates mailbox, piece list, and king tracking.
    /// Returns the removed piece.
    pub fn remove_piece(&mut self, sq: Square) -> Option<ColoredPiece> {
        let piece = self.squares[sq.index() as usize]?;
        self.squares[sq.index() as usize] = None;
        let pi = piece.player.index();

        // Remove from piece list (swap-remove)
        let count = self.piece_count[pi] as usize;
        for i in 0..count {
            if self.piece_list[pi][i] == sq {
                self.piece_list[pi][i] = self.piece_list[pi][count - 1];
                self.piece_count[pi] -= 1;
                break;
            }
        }

        Some(piece)
    }

    /// Move a piece from one square to another. Does NOT handle captures.
    /// For captures, call remove_piece on target first.
    pub fn move_piece(&mut self, from: Square, to: Square) {
        let piece = self.squares[from.index() as usize].expect("No piece on source square");
        self.squares[from.index() as usize] = None;
        self.squares[to.index() as usize] = Some(piece);
        let pi = piece.player.index();

        // Update king tracking
        if piece.piece_type == PieceType::King {
            self.king_sq[pi] = to.index();
        }

        // Update piece list
        let count = self.piece_count[pi] as usize;
        for i in 0..count {
            if self.piece_list[pi][i] == from {
                self.piece_list[pi][i] = to;
                break;
            }
        }
    }

    /// Get the king square for a player. Returns None if eliminated.
    #[inline]
    pub fn king_square(&self, player: Player) -> Option<Square> {
        let sq = self.king_sq[player.index()];
        if sq == ELIMINATED_KING_SENTINEL {
            None
        } else {
            Some(Square(sq))
        }
    }

    /// Check if a player is eliminated (king sentinel).
    #[inline]
    pub fn is_eliminated(&self, player: Player) -> bool {
        self.king_sq[player.index()] == ELIMINATED_KING_SENTINEL
    }

    /// Iterate over all pieces for a player. Returns (square, piece_type) pairs.
    pub fn pieces_for_player(
        &self,
        player: Player,
    ) -> impl Iterator<Item = (Square, PieceType)> + '_ {
        let pi = player.index();
        let count = self.piece_count[pi] as usize;
        (0..count).map(move |i| {
            let sq = self.piece_list[pi][i];
            let piece = self.squares[sq.index() as usize].expect("Piece list inconsistency");
            debug_assert_eq!(piece.player, player);
            (sq, piece.piece_type)
        })
    }

    /// Total number of pieces on the board.
    pub fn total_pieces(&self) -> u32 {
        ALL_PLAYERS
            .iter()
            .map(|p| self.piece_count[p.index()] as u32)
            .sum()
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
