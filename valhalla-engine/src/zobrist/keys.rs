/// Zobrist key tables for four-player chess.
/// Generated deterministically from a fixed seed using xorshift64.
use crate::types::{PLAYERS, TOTAL_SQUARES};

/// Number of piece types (Pawn..PromotedQueen = 7).
const PIECE_TYPES: usize = 7;

/// Piece-square keys: [player][piece_type][square].
pub static PIECE_SQUARE_KEYS: [[[u64; TOTAL_SQUARES]; PIECE_TYPES]; PLAYERS] = {
    let mut keys = [[[0u64; TOTAL_SQUARES]; PIECE_TYPES]; PLAYERS];
    let mut state = 0x4D595F56414C4841u64; // "MY_VALHA" as seed

    let mut p = 0;
    while p < PLAYERS {
        let mut pt = 0;
        while pt < PIECE_TYPES {
            let mut sq = 0;
            while sq < TOTAL_SQUARES {
                state = xorshift64(state);
                keys[p][pt][sq] = state;
                sq += 1;
            }
            pt += 1;
        }
        p += 1;
    }
    keys
};

/// Side-to-move keys (one per player).
pub static SIDE_TO_MOVE_KEYS: [u64; PLAYERS] = {
    let mut keys = [0u64; PLAYERS];
    let mut state = 0x5349_4445_4D4F_5645_u64; // "SIDEMOVE"

    let mut i = 0;
    while i < PLAYERS {
        state = xorshift64(state);
        keys[i] = state;
        i += 1;
    }
    keys
};

/// Castling right keys (one per bit, 8 total).
pub static CASTLING_KEYS: [u64; 8] = {
    let mut keys = [0u64; 8];
    let mut state = 0x4341_5354_4C49_4E47_u64; // "CASTLING"

    let mut i = 0;
    while i < 8 {
        state = xorshift64(state);
        keys[i] = state;
        i += 1;
    }
    keys
};

/// En passant square keys (one per square index).
pub static EP_SQUARE_KEYS: [u64; TOTAL_SQUARES] = {
    let mut keys = [0u64; TOTAL_SQUARES];
    let mut state = 0x454E_5041_5353_414E_u64; // "ENPASSAN"

    let mut i = 0;
    while i < TOTAL_SQUARES {
        state = xorshift64(state);
        keys[i] = state;
        i += 1;
    }
    keys
};

/// Deterministic xorshift64 PRNG.
const fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}
