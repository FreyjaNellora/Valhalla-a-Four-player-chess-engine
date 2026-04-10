/// Move generation module — attack detection, pseudo-legal and legal movegen.
pub mod attack;
pub mod castling;
pub mod legal;
pub mod pawns;
pub mod perft;
pub mod pieces;

pub use attack::{attackers_of, is_in_check, is_square_attacked_by};
pub use castling::{castling_bit, get_castle_rook_squares, ROOK_START_SQUARES};
pub use legal::generate_legal_moves;
