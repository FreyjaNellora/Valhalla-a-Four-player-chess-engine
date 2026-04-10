/// Dead King Walking module.
pub mod rules;

pub use rules::{
    eliminate_player_dkw, eliminate_player_full, generate_dkw_king_move, process_dkw_moves,
};
