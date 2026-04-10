pub mod fen4;
pub mod make_unmake;
/// Game state module — position state, make/unmake, FEN4.
pub mod state;

pub use state::{GameMode, GameState, PlayerStatus};
