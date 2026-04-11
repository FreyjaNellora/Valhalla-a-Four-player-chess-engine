//! Influence map infrastructure — ray-attenuated per-player influence grids.
//!
//! Each piece projects influence along its movement vectors with blocker
//! attenuation. Output feeds swarm Layers 1-3 in Phase 3.

pub mod compute;

pub use compute::InfluenceMap;
