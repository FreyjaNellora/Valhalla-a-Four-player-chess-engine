//! Observer module — self-play, game recording, and A/B testing.
//!
//! Rust-native implementation producing JSON game records.
//! Node.js WebSocket observer deferred to Phase 4.

pub mod self_play;

pub use self_play::{DuelConfig, DuelResult, GameConfig, GameRecord, MoveRecord};
