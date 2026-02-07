//! Last-Write-Wins (LWW) Register CRDT
//!
//! The LWW Register always retains the value with the highest timestamp.
//! In case of a tie, the replica with the highest ID wins.
//!
//! This is a simple eventual-consistency mechanism that resolves concurrent
//! writes by always choosing the "latest" update based on timestamp and
//! replica ordering.

use crate::lattice::Lattice;
use serde::{Deserialize, Serialize};

/// A Last-Write-Wins Register CRDT
///
/// Stores a value along with a timestamp and replica ID.
/// The value with the highest timestamp (tie-break on replica_id) always wins.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LWWRegister<T: Ord + Clone, K: Ord + Clone> {
    /// The current value
    value: Option<T>,
    /// The timestamp of the last write
    timestamp: u64,
    /// The replica ID that wrote this value (for tie-breaking)
    replica_id: K,
}

impl<T: Ord + Clone, K: Ord + Clone> LWWRegister<T, K> {
    /// Create a new LWW Register with no value
    pub fn new(replica_id: K) -> Self {
        Self {
            value: None,
            timestamp: 0,
            replica_id,
        }
    }

    /// Set a new value with the given timestamp
    pub fn set(&mut self, value: T, timestamp: u64, replica_id: K) {
        if timestamp > self.timestamp
            || (timestamp == self.timestamp && replica_id >= self.replica_id)
        {
            self.value = Some(value);
            self.timestamp = timestamp;
            self.replica_id = replica_id;
        }
    }

    /// Get the current value if it exists
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Get the timestamp of the current value
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Get the replica ID that wrote the current value
    pub fn replica_id(&self) -> &K {
        &self.replica_id
    }

    /// Check if the register is empty (no value set)
    pub fn is_empty(&self) -> bool {
        self.value.is_none()
    }

    /// Clear the register (set to empty state)
    pub fn clear(&mut self) {
        self.value = None;
        self.timestamp = 0;
    }
}

impl<T: Ord + Clone, K: Ord + Clone + Default> Default for LWWRegister<T, K> {
    fn default() -> Self {
        Self::new(K::default())
    }
}

impl<T: Ord + Clone, K: Ord + Clone + Default> Lattice for LWWRegister<T, K> {
    fn bottom() -> Self {
        Self {
            value: None,
            timestamp: 0,
            replica_id: K::default(),
        }
    }

    /// Join operation: keep the value with the highest timestamp
    /// Tie-break on replica_id (higher wins), then on value (higher wins)
    fn join(&self, other: &Self) -> Self {
        // Compare by (timestamp, replica_id, value) tuple
        let self_wins = match self.timestamp.cmp(&other.timestamp) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => {
                match self.replica_id.cmp(&other.replica_id) {
                    std::cmp::Ordering::Greater => true,
                    std::cmp::Ordering::Less => false,
                    std::cmp::Ordering::Equal => {
                        // Same timestamp and replica_id: compare values for determinism
                        self.value >= other.value
                    }
                }
            }
        };

        if self_wins {
            self.clone()
        } else {
            other.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lwwreg_basic_operations() {
        let mut reg: LWWRegister<i32, String> = LWWRegister::new("replica1".to_string());

        assert!(reg.is_empty());
        assert_eq!(reg.get(), None);

        // Set a value
        reg.set(42, 100, "replica1".to_string());
        assert_eq!(reg.get(), Some(&42));
        assert_eq!(reg.timestamp(), 100);
    }

    #[test]
    fn test_lwwreg_higher_timestamp_wins() {
        let mut reg: LWWRegister<i32, String> = LWWRegister::new("replica1".to_string());

        reg.set(10, 100, "replica1".to_string());
        assert_eq!(reg.get(), Some(&10));

        reg.set(20, 200, "replica2".to_string());
        assert_eq!(reg.get(), Some(&20));

        // Old timestamp doesn't overwrite
        reg.set(30, 150, "replica1".to_string());
        assert_eq!(reg.get(), Some(&20));
    }

    #[test]
    fn test_lwwreg_tie_break_replica_id() {
        let mut reg: LWWRegister<i32, String> = LWWRegister::new("replica1".to_string());

        // Same timestamp, different replicas
        reg.set(10, 100, "replica1".to_string());
        assert_eq!(reg.get(), Some(&10));

        // Replica with higher ID wins on tie
        reg.set(20, 100, "replica2".to_string());
        assert_eq!(reg.get(), Some(&20));

        // Lower replica ID doesn't overwrite
        reg.set(30, 100, "replica1".to_string());
        assert_eq!(reg.get(), Some(&20));
    }

    #[test]
    fn test_lwwreg_join_idempotent() {
        let mut reg1: LWWRegister<i32, String> = LWWRegister::new("replica1".to_string());
        reg1.set(42, 100, "replica1".to_string());

        let joined = reg1.join(&reg1);
        assert_eq!(joined.get(), Some(&42));
        assert_eq!(joined.timestamp(), 100);
    }

    #[test]
    fn test_lwwreg_join_commutative() {
        let mut reg1: LWWRegister<i32, String> = LWWRegister::new("replica1".to_string());
        reg1.set(10, 100, "replica1".to_string());

        let mut reg2: LWWRegister<i32, String> = LWWRegister::new("replica2".to_string());
        reg2.set(20, 150, "replica2".to_string());

        let joined1 = reg1.join(&reg2);
        let joined2 = reg2.join(&reg1);

        assert_eq!(joined1.get(), joined2.get());
        assert_eq!(joined1.timestamp(), joined2.timestamp());
    }

    #[test]
    fn test_lwwreg_join_associative() {
        let mut reg1: LWWRegister<i32, String> = LWWRegister::new("replica1".to_string());
        reg1.set(10, 100, "replica1".to_string());

        let mut reg2: LWWRegister<i32, String> = LWWRegister::new("replica2".to_string());
        reg2.set(20, 150, "replica2".to_string());

        let mut reg3: LWWRegister<i32, String> = LWWRegister::new("replica3".to_string());
        reg3.set(30, 120, "replica3".to_string());

        let left = reg1.join(&reg2).join(&reg3);
        let right = reg1.join(&reg2.join(&reg3));

        assert_eq!(left.get(), right.get());
        assert_eq!(left.timestamp(), right.timestamp());
    }

    #[test]
    fn test_lwwreg_bottom_is_identity() {
        let mut reg: LWWRegister<i32, String> = LWWRegister::new("replica1".to_string());
        reg.set(42, 100, "replica1".to_string());

        let bottom = LWWRegister::bottom();
        let joined = reg.join(&bottom);

        assert_eq!(joined.get(), reg.get());
        assert_eq!(joined.timestamp(), reg.timestamp());
    }

    #[test]
    fn test_lwwreg_serialization() {
        let mut reg: LWWRegister<i32, String> = LWWRegister::new("replica1".to_string());
        reg.set(42, 100, "replica1".to_string());

        let serialized = serde_json::to_string(&reg).unwrap();
        let deserialized: LWWRegister<i32, String> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.get(), Some(&42));
        assert_eq!(deserialized.timestamp(), 100);
    }
}
