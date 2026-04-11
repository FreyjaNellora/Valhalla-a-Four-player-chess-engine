//! Transposition table — hash-indexed position cache for search.
//!
//! Allocated once at construction (Vec), never resized during search.
//! Depth-4 aware: only stores entries at depths divisible by 4.

use crate::types::{Move, Score};

/// Node type for TT entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TTFlag {
    /// Exact score (PV node).
    Exact,
    /// Score is a lower bound (beta cutoff).
    LowerBound,
    /// Score is an upper bound (all-node).
    UpperBound,
}

/// A single transposition table entry.
#[derive(Clone, Copy)]
pub struct TTEntry {
    /// Full Zobrist hash for collision detection.
    pub hash: u64,
    /// Best move found at this position.
    pub best_move: Move,
    /// Evaluation score.
    pub score: Score,
    /// Search depth (always divisible by 4 for stored entries).
    pub depth: u32,
    /// Node type.
    pub flag: TTFlag,
    /// Search generation for replacement policy.
    pub age: u16,
}

impl TTEntry {
    /// Empty/invalid entry sentinel.
    const EMPTY: Self = Self {
        hash: 0,
        best_move: Move::NULL,
        score: 0,
        depth: 0,
        flag: TTFlag::Exact,
        age: 0,
    };
}

/// Hash-indexed transposition table.
///
/// Uses power-of-2 sizing with mask-based indexing.
/// Replacement: deeper entry wins; ties broken by age (newer wins).
pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    mask: usize,
    age: u16,
}

impl TranspositionTable {
    /// Create a new TT with the given size in megabytes.
    /// Rounds down to the nearest power of 2 number of entries.
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let total_entries = (size_mb * 1024 * 1024) / entry_size;
        // Round down to power of 2
        let capacity = if total_entries == 0 {
            1
        } else {
            1 << (usize::BITS - 1 - total_entries.leading_zeros())
        };
        Self {
            entries: vec![TTEntry::EMPTY; capacity],
            mask: capacity - 1,
            age: 0,
        }
    }

    /// Index into the table from a hash.
    #[inline]
    fn index(&self, hash: u64) -> usize {
        (hash as usize) & self.mask
    }

    /// Probe the table for a matching entry.
    ///
    /// Returns the entry if the hash matches and the stored depth is
    /// at least `min_depth`. Returns `None` on miss or insufficient depth.
    pub fn probe(&self, hash: u64, min_depth: u32) -> Option<&TTEntry> {
        let entry = &self.entries[self.index(hash)];
        if entry.hash == hash && entry.depth >= min_depth {
            Some(entry)
        } else {
            None
        }
    }

    /// Probe for just the best move hint (ignores depth requirement).
    pub fn probe_move(&self, hash: u64) -> Option<Move> {
        let entry = &self.entries[self.index(hash)];
        if entry.hash == hash && !entry.best_move.is_null() {
            Some(entry.best_move)
        } else {
            None
        }
    }

    /// Store an entry. Depth-4 enforcement: panics if depth is not divisible by 4.
    ///
    /// Replacement policy: always replace if new entry has greater or equal depth,
    /// or if the existing entry is from an older search generation.
    pub fn store(&mut self, hash: u64, best_move: Move, score: Score, depth: u32, flag: TTFlag) {
        assert!(
            depth.is_multiple_of(4),
            "TT store: depth {} is not divisible by 4",
            depth
        );

        let idx = self.index(hash);
        let existing = &self.entries[idx];

        // Replace if: empty, deeper or equal depth, or older generation
        let should_replace =
            existing.hash == 0 || depth >= existing.depth || existing.age != self.age;

        if should_replace {
            self.entries[idx] = TTEntry {
                hash,
                best_move,
                score,
                depth,
                flag,
                age: self.age,
            };
        }
    }

    /// Clear all entries (e.g., for a new game).
    pub fn clear(&mut self) {
        self.entries.fill(TTEntry::EMPTY);
        self.age = 0;
    }

    /// Increment age for a new search. Old entries become replaceable.
    pub fn new_search(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    /// Number of entries (capacity).
    pub fn capacity(&self) -> usize {
        self.entries.len()
    }

    /// Approximate fill rate (samples first 1000 entries).
    pub fn occupancy(&self) -> f64 {
        let sample = self.entries.len().min(1000);
        let filled = self.entries[..sample]
            .iter()
            .filter(|e| e.hash != 0)
            .count();
        filled as f64 / sample as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Move, PieceType, Square};

    fn test_move() -> Move {
        Move::quiet(Square(31), Square(45), PieceType::Knight)
    }

    #[test]
    fn test_new_table() {
        let tt = TranspositionTable::new(1); // 1MB
        assert!(tt.capacity() > 0);
        // Power of 2
        assert_eq!(tt.capacity() & (tt.capacity() - 1), 0);
    }

    #[test]
    fn test_store_and_probe() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0xDEAD_BEEF_1234_5678u64;
        tt.store(hash, test_move(), 150, 4, TTFlag::Exact);

        let entry = tt.probe(hash, 4).expect("should find entry");
        assert_eq!(entry.hash, hash);
        assert_eq!(entry.score, 150);
        assert_eq!(entry.depth, 4);
        assert_eq!(entry.flag, TTFlag::Exact);
        assert_eq!(entry.best_move, test_move());
    }

    #[test]
    fn test_probe_miss_wrong_hash() {
        let mut tt = TranspositionTable::new(1);
        tt.store(0x1111, test_move(), 100, 4, TTFlag::Exact);
        assert!(tt.probe(0x2222, 4).is_none());
    }

    #[test]
    fn test_probe_miss_insufficient_depth() {
        let mut tt = TranspositionTable::new(1);
        tt.store(0x1111, test_move(), 100, 4, TTFlag::Exact);
        // Requesting depth 8, but stored at depth 4
        assert!(tt.probe(0x1111, 8).is_none());
    }

    #[test]
    fn test_probe_move_ignores_depth() {
        let mut tt = TranspositionTable::new(1);
        let mv = test_move();
        tt.store(0x1111, mv, 100, 4, TTFlag::Exact);
        // probe_move doesn't check depth
        assert_eq!(tt.probe_move(0x1111), Some(mv));
    }

    #[test]
    fn test_deeper_replaces_shallower() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1111;
        tt.store(hash, test_move(), 100, 4, TTFlag::Exact);
        tt.store(hash, test_move(), 200, 8, TTFlag::Exact);
        let entry = tt.probe(hash, 4).unwrap();
        assert_eq!(entry.score, 200);
        assert_eq!(entry.depth, 8);
    }

    #[test]
    fn test_shallower_does_not_replace_deeper() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1111;
        tt.store(hash, test_move(), 200, 8, TTFlag::Exact);
        // Same hash, lower depth, same age — should NOT replace
        tt.store(hash, test_move(), 100, 4, TTFlag::Exact);
        let entry = tt.probe(hash, 4).unwrap();
        assert_eq!(entry.score, 200);
        assert_eq!(entry.depth, 8);
    }

    #[test]
    fn test_new_search_allows_replacement() {
        let mut tt = TranspositionTable::new(1);
        let hash = 0x1111;
        tt.store(hash, test_move(), 200, 8, TTFlag::Exact);
        tt.new_search();
        // After age increment, shallower entry replaces older deep entry
        tt.store(hash, test_move(), 100, 4, TTFlag::Exact);
        let entry = tt.probe(hash, 4).unwrap();
        assert_eq!(entry.score, 100);
    }

    #[test]
    fn test_clear() {
        let mut tt = TranspositionTable::new(1);
        tt.store(0x1111, test_move(), 100, 4, TTFlag::Exact);
        tt.clear();
        assert!(tt.probe(0x1111, 4).is_none());
    }

    #[test]
    #[should_panic(expected = "not divisible by 4")]
    fn test_depth_4_enforcement() {
        let mut tt = TranspositionTable::new(1);
        tt.store(0x1111, test_move(), 100, 3, TTFlag::Exact);
    }
}
