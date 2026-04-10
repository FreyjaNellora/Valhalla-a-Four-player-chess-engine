use super::repr::Board;
/// Starting position setup for four-player chess.
/// All square indices from 4PC_RULES_REFERENCE.md Section 3.
use crate::types::{ColoredPiece, PieceType, Player, Square};

/// Set up the standard starting position with all 64 pieces.
pub fn starting_position() -> Board {
    let mut board = Board::new();

    // Red (South) — back rank d1-k1, pawns d2-k2
    setup_red(&mut board);
    // Blue (West) — back file a4-a11, pawns b4-b11
    setup_blue(&mut board);
    // Yellow (North) — back rank d14-k14, pawns d13-k13
    setup_yellow(&mut board);
    // Green (East) — back file n4-n11, pawns m4-m11
    setup_green(&mut board);

    board
}

fn setup_red(board: &mut Board) {
    use PieceType::*;
    let p = Player::Red;

    // Back rank (rank 1, index 0): d1=3, e1=4, f1=5, g1=6, h1=7, i1=8, j1=9, k1=10
    // Order: R N B Q K B N R
    let back_rank = [
        (3, Rook),
        (4, Knight),
        (5, Bishop),
        (6, Queen),
        (7, King),
        (8, Bishop),
        (9, Knight),
        (10, Rook),
    ];
    for (idx, piece) in back_rank {
        board.place_piece(Square(idx), ColoredPiece::new(p, piece));
    }

    // Pawns (rank 2, index 1): d2=17, e2=18, ..., k2=24
    for file in 3..=10u8 {
        board.place_piece(Square(14 + file), ColoredPiece::new(p, Pawn));
    }
}

fn setup_blue(board: &mut Board) {
    use PieceType::*;
    let p = Player::Blue;

    // Back file (file a, index 0): a4=42, a5=56, a6=70, a7=84, a8=98, a9=112, a10=126, a11=140
    // Order: R N B K Q B N R (K/Q swapped vs Red)
    let back_file = [
        (42, Rook),
        (56, Knight),
        (70, Bishop),
        (84, King),
        (98, Queen),
        (112, Bishop),
        (126, Knight),
        (140, Rook),
    ];
    for (idx, piece) in back_file {
        board.place_piece(Square(idx), ColoredPiece::new(p, piece));
    }

    // Pawns (file b, index 1): b4=43, b5=57, ..., b11=141
    for rank in 3..=10u8 {
        board.place_piece(Square(rank * 14 + 1), ColoredPiece::new(p, Pawn));
    }
}

fn setup_yellow(board: &mut Board) {
    use PieceType::*;
    let p = Player::Yellow;

    // Back rank (rank 14, index 13): d14=185, e14=186, ..., k14=192
    // Order: R N B K Q B N R (K/Q swapped vs Red)
    let back_rank = [
        (185, Rook),
        (186, Knight),
        (187, Bishop),
        (188, King),
        (189, Queen),
        (190, Bishop),
        (191, Knight),
        (192, Rook),
    ];
    for (idx, piece) in back_rank {
        board.place_piece(Square(idx), ColoredPiece::new(p, piece));
    }

    // Pawns (rank 13, index 12): d13=171, e13=172, ..., k13=178
    for file in 3..=10u8 {
        board.place_piece(Square(12 * 14 + file), ColoredPiece::new(p, Pawn));
    }
}

fn setup_green(board: &mut Board) {
    use PieceType::*;
    let p = Player::Green;

    // Back file (file n, index 13): n4=55, n5=69, n6=83, n7=97, n8=111, n9=125, n10=139, n11=153
    // Order: R N B Q K B N R (same as Red)
    let back_file = [
        (55, Rook),
        (69, Knight),
        (83, Bishop),
        (97, Queen),
        (111, King),
        (125, Bishop),
        (139, Knight),
        (153, Rook),
    ];
    for (idx, piece) in back_file {
        board.place_piece(Square(idx), ColoredPiece::new(p, piece));
    }

    // Pawns (file m, index 12): m4=54, m5=68, ..., m11=152
    for rank in 3..=10u8 {
        board.place_piece(Square(rank * 14 + 12), ColoredPiece::new(p, Pawn));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ALL_PLAYERS, IS_VALID};

    #[test]
    fn test_total_pieces() {
        let board = starting_position();
        assert_eq!(board.total_pieces(), 64, "Should have 64 pieces total");
    }

    #[test]
    fn test_pieces_per_player() {
        let board = starting_position();
        for player in ALL_PLAYERS {
            assert_eq!(
                board.piece_count[player.index()],
                16,
                "Player {:?} should have 16 pieces",
                player
            );
        }
    }

    #[test]
    fn test_red_king_square() {
        let board = starting_position();
        assert_eq!(board.king_square(Player::Red).unwrap().index(), 7); // h1
    }

    #[test]
    fn test_blue_king_square() {
        let board = starting_position();
        assert_eq!(board.king_square(Player::Blue).unwrap().index(), 84); // a7
    }

    #[test]
    fn test_yellow_king_square() {
        let board = starting_position();
        assert_eq!(board.king_square(Player::Yellow).unwrap().index(), 188); // g14
    }

    #[test]
    fn test_green_king_square() {
        let board = starting_position();
        assert_eq!(board.king_square(Player::Green).unwrap().index(), 111); // n8
    }

    #[test]
    fn test_no_pieces_on_invalid_squares() {
        let board = starting_position();
        for (i, valid) in IS_VALID.iter().enumerate() {
            if !valid {
                assert!(
                    board.squares[i].is_none(),
                    "Invalid square {} has a piece",
                    i
                );
            }
        }
    }

    #[test]
    fn test_red_starting_pieces() {
        let board = starting_position();
        // Back rank: d1=R, e1=N, f1=B, g1=Q, h1=K, i1=B, j1=N, k1=R
        let expected = [
            (3, PieceType::Rook),
            (4, PieceType::Knight),
            (5, PieceType::Bishop),
            (6, PieceType::Queen),
            (7, PieceType::King),
            (8, PieceType::Bishop),
            (9, PieceType::Knight),
            (10, PieceType::Rook),
        ];
        for (idx, piece_type) in expected {
            let cp = board.get(Square(idx)).expect("Expected piece");
            assert_eq!(cp.player, Player::Red);
            assert_eq!(cp.piece_type, piece_type, "Wrong piece at index {}", idx);
        }
        // Pawns on rank 2
        for file in 3..=10u8 {
            let sq = Square(14 + file);
            let cp = board.get(sq).expect("Expected pawn");
            assert_eq!(cp.player, Player::Red);
            assert_eq!(cp.piece_type, PieceType::Pawn);
        }
    }

    #[test]
    fn test_blue_starting_pieces() {
        let board = starting_position();
        let expected = [
            (42, PieceType::Rook),
            (56, PieceType::Knight),
            (70, PieceType::Bishop),
            (84, PieceType::King),
            (98, PieceType::Queen),
            (112, PieceType::Bishop),
            (126, PieceType::Knight),
            (140, PieceType::Rook),
        ];
        for (idx, piece_type) in expected {
            let cp = board.get(Square(idx)).expect("Expected piece");
            assert_eq!(cp.player, Player::Blue);
            assert_eq!(cp.piece_type, piece_type, "Wrong piece at index {}", idx);
        }
    }

    #[test]
    fn test_yellow_starting_pieces() {
        let board = starting_position();
        let expected = [
            (185, PieceType::Rook),
            (186, PieceType::Knight),
            (187, PieceType::Bishop),
            (188, PieceType::King),
            (189, PieceType::Queen),
            (190, PieceType::Bishop),
            (191, PieceType::Knight),
            (192, PieceType::Rook),
        ];
        for (idx, piece_type) in expected {
            let cp = board.get(Square(idx)).expect("Expected piece");
            assert_eq!(cp.player, Player::Yellow);
            assert_eq!(cp.piece_type, piece_type, "Wrong piece at index {}", idx);
        }
    }

    #[test]
    fn test_green_starting_pieces() {
        let board = starting_position();
        let expected = [
            (55, PieceType::Rook),
            (69, PieceType::Knight),
            (83, PieceType::Bishop),
            (97, PieceType::Queen),
            (111, PieceType::King),
            (125, PieceType::Bishop),
            (139, PieceType::Knight),
            (153, PieceType::Rook),
        ];
        for (idx, piece_type) in expected {
            let cp = board.get(Square(idx)).expect("Expected piece");
            assert_eq!(cp.player, Player::Green);
            assert_eq!(cp.piece_type, piece_type, "Wrong piece at index {}", idx);
        }
    }

    #[test]
    fn test_piece_list_consistency() {
        let board = starting_position();
        for player in ALL_PLAYERS {
            let count = board.piece_count[player.index()] as usize;
            for i in 0..count {
                let sq = board.piece_list[player.index()][i];
                let piece = board.get(sq).expect("Piece list points to empty square");
                assert_eq!(piece.player, player, "Piece list has wrong player");
            }
        }
    }

    #[test]
    fn test_remove_and_place() {
        let mut board = starting_position();
        let sq = Square(7); // h1 (Red King)
        let piece = board.remove_piece(sq).unwrap();
        assert_eq!(piece.piece_type, PieceType::King);
        assert!(board.get(sq).is_none());
        assert_eq!(board.piece_count[Player::Red.index()], 15);

        board.place_piece(sq, piece);
        assert_eq!(board.get(sq).unwrap().piece_type, PieceType::King);
        assert_eq!(board.piece_count[Player::Red.index()], 16);
    }

    #[test]
    fn test_move_piece() {
        let mut board = starting_position();
        // Move Red pawn d2 to d3
        let from = Square(17); // d2
        let to = Square(31); // d3
        board.move_piece(from, to);
        assert!(board.get(from).is_none());
        let piece = board.get(to).unwrap();
        assert_eq!(piece.player, Player::Red);
        assert_eq!(piece.piece_type, PieceType::Pawn);
    }
}
