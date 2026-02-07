//! Causal Consistency Mode for δ-CRDTs (Algorithm 2)
//!
//! This module implements the delta-interval anti-entropy algorithm that provides
//! **causal consistency** guarantees, extending Algorithm 1's convergence mode.
//!
//! # Algorithm 2: δ-CRDT Anti-Entropy with Causal Delivery
//!
//! Unlike Algorithm 1 which only guarantees eventual convergence, Algorithm 2
//! ensures that deltas are applied in causal order. This prevents seeing effects
//! before their causes.
//!
//! ## State Components
//!
//! Each replica i maintains:
//! - **Durable state** `(Xᵢ, cᵢ)`:
//!   - `Xᵢ`: The current CRDT state
//!   - `cᵢ`: A durable counter (sequence number) that survives crashes
//!
//! - **Volatile state** `(Dᵢ, Aᵢ)`:
//!   - `Dᵢ[j]`: Delta-interval buffer for each peer j (deltas to send)
//!   - `Aᵢ[j]`: Acknowledgment map - last seq acked by peer j
//!
//! ## Protocol
//!
//! 1. **On local mutation m**:
//!    ```text
//!    cᵢ := cᵢ + 1
//!    d := mδ(Xᵢ)
//!    Xᵢ := Xᵢ ⊔ d
//!    ∀j: Dᵢ[j] := Dᵢ[j] ⊔ d   // add delta to all peer buffers
//!    ```
//!
//! 2. **On send to peer j** (periodic or on-demand):
//!    ```text
//!    if Dᵢ[j] ≠ ⊥ then
//!        send ⟨Dᵢ[j], Aᵢ[j]+1, cᵢ⟩ to j   // delta-interval with seq range
//!    ```
//!
//! 3. **On receive `⟨d, n, m⟩` from peer j**:
//!    ```text
//!    if n = Aᵢ[j] + 1 then        // causally ready
//!        Xᵢ := Xᵢ ⊔ d
//!        Aᵢ[j] := m
//!        send ack(m) to j
//!    else
//!        discard (or buffer for later)
//!    ```
//!
//! 4. **On receive ack(m) from peer j**:
//!    ```text
//!    Dᵢ[j] := ⊥                   // clear delta buffer for j
//!    ```
//!
//! ## Garbage Collection
//!
//! Deltas can be safely garbage collected when ALL tracked peers have acknowledged them.
//! This ensures no peer will ever need those deltas again.
//!
//! ## Crash Recovery
//!
//! On restart:
//! - `Xᵢ` and `cᵢ` are restored from durable storage
//! - `Dᵢ` and `Aᵢ` start fresh (volatile state lost)
//! - Peers will detect the gap and request retransmission

use crate::buffer::{ReplicaId, SeqNo};
use mdcs_core::lattice::Lattice;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// A delta-interval message for causal delivery
///
/// Contains: `⟨delta, from_seq, to_seq⟩`
/// - `delta`: The joined delta-group to apply
/// - `from_seq`: Starting sequence number (exclusive)
/// - `to_seq`: Ending sequence number (inclusive)
///
/// The receiver should only accept if `from_seq == last_acked_from_this_sender`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeltaInterval<D> {
    /// The source replica that generated this interval
    pub from: ReplicaId,
    /// The destination replica
    pub to: ReplicaId,
    /// The joined delta-group
    pub delta: D,
    /// Sequence number just before this interval (exclusive lower bound)
    pub from_seq: SeqNo,
    /// Sequence number at the end of this interval (inclusive upper bound)
    pub to_seq: SeqNo,
}

/// Acknowledgment for a delta-interval
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntervalAck {
    pub from: ReplicaId,
    pub to: ReplicaId,
    /// The sequence number being acknowledged
    pub acked_seq: SeqNo,
}

/// Messages for the causal anti-entropy protocol
#[derive(Debug, Clone)]
pub enum CausalMessage<D> {
    /// Delta-interval with causal ordering information
    DeltaInterval(DeltaInterval<D>),
    /// Acknowledgment of received interval
    Ack(IntervalAck),
    /// Request for state snapshot (for bootstrapping new replicas)
    SnapshotRequest { from: ReplicaId, to: ReplicaId },
    /// Full state snapshot response
    Snapshot {
        from: ReplicaId,
        to: ReplicaId,
        state: D,
        seq: SeqNo,
    },
}

/// Durable state that survives crashes
///
/// This must be persisted to stable storage before acknowledging
/// any mutation or received delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurableState<S> {
    /// The replica's unique identifier
    pub replica_id: ReplicaId,
    /// The current CRDT state
    pub state: S,
    /// The durable counter (last generated sequence number)
    pub counter: SeqNo,
}

impl<S: Lattice> DurableState<S> {
    pub fn new(replica_id: impl Into<ReplicaId>) -> Self {
        Self {
            replica_id: replica_id.into(),
            state: S::bottom(),
            counter: 0,
        }
    }
}

/// Per-peer delta buffer for causal mode
///
/// Stores deltas that need to be sent to a specific peer,
/// along with the sequence range they cover.
#[derive(Debug, Clone)]
pub struct PeerDeltaBuffer<D: Lattice> {
    /// The accumulated delta to send
    delta: Option<D>,
    /// Sequence number before the first delta in buffer
    from_seq: SeqNo,
    /// Sequence number of the last delta in buffer
    to_seq: SeqNo,
}

impl<D: Lattice> PeerDeltaBuffer<D> {
    pub fn new() -> Self {
        Self {
            delta: None,
            from_seq: 0,
            to_seq: 0,
        }
    }

    /// Start tracking from a specific sequence number
    pub fn start_from(seq: SeqNo) -> Self {
        Self {
            delta: None,
            from_seq: seq,
            to_seq: seq,
        }
    }

    /// Add a delta to this buffer
    pub fn push(&mut self, delta: D, seq: SeqNo) {
        match &mut self.delta {
            Some(existing) => {
                existing.join_assign(&delta);
            }
            None => {
                self.delta = Some(delta);
            }
        }
        self.to_seq = seq;
    }

    /// Check if buffer has pending deltas
    pub fn has_pending(&self) -> bool {
        self.delta.is_some()
    }

    /// Take the delta, clearing the buffer
    pub fn take(&mut self) -> Option<(D, SeqNo, SeqNo)> {
        self.delta.take().map(|d| {
            let from = self.from_seq;
            let to = self.to_seq;
            self.from_seq = to;
            (d, from, to)
        })
    }

    /// Clear the buffer (on successful ack)
    pub fn clear(&mut self) {
        self.delta = None;
        self.from_seq = self.to_seq;
    }

    /// Reset the buffer from a new sequence (after peer reconnect)
    pub fn reset_from(&mut self, seq: SeqNo) {
        self.delta = None;
        self.from_seq = seq;
        self.to_seq = seq;
    }
}

impl<D: Lattice> Default for PeerDeltaBuffer<D> {
    fn default() -> Self {
        Self::new()
    }
}

/// Volatile state for causal anti-entropy (lost on crash)
#[derive(Debug, Clone)]
pub struct VolatileState<D: Lattice> {
    /// Per-peer delta buffers: Dᵢ\[j\]
    pub delta_buffers: HashMap<ReplicaId, PeerDeltaBuffer<D>>,
    /// Per-peer acknowledgment tracking: Aᵢ\[j\]
    /// Stores the last sequence number we've received from each peer
    pub peer_acks: HashMap<ReplicaId, SeqNo>,
}

impl<D: Lattice> VolatileState<D> {
    pub fn new() -> Self {
        Self {
            delta_buffers: HashMap::new(),
            peer_acks: HashMap::new(),
        }
    }

    /// Register a peer
    pub fn register_peer(&mut self, peer_id: ReplicaId) {
        self.delta_buffers
            .entry(peer_id.clone())
            .or_default();
        self.peer_acks.entry(peer_id).or_insert(0);
    }

    /// Get last acked sequence from a peer
    pub fn get_peer_ack(&self, peer_id: &str) -> SeqNo {
        self.peer_acks.get(peer_id).copied().unwrap_or(0)
    }

    /// Update the ack for a peer
    pub fn update_peer_ack(&mut self, peer_id: &str, seq: SeqNo) {
        if let Some(ack) = self.peer_acks.get_mut(peer_id) {
            *ack = (*ack).max(seq);
        }
    }
}

impl<D: Lattice> Default for VolatileState<D> {
    fn default() -> Self {
        Self::new()
    }
}

/// A causal δ-CRDT replica implementing Algorithm 2
///
/// Provides causal consistency guarantees by:
/// 1. Tracking per-peer delta intervals
/// 2. Only accepting deltas in causal order
/// 3. Supporting crash recovery via durable state
#[derive(Debug, Clone)]
pub struct CausalReplica<S: Lattice + Clone> {
    /// Durable state (survives crashes)
    durable: DurableState<S>,
    /// Volatile state (lost on crash)
    volatile: VolatileState<S>,
    /// Pending deltas waiting for causal predecessors
    pending: HashMap<ReplicaId, VecDeque<DeltaInterval<S>>>,
}

impl<S: Lattice + Clone> CausalReplica<S> {
    /// Create a new causal replica
    pub fn new(id: impl Into<ReplicaId>) -> Self {
        Self {
            durable: DurableState::new(id),
            volatile: VolatileState::new(),
            pending: HashMap::new(),
        }
    }

    /// Restore from durable state (after crash)
    pub fn restore(durable: DurableState<S>) -> Self {
        Self {
            durable,
            volatile: VolatileState::new(),
            pending: HashMap::new(),
        }
    }

    /// Get the replica ID
    pub fn id(&self) -> &ReplicaId {
        &self.durable.replica_id
    }

    /// Get current state (read-only)
    pub fn state(&self) -> &S {
        &self.durable.state
    }

    /// Get the durable counter (sequence number)
    pub fn counter(&self) -> SeqNo {
        self.durable.counter
    }

    /// Get durable state for persistence
    pub fn durable_state(&self) -> &DurableState<S> {
        &self.durable
    }

    /// Register a peer for causal anti-entropy
    pub fn register_peer(&mut self, peer_id: ReplicaId) {
        self.volatile.register_peer(peer_id.clone());
        self.pending.entry(peer_id).or_default();
    }

    /// Apply a local mutation
    ///
    /// Algorithm 2, step 1:
    /// ```text
    /// cᵢ := cᵢ + 1
    /// d := mδ(Xᵢ)
    /// Xᵢ := Xᵢ ⊔ d
    /// ∀j: Dᵢ[j] := Dᵢ[j] ⊔ d
    /// ```
    ///
    /// Returns the computed delta
    pub fn mutate<F>(&mut self, mutator: F) -> S
    where
        F: FnOnce(&S) -> S,
    {
        // Increment durable counter
        self.durable.counter += 1;
        let seq = self.durable.counter;

        // Compute delta: d = mδ(X)
        let delta = mutator(&self.durable.state);

        // Apply to state: X = X ⊔ d
        self.durable.state.join_assign(&delta);

        // Add to all peer buffers: ∀j: Dᵢ[j] := Dᵢ[j] ⊔ d
        for buffer in self.volatile.delta_buffers.values_mut() {
            buffer.push(delta.clone(), seq);
        }

        delta
    }

    /// Prepare a delta-interval to send to a peer
    ///
    /// Returns `Some(DeltaInterval)` if there are pending deltas for this peer,
    /// or `None` if the buffer is empty.
    pub fn prepare_interval(&mut self, peer_id: &str) -> Option<DeltaInterval<S>> {
        let buffer = self.volatile.delta_buffers.get_mut(peer_id)?;

        buffer
            .take()
            .map(|(delta, from_seq, to_seq)| DeltaInterval {
                from: self.durable.replica_id.clone(),
                to: peer_id.to_string(),
                delta,
                from_seq,
                to_seq,
            })
    }

    /// Check if a delta-interval is causally ready
    ///
    /// A delta-interval is ready if its from_seq matches our last acked seq from that peer
    fn is_causally_ready(&self, interval: &DeltaInterval<S>) -> bool {
        let last_acked = self.volatile.get_peer_ack(&interval.from);
        interval.from_seq == last_acked
    }

    /// Receive a delta-interval from a peer
    ///
    /// Algorithm 2, step 3:
    /// ```text
    /// if n = Aᵢ[j] + 1 then        // causally ready
    ///     Xᵢ := Xᵢ ⊔ d
    ///     Aᵢ[j] := m
    ///     send ack(m) to j
    /// else
    ///     buffer for later
    /// ```
    ///
    /// Returns `Some(IntervalAck)` if the interval was applied (causally ready),
    /// or `None` if it was buffered for later.
    pub fn receive_interval(&mut self, interval: DeltaInterval<S>) -> Option<IntervalAck> {
        // Register the peer if not known
        if !self.volatile.peer_acks.contains_key(&interval.from) {
            self.register_peer(interval.from.clone());
        }

        if self.is_causally_ready(&interval) {
            // Apply the delta
            self.durable.state.join_assign(&interval.delta);

            // Update our ack for this peer
            self.volatile
                .update_peer_ack(&interval.from, interval.to_seq);

            let ack = IntervalAck {
                from: self.durable.replica_id.clone(),
                to: interval.from.clone(),
                acked_seq: interval.to_seq,
            };

            // Try to apply any pending intervals that are now ready
            self.try_apply_pending(&interval.from);

            Some(ack)
        } else {
            // Buffer for later
            let pending = self
                .pending
                .entry(interval.from.clone())
                .or_default();

            // Insert in sorted order by from_seq
            let pos = pending.iter().position(|p| p.from_seq > interval.from_seq);
            match pos {
                Some(i) => pending.insert(i, interval),
                None => pending.push_back(interval),
            }

            None
        }
    }

    /// Try to apply pending intervals that are now causally ready
    fn try_apply_pending(&mut self, peer_id: &str) -> Vec<IntervalAck> {
        let mut acks = Vec::new();

        if let Some(pending) = self.pending.get_mut(peer_id) {
            while let Some(interval) = pending.front() {
                let last_acked = self.volatile.get_peer_ack(peer_id);
                if interval.from_seq == last_acked {
                    let interval = pending.pop_front().unwrap();

                    // Apply the delta
                    self.durable.state.join_assign(&interval.delta);

                    // Update our ack
                    self.volatile.update_peer_ack(peer_id, interval.to_seq);

                    acks.push(IntervalAck {
                        from: self.durable.replica_id.clone(),
                        to: interval.from.clone(),
                        acked_seq: interval.to_seq,
                    });
                } else {
                    break;
                }
            }
        }

        acks
    }

    /// Process an acknowledgment from a peer
    ///
    /// Algorithm 2, step 4:
    /// ```text
    /// Dᵢ[j] := ⊥   // clear delta buffer for j
    /// ```
    pub fn receive_ack(&mut self, ack: &IntervalAck) {
        if let Some(buffer) = self.volatile.delta_buffers.get_mut(&ack.from) {
            buffer.clear();
        }
    }

    /// Get a full state snapshot for bootstrapping
    pub fn snapshot(&self) -> (S, SeqNo) {
        (self.durable.state.clone(), self.durable.counter)
    }

    /// Apply a snapshot from another replica (for bootstrapping)
    pub fn apply_snapshot(&mut self, state: S, seq: SeqNo, from: &str) {
        self.durable.state.join_assign(&state);
        self.volatile.update_peer_ack(from, seq);
    }

    /// Get all registered peer IDs
    pub fn peers(&self) -> impl Iterator<Item = &ReplicaId> {
        self.volatile.peer_acks.keys()
    }

    /// Check if we have pending deltas for any peer
    pub fn has_pending_deltas(&self) -> bool {
        self.volatile
            .delta_buffers
            .values()
            .any(|b| b.has_pending())
    }

    /// Count of pending out-of-order intervals
    pub fn pending_count(&self) -> usize {
        self.pending.values().map(|v| v.len()).sum()
    }
}

/// Trait for durable storage backends
///
/// Implement this trait to persist `DurableState` across crashes.
pub trait DurableStorage<S: Lattice> {
    /// Persist the durable state
    fn persist(&mut self, state: &DurableState<S>) -> Result<(), StorageError>;

    /// Load the durable state
    fn load(&self, replica_id: &str) -> Result<Option<DurableState<S>>, StorageError>;

    /// Force sync to stable storage
    fn sync(&mut self) -> Result<(), StorageError>;
}

/// Storage errors
#[derive(Debug, Clone)]
pub enum StorageError {
    IoError(String),
    SerializationError(String),
    NotFound,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::IoError(msg) => write!(f, "IO error: {}", msg),
            StorageError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::NotFound => write!(f, "State not found"),
        }
    }
}

impl std::error::Error for StorageError {}

/// In-memory storage for testing (simulates durable storage)
#[derive(Debug, Default)]
pub struct MemoryStorage<S> {
    states: HashMap<ReplicaId, DurableState<S>>,
}

impl<S: Clone> MemoryStorage<S> {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }
}

impl<S: Lattice + Clone + Serialize + for<'de> Deserialize<'de>> DurableStorage<S>
    for MemoryStorage<S>
{
    fn persist(&mut self, state: &DurableState<S>) -> Result<(), StorageError> {
        self.states.insert(state.replica_id.clone(), state.clone());
        Ok(())
    }

    fn load(&self, replica_id: &str) -> Result<Option<DurableState<S>>, StorageError> {
        Ok(self.states.get(replica_id).cloned())
    }

    fn sync(&mut self) -> Result<(), StorageError> {
        Ok(())
    }
}

/// Network simulator for causal anti-entropy
#[derive(Debug)]
pub struct CausalNetworkSimulator<D> {
    /// Messages in flight
    in_flight: VecDeque<CausalMessage<D>>,
    /// Messages that were "lost"
    lost: Vec<CausalMessage<D>>,
    /// Loss rate (0.0 - 1.0)
    loss_rate: f64,
    /// Random state
    rng_state: u64,
}

impl<D: Clone> CausalNetworkSimulator<D> {
    pub fn new(loss_rate: f64) -> Self {
        Self {
            in_flight: VecDeque::new(),
            lost: Vec::new(),
            loss_rate,
            rng_state: 42,
        }
    }

    /// Simple random number generator
    fn next_random(&mut self) -> f64 {
        self.rng_state = self.rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.rng_state >> 16) & 0x7fff) as f64 / 32768.0
    }

    /// Send a message
    pub fn send(&mut self, msg: CausalMessage<D>) {
        if self.next_random() < self.loss_rate {
            self.lost.push(msg);
        } else {
            self.in_flight.push_back(msg);
        }
    }

    /// Receive the next message
    pub fn receive(&mut self) -> Option<CausalMessage<D>> {
        self.in_flight.pop_front()
    }

    /// Retransmit lost messages
    pub fn retransmit_lost(&mut self) {
        for msg in self.lost.drain(..) {
            self.in_flight.push_back(msg);
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.in_flight.is_empty()
    }

    /// Messages in flight
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    /// Lost messages
    pub fn lost_count(&self) -> usize {
        self.lost.len()
    }
}

/// Cluster coordinator for causal anti-entropy
#[derive(Debug)]
pub struct CausalCluster<S: Lattice + Clone> {
    /// All replicas
    replicas: Vec<CausalReplica<S>>,
    /// Network simulator
    network: CausalNetworkSimulator<S>,
}

impl<S: Lattice + Clone> CausalCluster<S> {
    /// Create a new cluster with n replicas
    pub fn new(n: usize, loss_rate: f64) -> Self {
        let mut replicas = Vec::with_capacity(n);

        // Create replicas
        for i in 0..n {
            let mut replica = CausalReplica::new(format!("causal_{}", i));
            // Register all other peers
            for j in 0..n {
                if i != j {
                    replica.register_peer(format!("causal_{}", j));
                }
            }
            replicas.push(replica);
        }

        Self {
            replicas,
            network: CausalNetworkSimulator::new(loss_rate),
        }
    }

    /// Get replica by index
    pub fn replica(&self, idx: usize) -> &CausalReplica<S> {
        &self.replicas[idx]
    }

    /// Get mutable replica
    pub fn replica_mut(&mut self, idx: usize) -> &mut CausalReplica<S> {
        &mut self.replicas[idx]
    }

    /// Perform a mutation
    pub fn mutate<F>(&mut self, replica_idx: usize, mutator: F) -> S
    where
        F: FnOnce(&S) -> S,
    {
        self.replicas[replica_idx].mutate(mutator)
    }

    /// Initiate sync from one replica to all its peers
    pub fn broadcast_intervals(&mut self, from_idx: usize) {
        let replica = &mut self.replicas[from_idx];
        let peer_ids: Vec<_> = replica.peers().cloned().collect();

        for peer_id in peer_ids {
            if let Some(interval) = replica.prepare_interval(&peer_id) {
                self.network.send(CausalMessage::DeltaInterval(interval));
            }
        }
    }

    /// Process one network message
    pub fn process_one(&mut self) -> bool {
        if let Some(msg) = self.network.receive() {
            match msg {
                CausalMessage::DeltaInterval(interval) => {
                    // Find recipient
                    for replica in &mut self.replicas {
                        if replica.id() == &interval.to {
                            if let Some(ack) = replica.receive_interval(interval.clone()) {
                                self.network.send(CausalMessage::Ack(ack));
                            }
                            break;
                        }
                    }
                }
                CausalMessage::Ack(ack) => {
                    // Find recipient
                    for replica in &mut self.replicas {
                        if replica.id() == &ack.to {
                            replica.receive_ack(&ack);
                            break;
                        }
                    }
                }
                CausalMessage::SnapshotRequest { from, to } => {
                    // Find source and send snapshot
                    for replica in &self.replicas {
                        if replica.id() == &to {
                            let (state, seq) = replica.snapshot();
                            self.network.send(CausalMessage::Snapshot {
                                from: to,
                                to: from,
                                state,
                                seq,
                            });
                            break;
                        }
                    }
                }
                CausalMessage::Snapshot {
                    from,
                    to,
                    state,
                    seq,
                } => {
                    // Find recipient and apply
                    for replica in &mut self.replicas {
                        if replica.id() == &to {
                            replica.apply_snapshot(state, seq, &from);
                            break;
                        }
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Drain all messages
    pub fn drain_network(&mut self) {
        while self.process_one() {}
    }

    /// Full sync round
    pub fn full_sync_round(&mut self) {
        let n = self.replicas.len();
        for i in 0..n {
            self.broadcast_intervals(i);
        }
        self.drain_network();
    }

    /// Check if converged
    pub fn is_converged(&self) -> bool {
        if self.replicas.len() < 2 {
            return true;
        }

        let first = self.replicas[0].state();
        self.replicas.iter().skip(1).all(|r| r.state() == first)
    }

    /// Retransmit and process
    pub fn retransmit_and_process(&mut self) {
        self.network.retransmit_lost();
        self.drain_network();
    }

    /// Number of replicas
    pub fn len(&self) -> usize {
        self.replicas.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.replicas.is_empty()
    }

    /// Simulate a crash and recovery for a replica
    pub fn crash_and_recover(&mut self, idx: usize) {
        let durable = self.replicas[idx].durable_state().clone();

        // Restore from durable state (volatile state is lost)
        let mut recovered = CausalReplica::restore(durable);

        // Re-register peers
        let n = self.replicas.len();
        for j in 0..n {
            if idx != j {
                recovered.register_peer(format!("causal_{}", j));
            }
        }

        self.replicas[idx] = recovered;
    }

    /// Get total pending out-of-order intervals across all replicas
    pub fn total_pending(&self) -> usize {
        self.replicas.iter().map(|r| r.pending_count()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdcs_core::gset::GSet;
    use mdcs_core::pncounter::PNCounter;

    #[test]
    fn test_causal_replica_basic() {
        let mut replica: CausalReplica<GSet<i32>> = CausalReplica::new("test1");

        replica.mutate(|_| {
            let mut d = GSet::new();
            d.insert(42);
            d
        });

        assert!(replica.state().contains(&42));
        assert_eq!(replica.counter(), 1);
    }

    #[test]
    fn test_causal_interval_generation() {
        let mut replica: CausalReplica<GSet<i32>> = CausalReplica::new("test1");
        replica.register_peer("peer1".to_string());

        replica.mutate(|_| {
            let mut d = GSet::new();
            d.insert(1);
            d
        });

        replica.mutate(|_| {
            let mut d = GSet::new();
            d.insert(2);
            d
        });

        let interval = replica.prepare_interval("peer1").unwrap();
        assert_eq!(interval.from_seq, 0);
        assert_eq!(interval.to_seq, 2);
        assert!(interval.delta.contains(&1));
        assert!(interval.delta.contains(&2));
    }

    #[test]
    fn test_causal_delivery() {
        let mut r1: CausalReplica<GSet<i32>> = CausalReplica::new("r1");
        let mut r2: CausalReplica<GSet<i32>> = CausalReplica::new("r2");

        r1.register_peer("r2".to_string());
        r2.register_peer("r1".to_string());

        // r1 creates two mutations
        r1.mutate(|_| {
            let mut d = GSet::new();
            d.insert(1);
            d
        });
        r1.mutate(|_| {
            let mut d = GSet::new();
            d.insert(2);
            d
        });

        // Get interval
        let interval = r1.prepare_interval("r2").unwrap();
        assert_eq!(interval.from_seq, 0);
        assert_eq!(interval.to_seq, 2);

        // r2 receives it
        let ack = r2.receive_interval(interval).unwrap();
        assert_eq!(ack.acked_seq, 2);

        // r2 now has both elements
        assert!(r2.state().contains(&1));
        assert!(r2.state().contains(&2));
    }

    #[test]
    fn test_out_of_order_buffering() {
        let mut replica: CausalReplica<GSet<i32>> = CausalReplica::new("r1");
        replica.register_peer("peer".to_string());

        // Create an interval that's NOT causally ready (from_seq = 5, but we've acked 0)
        let out_of_order = DeltaInterval {
            from: "peer".to_string(),
            to: "r1".to_string(),
            delta: {
                let mut d = GSet::new();
                d.insert(999);
                d
            },
            from_seq: 5, // Not ready - we haven't seen 1-5
            to_seq: 6,
        };

        // Should be buffered, not applied
        let result = replica.receive_interval(out_of_order);
        assert!(result.is_none());
        assert_eq!(replica.pending_count(), 1);
        assert!(!replica.state().contains(&999));
    }

    #[test]
    fn test_cluster_convergence() {
        let mut cluster: CausalCluster<GSet<i32>> = CausalCluster::new(3, 0.0);

        // Each replica adds different element
        for i in 0..3 {
            let val = (i + 1) as i32;
            cluster.mutate(i, move |_| {
                let mut d = GSet::new();
                d.insert(val);
                d
            });
        }

        // Not converged yet
        assert!(!cluster.is_converged());

        // Sync
        cluster.full_sync_round();

        // Should converge
        assert!(cluster.is_converged());

        // All replicas should have all elements
        for i in 0..3 {
            for val in 1..=3 {
                assert!(cluster.replica(i).state().contains(&val));
            }
        }
    }

    #[test]
    fn test_cluster_with_loss() {
        let mut cluster: CausalCluster<GSet<i32>> = CausalCluster::new(3, 0.3);

        for i in 0..3 {
            let val = (i + 1) as i32;
            cluster.mutate(i, move |_| {
                let mut d = GSet::new();
                d.insert(val);
                d
            });
        }

        // Multiple rounds with retransmission
        for _ in 0..10 {
            cluster.full_sync_round();
            cluster.retransmit_and_process();
        }

        // Should eventually converge
        assert!(cluster.is_converged());
    }

    #[test]
    fn test_crash_recovery() {
        let mut cluster: CausalCluster<GSet<i32>> = CausalCluster::new(2, 0.0);

        // r0 adds element
        cluster.mutate(0, |_| {
            let mut d = GSet::new();
            d.insert(1);
            d
        });

        // Sync
        cluster.full_sync_round();
        assert!(cluster.is_converged());

        // r0 adds another element
        cluster.mutate(0, |_| {
            let mut d = GSet::new();
            d.insert(2);
            d
        });

        // r0 crashes before syncing
        let counter_before = cluster.replica(0).counter();
        cluster.crash_and_recover(0);

        // Durable state should be preserved
        assert_eq!(cluster.replica(0).counter(), counter_before);
        assert!(cluster.replica(0).state().contains(&1));
        assert!(cluster.replica(0).state().contains(&2));

        // But volatile state (delta buffers) is lost
        // r0 needs to re-sync
        assert!(!cluster.replica(0).has_pending_deltas());
    }

    #[test]
    fn test_pncounter_causal() {
        let mut cluster: CausalCluster<PNCounter<String>> = CausalCluster::new(2, 0.0);

        // r0 increments
        cluster.mutate(0, |_s| {
            let mut delta = PNCounter::new();
            delta.increment("r0".to_string(), 1);
            delta
        });

        // r1 decrements
        cluster.mutate(1, |_s| {
            let mut delta = PNCounter::new();
            delta.decrement("r1".to_string(), 1);
            delta
        });

        // Sync
        cluster.full_sync_round();

        // Both should have value 0 (1 - 1)
        assert!(cluster.is_converged());
        assert_eq!(cluster.replica(0).state().value(), 0);
    }

    #[test]
    fn test_causal_ordering_preserved() {
        // This test verifies that causal ordering is respected
        let mut r1: CausalReplica<GSet<i32>> = CausalReplica::new("r1");
        let mut r2: CausalReplica<GSet<i32>> = CausalReplica::new("r2");

        r1.register_peer("r2".to_string());
        r2.register_peer("r1".to_string());

        // r1 creates three sequential mutations
        for i in 1..=3 {
            r1.mutate(move |_| {
                let mut d = GSet::new();
                d.insert(i);
                d
            });
        }

        // Create intervals for each mutation
        // Simulate them arriving out of order by creating separate intervals

        // We need to manually create intervals to test out-of-order delivery
        let interval_1_3 = DeltaInterval {
            from: "r1".to_string(),
            to: "r2".to_string(),
            delta: {
                let mut d = GSet::new();
                d.insert(3);
                d
            },
            from_seq: 2, // This requires seq 1-2 to be acked first
            to_seq: 3,
        };

        let interval_0_2 = DeltaInterval {
            from: "r1".to_string(),
            to: "r2".to_string(),
            delta: {
                let mut d = GSet::new();
                d.insert(1);
                d.insert(2);
                d
            },
            from_seq: 0,
            to_seq: 2,
        };

        // Send interval 2-3 first (out of order)
        let result = r2.receive_interval(interval_1_3.clone());
        assert!(result.is_none()); // Should be buffered
        assert!(!r2.state().contains(&3)); // Not yet applied

        // Now send interval 0-2
        let result = r2.receive_interval(interval_0_2);
        assert!(result.is_some()); // Should be applied
        assert!(r2.state().contains(&1));
        assert!(r2.state().contains(&2));

        // And the pending interval should now be applied too!
        assert!(r2.state().contains(&3));
        assert_eq!(r2.pending_count(), 0);
    }

    #[test]
    fn test_durable_storage() {
        let mut storage: MemoryStorage<GSet<i32>> = MemoryStorage::new();

        let mut replica: CausalReplica<GSet<i32>> = CausalReplica::new("test");
        replica.mutate(|_| {
            let mut d = GSet::new();
            d.insert(42);
            d
        });

        // Persist
        storage.persist(replica.durable_state()).unwrap();

        // Load
        let loaded = storage.load("test").unwrap().unwrap();
        assert_eq!(loaded.counter, 1);
        assert!(loaded.state.contains(&42));

        // Restore
        let recovered = CausalReplica::restore(loaded);
        assert!(recovered.state().contains(&42));
    }
}
