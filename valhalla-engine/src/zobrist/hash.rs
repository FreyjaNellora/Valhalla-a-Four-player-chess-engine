use super::keys::*;
use crate::board::Board;
/// Zobrist hash computation and incremental update.
use crate::types::{PieceType, Player, Square};

/// Compute the full Zobrist hash from scratch.
pub fn compute_full_hash(
    board: &Board,
    side_to_move: Player,
    castling_rights: u8,
    en_passant: Option<Square>,
) -> u64 {
    let mut hash = 0u64;

    // Piece-square contributions
    for player in crate::types::ALL_PLAYERS {
        for (sq, piece_type) in board.pieces_for_player(player) {
            hash ^= PIECE_SQUARE_KEYS[player.index()][piece_type as usize][sq.index() as usize];
        }
    }

    // Side to move
    hash ^= SIDE_TO_MOVE_KEYS[side_to_move.index()];

    // Castling rights
    for bit in 0..8u8 {
        if castling_rights & (1 << bit) != 0 {
            hash ^= CASTLING_KEYS[bit as usize];
        }
    }

    // En passant
    if let Some(ep_sq) = en_passant {
        hash ^= EP_SQUARE_KEYS[ep_sq.index() as usize];
    }

    hash
}

/// Toggle a piece on/off at a square (XOR — same operation for add and remove).
#[inline]
pub fn toggle_piece(hash: &mut u64, player: Player, piece_type: PieceType, sq: Square) {
    *hash ^= PIECE_SQUARE_KEYS[player.index()][piece_type as usize][sq.index() as usize];
}

/// Toggle side-to-move (remove old, add new).
#[inline]
pub fn toggle_side(hash: &mut u64, old: Player, new: Player) {
    *hash ^= SIDE_TO_MOVE_KEYS[old.index()];
    *hash ^= SIDE_TO_MOVE_KEYS[new.index()];
}

/// Toggle a single castling right bit.
#[inline]
pub fn toggle_castling_bit(hash: &mut u64, bit: u8) {
    *hash ^= CASTLING_KEYS[bit as usize];
}

/// Toggle en passant square (remove old if any, add new if any).
#[inline]
pub fn toggle_ep(hash: &mut u64, old: Option<Square>, new: Option<Square>) {
    if let Some(sq) = old {
        *hash ^= EP_SQUARE_KEYS[sq.index() as usize];
    }
    if let Some(sq) = new {
        *hash ^= EP_SQUARE_KEYS[sq.index() as usize];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::starting_position;

    #[test]
    fn test_starting_hash_deterministic() {
        let board = starting_position();
        let h1 = compute_full_hash(&board, Player::Red, 0xFF, None);
        let h2 = compute_full_hash(&board, Player::Red, 0xFF, None);
        assert_eq!(h1, h2, "Hash must be deterministic");
        assert_ne!(h1, 0, "Hash should not be zero");
    }

    #[test]
    fn test_toggle_piece_xor_roundtrip() {
        let mut hash = 0u64;
        let player = Player::Red;
        let piece = PieceType::Pawn;
        let sq = Square(17);
        toggle_piece(&mut hash, player, piece, sq);
        assert_ne!(hash, 0);
        toggle_piece(&mut hash, player, piece, sq);
        assert_eq!(hash, 0, "Double toggle should restore to zero");
    }

    #[test]
    fn test_different_side_different_hash() {
        let board = starting_position();
        let h_red = compute_full_hash(&board, Player::Red, 0xFF, None);
        let h_blue = compute_full_hash(&board, Player::Blue, 0xFF, None);
        assert_ne!(
            h_red, h_blue,
            "Different side to move should give different hash"
        );
    }

    #[test]
    fn test_different_castling_different_hash() {
        let board = starting_position();
        let h1 = compute_full_hash(&board, Player::Red, 0xFF, None);
        let h2 = compute_full_hash(&board, Player::Red, 0xFE, None);
        assert_ne!(
            h1, h2,
            "Different castling rights should give different hash"
        );
    }

    #[test]
    fn test_different_ep_different_hash() {
        let board = starting_position();
        let h1 = compute_full_hash(&board, Player::Red, 0xFF, None);
        let h2 = compute_full_hash(&board, Player::Red, 0xFF, Some(Square(31)));
        assert_ne!(h1, h2, "EP square should change hash");
    }

    #[test]
    fn test_incremental_matches_full() {
        let board = starting_position();
        let mut hash = compute_full_hash(&board, Player::Red, 0xFF, None);

        // Simulate changing side to move from Red to Blue
        toggle_side(&mut hash, Player::Red, Player::Blue);
        let expected = compute_full_hash(&board, Player::Blue, 0xFF, None);
        assert_eq!(
            hash, expected,
            "Incremental side toggle should match full recomputation"
        );
    }

    #[test]
    fn test_ep_toggle_roundtrip() {
        let mut hash = 12345u64;
        let original = hash;
        toggle_ep(&mut hash, None, Some(Square(31)));
        assert_ne!(hash, original);
        toggle_ep(&mut hash, Some(Square(31)), None);
        assert_eq!(hash, original);
    }
}
