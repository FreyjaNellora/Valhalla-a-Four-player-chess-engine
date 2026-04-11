//! History heuristic — scores quiet moves by how often they caused beta cutoffs.
//!
//! Indexed by [from_square][to_square]. Updated by depth² on cutoff.
//! Fixed-size array (~150KB), no heap.

use crate::types::constants::TOTAL_SQUARES;

/// History bonus cap to prevent overflow.
const HISTORY_CAP: i32 = 400_000;

/// History table indexed by [from_index][to_index].
pub struct HistoryTable {
    scores: [[i32; TOTAL_SQUARES]; TOTAL_SQUARES],
}

impl HistoryTable {
    /// Create a new zeroed history table.
    pub fn new() -> Self {
        Self {
            scores: [[0; TOTAL_SQUARES]; TOTAL_SQUARES],
        }
    }

    /// Update history score for a move that caused a beta cutoff.
    /// Bonus = depth * depth (capped to prevent overflow).
    #[inline]
    pub fn update(&mut self, from: u8, to: u8, depth: u32) {
        let bonus = (depth * depth) as i32;
        let score = &mut self.scores[from as usize][to as usize];
        *score = (*score + bonus).min(HISTORY_CAP);
    }

    /// Get history score for a from-to pair.
    #[inline]
    pub fn get(&self, from: u8, to: u8) -> i32 {
        self.scores[from as usize][to as usize]
    }

    /// Age all scores by halving (gravity). Call between searches to prevent
    /// historical moves from dominating new positions.
    pub fn age_all(&mut self) {
        for row in self.scores.iter_mut() {
            for score in row.iter_mut() {
                *score /= 2;
            }
        }
    }

    /// Clear all scores (e.g., for a new game).
    pub fn clear(&mut self) {
        self.scores = [[0; TOTAL_SQUARES]; TOTAL_SQUARES];
    }
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_zero() {
        let ht = HistoryTable::new();
        assert_eq!(ht.get(31, 45), 0);
    }

    #[test]
    fn test_update_depth_squared() {
        let mut ht = HistoryTable::new();
        ht.update(31, 45, 4);
        assert_eq!(ht.get(31, 45), 16); // 4*4 = 16
    }

    #[test]
    fn test_accumulation() {
        let mut ht = HistoryTable::new();
        ht.update(31, 45, 4); // +16
        ht.update(31, 45, 8); // +64
        assert_eq!(ht.get(31, 45), 80);
    }

    #[test]
    fn test_cap() {
        let mut ht = HistoryTable::new();
        for _ in 0..100_000 {
            ht.update(31, 45, 12); // +144 each
        }
        assert!(ht.get(31, 45) <= super::HISTORY_CAP);
    }

    #[test]
    fn test_age_halves() {
        let mut ht = HistoryTable::new();
        ht.update(31, 45, 8); // 64
        ht.age_all();
        assert_eq!(ht.get(31, 45), 32);
    }

    #[test]
    fn test_clear() {
        let mut ht = HistoryTable::new();
        ht.update(31, 45, 8);
        ht.clear();
        assert_eq!(ht.get(31, 45), 0);
    }

    #[test]
    fn test_independent_pairs() {
        let mut ht = HistoryTable::new();
        ht.update(31, 45, 4);
        assert_eq!(ht.get(31, 45), 16);
        assert_eq!(ht.get(45, 31), 0); // reverse is independent
    }
}
