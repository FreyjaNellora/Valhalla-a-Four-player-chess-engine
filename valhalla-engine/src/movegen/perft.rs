use super::legal::generate_legal_moves;
use crate::game_state::GameState;
use crate::types::{Move, MAX_MOVES};
/// Perft — performance test for move generation correctness.
/// Counts all leaf nodes at a given depth.
use arrayvec::ArrayVec;

/// Count all leaf positions at the given depth.
pub fn perft(state: &mut GameState, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
    generate_legal_moves(state, &mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut count = 0u64;
    for mv in moves {
        let undo = state.make_move(mv);
        count += perft(state, depth - 1);
        state.unmake_move(mv, undo);
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perft_1() {
        let mut state = GameState::new();
        assert_eq!(perft(&mut state, 1), 20, "perft(1) should be 20");
    }

    #[test]
    fn test_perft_2() {
        let mut state = GameState::new();
        assert_eq!(perft(&mut state, 2), 395, "perft(2) should be 395");
    }

    #[test]
    fn test_perft_3() {
        let mut state = GameState::new();
        assert_eq!(perft(&mut state, 3), 7_800, "perft(3) should be 7,800");
    }

    // perft(4) is slower — run with `cargo test -- --ignored` or in release mode
    #[test]
    #[ignore]
    fn test_perft_4() {
        let mut state = GameState::new();
        assert_eq!(perft(&mut state, 4), 152_050, "perft(4) should be 152,050");
    }
}
