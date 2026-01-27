//! Delta buffer for grouping and batching deltas
//! Implements Algorithm 1 from the δ-CRDT paper (convergence mode)
//!
//! # Algorithm 1: δ-CRDT Anti-Entropy (Convergence Mode)
//!
//! The algorithm maintains:
//! - A local state X
//! - A delta buffer D
//! - Sequence numbers for causal ordering
//!
//! On local mutation m:
//!   d = mδ(X)          // compute delta
//!   X = X ⊔ d          // apply to state
//!   D = D ⊔ d          // buffer delta
//!
//! On send to peer j:
//!   send D[acked[j]..] to j
//!
//! On receive delta d from peer i:
//!   X = X ⊔ d          // apply (idempotent!)
//!   ack to i

use mdcs_core::lattice::Lattice;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

/// Sequence number for delta intervals
pub type SeqNo = u64;

/// Replica identifier
pub type ReplicaId = String;

/// A delta tagged with sequence information for causal ordering
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaggedDelta<D> {
    pub seq: SeqNo,
    pub delta: D,
}

/// Buffer for outgoing deltas with grouping support
#[derive(Debug, Clone)]
pub struct DeltaBuffer<D: Lattice> {
    /// Current sequence number
    current_seq: SeqNo,
    /// Buffered deltas awaiting acknowledgment
    deltas: VecDeque<TaggedDelta<D>>,
    /// Maximum deltas to buffer before forcing group-join
    max_buffer_size: usize,
}

impl<D: Lattice> DeltaBuffer<D> {
    pub fn new(max_buffer_size: usize) -> Self {
        Self {
            current_seq: 0,
            deltas: VecDeque::new(),
            max_buffer_size,
        }
    }

    /// Add a new delta to the buffer
    pub fn push(&mut self, delta: D) {
        self.current_seq += 1;
        self.deltas.push_back(TaggedDelta {
            seq: self.current_seq,
            delta,
        });

        // If buffer is full, compact by joining older deltas
        if self.deltas.len() > self.max_buffer_size {
            self.compact_oldest();
        }
    }

    /// Get deltas for sending to a peer that has acked up to `acked_seq`
    pub fn deltas_since(&self, acked_seq: SeqNo) -> Vec<&TaggedDelta<D>> {
        self.deltas
            .iter()
            .filter(|td| td.seq > acked_seq)
            .collect()
    }

    /// Create a delta-group (joined deltas) for a peer
    pub fn delta_group_since(&self, acked_seq: SeqNo) -> Option<D> {
        let deltas: Vec<_> = self.deltas_since(acked_seq);
        if deltas.is_empty() {
            return None;
        }

        let mut group = D::bottom();
        for td in deltas {
            group.join_assign(&td.delta);
        }
        Some(group)
    }

    /// Acknowledge that a peer has received up to `seq`
    /// Deltas before this can be GC'd if all peers have acked
    pub fn ack(&mut self, acked_seq: SeqNo) -> usize {
        let initial_len = self.deltas.len();
        self.deltas.retain(|td| td.seq > acked_seq);
        initial_len - self.deltas.len()
    }

    /// Current sequence number
    pub fn current_seq(&self) -> SeqNo {
        self.current_seq
    }

    /// Number of buffered deltas
    pub fn len(&self) -> usize {
        self.deltas.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }

    /// Clear all buffered deltas
    pub fn clear(&mut self) {
        self.deltas.clear();
    }

    /// Compact oldest deltas by joining them
    fn compact_oldest(&mut self) {
        if self.deltas.len() < 2 {
            return;
        }

        // Join the two oldest deltas
        let oldest = self.deltas.pop_front().unwrap();
        if let Some(second) = self.deltas.front_mut() {
            second.delta = oldest.delta.join(&second.delta);
        }
    }
}

/// Tracks acknowledgments from peers for garbage collection
#[derive(Debug, Clone)]
pub struct AckTracker {
    /// Maps peer_id -> last acked sequence number
    acked: BTreeMap<ReplicaId, SeqNo>,
}

impl AckTracker {
    pub fn new() -> Self {
        Self {
            acked: BTreeMap::new(),
        }
    }

    /// Register a peer (initializes ack to 0)
    pub fn register_peer(&mut self, peer_id: ReplicaId) {
        self.acked.entry(peer_id).or_insert(0);
    }

    /// Update the ack for a peer
    pub fn update_ack(&mut self, peer_id: &str, seq: SeqNo) {
        if let Some(acked) = self.acked.get_mut(peer_id) {
            *acked = (*acked).max(seq);
        }
    }

    /// Get the ack for a peer
    pub fn get_ack(&self, peer_id: &str) -> SeqNo {
        self.acked.get(peer_id).copied().unwrap_or(0)
    }

    /// Get minimum acked sequence across all peers (safe to GC before this)
    pub fn min_acked(&self) -> SeqNo {
        self.acked.values().copied().min().unwrap_or(0)
    }

    /// Get all registered peers
    pub fn peers(&self) -> impl Iterator<Item = &ReplicaId> {
        self.acked.keys()
    }
}

impl Default for AckTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// A delta-CRDT replica implementing Algorithm 1
#[derive(Debug, Clone)]
pub struct DeltaReplica<S: Lattice, D: Lattice = S> {
    /// Replica identifier
    pub id: ReplicaId,
    /// Current state
    state: S,
    /// Delta buffer for outgoing deltas
    buffer: DeltaBuffer<D>,
    /// Ack tracker for peers
    acks: AckTracker,
    /// Function to convert state delta to buffer delta (usually identity or subset)
    _phantom: std::marker::PhantomData<D>,
}

impl<S: Lattice, D: Lattice> DeltaReplica<S, D> {
    /// Create a new replica with default buffer size
    pub fn new(id: impl Into<ReplicaId>) -> Self {
        Self::with_buffer_size(id, 100)
    }

    /// Create a new replica with specified buffer size
    pub fn with_buffer_size(id: impl Into<ReplicaId>, buffer_size: usize) -> Self {
        Self {
            id: id.into(),
            state: S::bottom(),
            buffer: DeltaBuffer::new(buffer_size),
            acks: AckTracker::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get current state (read-only)
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Get mutable access to buffer
    pub fn buffer(&self) -> &DeltaBuffer<D> {
        &self.buffer
    }

    /// Register a peer for anti-entropy
    pub fn register_peer(&mut self, peer_id: ReplicaId) {
        self.acks.register_peer(peer_id);
    }

    /// Current sequence number
    pub fn current_seq(&self) -> SeqNo {
        self.buffer.current_seq()
    }
}

/// Delta-CRDT replica where state and delta are the same type
impl<S: Lattice + Clone> DeltaReplica<S, S> {
    /// Apply a delta-mutator: computes delta, applies to state, buffers delta
    /// Returns the computed delta
    pub fn mutate<F>(&mut self, mutator: F) -> S
    where
        F: FnOnce(&S) -> S,
    {
        // Compute delta: d = mδ(X)
        let delta = mutator(&self.state);

        // Apply to state: X = X ⊔ d
        self.state.join_assign(&delta);

        // Buffer delta: D = D ⊔ d
        self.buffer.push(delta.clone());

        delta
    }

    /// Get delta-group to send to a peer
    pub fn prepare_sync(&self, peer_id: &str) -> Option<(S, SeqNo)> {
        let acked = self.acks.get_ack(peer_id);
        self.buffer.delta_group_since(acked).map(|d| (d, self.buffer.current_seq()))
    }

    /// Receive and apply a delta from a peer (idempotent!)
    pub fn receive_delta(&mut self, delta: &S) {
        // X = X ⊔ d (idempotent merge)
        self.state.join_assign(delta);
    }

    /// Process an ack from a peer
    pub fn process_ack(&mut self, peer_id: &str, seq: SeqNo) {
        self.acks.update_ack(peer_id, seq);

        // GC: remove deltas that all peers have acked
        let min_acked = self.acks.min_acked();
        self.buffer.ack(min_acked);
    }

    /// Full state (for initial sync or recovery)
    pub fn full_state(&self) -> &S {
        &self.state
    }

    /// Sync with another replica directly (for testing/simulation)
    pub fn sync_with(&mut self, other: &mut DeltaReplica<S, S>) {
        // Exchange full states (simulates delta exchange converging to full state)
        let my_state = self.state.clone();
        let their_state = other.state.clone();

        self.receive_delta(&their_state);
        other.receive_delta(&my_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdcs_core::gset::GSet;

    #[test]
    fn test_delta_buffer_basic() {
        let mut buffer: DeltaBuffer<GSet<i32>> = DeltaBuffer::new(10);

        let mut delta1 = GSet::new();
        delta1.insert(1);
        buffer.push(delta1);

        assert_eq!(buffer.current_seq(), 1);
        assert_eq!(buffer.len(), 1);

        let mut delta2 = GSet::new();
        delta2.insert(2);
        buffer.push(delta2);

        assert_eq!(buffer.current_seq(), 2);
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn test_delta_buffer_group() {
        let mut buffer: DeltaBuffer<GSet<i32>> = DeltaBuffer::new(10);

        for i in 1..=5 {
            let mut delta = GSet::new();
            delta.insert(i);
            buffer.push(delta);
        }

        // Get group from seq 2 onwards
        let group = buffer.delta_group_since(2).unwrap();
        assert!(!group.contains(&1));
        assert!(!group.contains(&2));
        assert!(group.contains(&3));
        assert!(group.contains(&4));
        assert!(group.contains(&5));
    }

    #[test]
    fn test_delta_buffer_ack() {
        let mut buffer: DeltaBuffer<GSet<i32>> = DeltaBuffer::new(10);

        for i in 1..=5 {
            let mut delta = GSet::new();
            delta.insert(i);
            buffer.push(delta);
        }

        assert_eq!(buffer.len(), 5);

        // Ack up to seq 3
        let removed = buffer.ack(3);
        assert_eq!(removed, 3);
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn test_delta_buffer_compaction() {
        let mut buffer: DeltaBuffer<GSet<i32>> = DeltaBuffer::new(3);

        for i in 1..=5 {
            let mut delta = GSet::new();
            delta.insert(i);
            buffer.push(delta);
        }

        // Should have compacted to stay within bounds
        assert!(buffer.len() <= 3);

        // But all elements should still be reachable via group
        let group = buffer.delta_group_since(0).unwrap();
        for i in 1..=5 {
            assert!(group.contains(&i));
        }
    }

    #[test]
    fn test_ack_tracker() {
        let mut tracker = AckTracker::new();

        tracker.register_peer("peer1".to_string());
        tracker.register_peer("peer2".to_string());

        assert_eq!(tracker.get_ack("peer1"), 0);
        assert_eq!(tracker.get_ack("peer2"), 0);

        tracker.update_ack("peer1", 5);
        assert_eq!(tracker.get_ack("peer1"), 5);
        assert_eq!(tracker.min_acked(), 0); // peer2 still at 0

        tracker.update_ack("peer2", 3);
        assert_eq!(tracker.min_acked(), 3);

        tracker.update_ack("peer2", 7);
        assert_eq!(tracker.min_acked(), 5);
    }

    #[test]
    fn test_delta_replica_basic() {
        let mut replica: DeltaReplica<GSet<i32>> = DeltaReplica::new("replica1");

        // Mutate using delta-mutator
        replica.mutate(|_state| {
            let mut delta = GSet::new();
            delta.insert(42);
            delta
        });

        assert!(replica.state().contains(&42));
        assert_eq!(replica.current_seq(), 1);
    }

    #[test]
    fn test_delta_replica_sync() {
        let mut replica1: DeltaReplica<GSet<i32>> = DeltaReplica::new("r1");
        let mut replica2: DeltaReplica<GSet<i32>> = DeltaReplica::new("r2");

        replica1.mutate(|_| {
            let mut d = GSet::new();
            d.insert(1);
            d
        });

        replica2.mutate(|_| {
            let mut d = GSet::new();
            d.insert(2);
            d
        });

        // Before sync
        assert!(replica1.state().contains(&1));
        assert!(!replica1.state().contains(&2));
        assert!(!replica2.state().contains(&1));
        assert!(replica2.state().contains(&2));

        // Sync
        replica1.sync_with(&mut replica2);

        // After sync - both should have both elements
        assert!(replica1.state().contains(&1));
        assert!(replica1.state().contains(&2));
        assert!(replica2.state().contains(&1));
        assert!(replica2.state().contains(&2));
    }
}
