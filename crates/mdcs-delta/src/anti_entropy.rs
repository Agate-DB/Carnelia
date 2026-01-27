//! δ-CRDT Anti-Entropy Algorithm 1 (Convergence Mode)
//!
//! This module implements the anti-entropy algorithm from the δ-CRDT paper.
//! It handles delta propagation, acknowledgments, and convergence testing.
//!
//! # Algorithm 1 Overview
//!
//! Each replica maintains:
//! - X: the local CRDT state
//! - D: delta buffer (sequence of deltas)
//! - acked[j]: last sequence number acknowledged by peer j
//!
//! Protocol:
//! 1. On local mutation m:
//!    - d = mδ(X)     // compute delta
//!    - X = X ⊔ d     // apply to local state
//!    - D.push(d)     // buffer for sending
//!
//! 2. On send to peer j:
//!    - send D[acked[j]..] to j
//!
//! 3. On receive delta d from peer i:
//!    - X = X ⊔ d     // apply (idempotent!)
//!    - send ack(seq) to i

use crate::buffer::{DeltaReplica, ReplicaId, SeqNo};
use mdcs_core::lattice::Lattice;
use std::collections::VecDeque;

/// Message types for the anti-entropy protocol
#[derive(Debug, Clone)]
pub enum AntiEntropyMessage<D> {
    /// Delta message: contains delta, source, destination and sequence number
    Delta { from: ReplicaId, to: ReplicaId, delta: D, seq: SeqNo },
    /// Acknowledgment message: from -> to acknowledges seq
    Ack { from: ReplicaId, to: ReplicaId, seq: SeqNo },
}

/// A network simulator for testing anti-entropy under various conditions
#[derive(Debug)]
pub struct NetworkSimulator<D> {
    /// Messages in flight
    in_flight: VecDeque<AntiEntropyMessage<D>>,
    /// Messages that were "lost"
    lost: Vec<AntiEntropyMessage<D>>,
    /// Configuration
    config: NetworkConfig,
    /// Random seed for deterministic testing
    rng_state: u64,
}

/// Network configuration for simulation
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Probability of message loss (0.0 - 1.0)
    pub loss_rate: f64,
    /// Probability of message duplication (0.0 - 1.0)
    pub dup_rate: f64,
    /// Probability of message reordering (0.0 - 1.0)
    pub reorder_rate: f64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            loss_rate: 0.0,
            dup_rate: 0.0,
            reorder_rate: 0.0,
        }
    }
}

impl NetworkConfig {
    /// Create a lossy network configuration
    pub fn lossy(loss_rate: f64) -> Self {
        Self {
            loss_rate,
            ..Default::default()
        }
    }

    /// Create a network with duplicates
    pub fn with_dups(dup_rate: f64) -> Self {
        Self {
            dup_rate,
            ..Default::default()
        }
    }

    /// Create a chaotic network (all problems)
    pub fn chaotic() -> Self {
        Self {
            loss_rate: 0.1,
            dup_rate: 0.2,
            reorder_rate: 0.3,
        }
    }
}

impl<D: Clone> NetworkSimulator<D> {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            in_flight: VecDeque::new(),
            lost: Vec::new(),
            config,
            rng_state: 12345,
        }
    }

    /// Simple LCG random number generator
    fn next_random(&mut self) -> f64 {
        self.rng_state = self.rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.rng_state >> 16) & 0x7fff) as f64 / 32768.0
    }

    /// Send a message through the network
    pub fn send(&mut self, msg: AntiEntropyMessage<D>) {
        // Check for loss
        if self.next_random() < self.config.loss_rate {
            self.lost.push(msg);
            return;
        }

        // Check for duplication
        if self.next_random() < self.config.dup_rate {
            self.in_flight.push_back(msg.clone());
        }

        // Check for reordering
        if self.next_random() < self.config.reorder_rate && !self.in_flight.is_empty() {
            // Insert at random position
            let pos = (self.next_random() * self.in_flight.len() as f64) as usize;
            let pos = pos.min(self.in_flight.len());
            // VecDeque doesn't have insert, so we'll just push and let it reorder naturally
            self.in_flight.push_back(msg);
            if pos < self.in_flight.len() - 1 {
                // Swap with a random earlier position to simulate reordering
                self.in_flight.swap(pos, self.in_flight.len() - 1);
            }
        } else {
            self.in_flight.push_back(msg);
        }
    }

    /// Receive the next message (if any)
    pub fn receive(&mut self) -> Option<AntiEntropyMessage<D>> {
        self.in_flight.pop_front()
    }

    /// Re-send lost messages (simulates retransmission)
    pub fn retransmit_lost(&mut self) {
        for msg in self.lost.drain(..) {
            self.in_flight.push_back(msg);
        }
    }

    /// Check if network is empty
    pub fn is_empty(&self) -> bool {
        self.in_flight.is_empty()
    }

    /// Number of messages in flight
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    /// Number of lost messages
    pub fn lost_count(&self) -> usize {
        self.lost.len()
    }
}

/// Anti-entropy coordinator for a cluster of replicas
#[derive(Debug)]
pub struct AntiEntropyCluster<S: Lattice + Clone> {
    /// All replicas in the cluster
    replicas: Vec<DeltaReplica<S, S>>,
    /// Network simulator
    network: NetworkSimulator<S>,
}

impl<S: Lattice + Clone> AntiEntropyCluster<S> {
    /// Create a new cluster with n replicas
    pub fn new(n: usize, config: NetworkConfig) -> Self {
        let mut replicas = Vec::with_capacity(n);

        // Create replicas
        for i in 0..n {
            let mut replica = DeltaReplica::new(format!("replica_{}", i));
            // Register all other peers
            for j in 0..n {
                if i != j {
                    replica.register_peer(format!("replica_{}", j));
                }
            }
            replicas.push(replica);
        }

        Self {
            replicas,
            network: NetworkSimulator::new(config),
        }
    }

    /// Get replica by index
    pub fn replica(&self, idx: usize) -> &DeltaReplica<S, S> {
        &self.replicas[idx]
    }

    /// Get mutable replica by index
    pub fn replica_mut(&mut self, idx: usize) -> &mut DeltaReplica<S, S> {
        &mut self.replicas[idx]
    }

    /// Perform a mutation on a specific replica
    pub fn mutate<F>(&mut self, replica_idx: usize, mutator: F) -> S
    where
        F: FnOnce(&S) -> S,
    {
        self.replicas[replica_idx].mutate(mutator)
    }

    /// Initiate sync from one replica to another
    pub fn initiate_sync(&mut self, from_idx: usize, to_idx: usize) {
        let to_id = self.replicas[to_idx].id.clone();
        if let Some((delta, seq)) = self.replicas[from_idx].prepare_sync(&to_id) {
            let msg = AntiEntropyMessage::Delta {
                from: self.replicas[from_idx].id.clone(),
                to: to_id.clone(),
                delta,
                seq,
            };
            self.network.send(msg);
        }
    }

    /// Process one network message
    pub fn process_one(&mut self) -> bool {
        if let Some(msg) = self.network.receive() {
            match msg {
                AntiEntropyMessage::Delta { from, to, delta, seq } => {
                    // Deliver delta to the intended recipient only
                    for replica in &mut self.replicas {
                        if replica.id == to {
                            replica.receive_delta(&delta);
                            // Send ack back to the original sender
                            let ack = AntiEntropyMessage::Ack {
                                from: replica.id.clone(),
                                to: from.clone(),
                                seq,
                            };
                            self.network.send(ack);
                            break;
                        }
                    }
                }
                AntiEntropyMessage::Ack { from, to, seq } => {
                    // Deliver ack to the intended recipient only
                    for replica in &mut self.replicas {
                        if replica.id == to {
                            replica.process_ack(&from, seq);
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

    /// Run until network is empty
    pub fn drain_network(&mut self) {
        while self.process_one() {}
    }

    /// Broadcast delta from one replica to all others
    pub fn broadcast(&mut self, from_idx: usize) {
        let n = self.replicas.len();
        for to_idx in 0..n {
            if from_idx != to_idx {
                self.initiate_sync(from_idx, to_idx);
            }
        }
    }

    /// Full sync: every replica syncs with every other replica
    pub fn full_sync_round(&mut self) {
        let n = self.replicas.len();
        for from_idx in 0..n {
            for to_idx in 0..n {
                if from_idx != to_idx {
                    self.initiate_sync(from_idx, to_idx);
                }
            }
        }
        self.drain_network();
    }

    /// Check if all replicas have converged
    pub fn is_converged(&self) -> bool {
        if self.replicas.len() < 2 {
            return true;
        }

        let first = self.replicas[0].state();
        self.replicas.iter().skip(1).all(|r| r.state() == first)
    }

    /// Retransmit lost messages and process
    pub fn retransmit_and_process(&mut self) {
        self.network.retransmit_lost();
        self.drain_network();
    }

    /// Get number of replicas
    pub fn len(&self) -> usize {
        self.replicas.len()
    }

    /// Check if cluster is empty
    pub fn is_empty(&self) -> bool {
        self.replicas.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdcs_core::gset::GSet;

    #[test]
    fn test_network_simulator_basic() {
        let mut net: NetworkSimulator<i32> = NetworkSimulator::new(NetworkConfig::default());

        net.send(AntiEntropyMessage::Delta {
            from: "r1".to_string(),
            to: "".to_string(),
            delta: 42,
            seq: 1,
        });

        assert_eq!(net.in_flight_count(), 1);

        let msg = net.receive().unwrap();
        match msg {
            AntiEntropyMessage::Delta { delta, .. } => assert_eq!(delta, 42),
            _ => panic!("Expected delta message"),
        }
    }

    #[test]
    fn test_cluster_basic_convergence() {
        let mut cluster: AntiEntropyCluster<GSet<i32>> =
            AntiEntropyCluster::new(3, NetworkConfig::default());

        // Replica 0 inserts 1
        cluster.mutate(0, |_| {
            let mut d = GSet::new();
            d.insert(1);
            d
        });

        // Replica 1 inserts 2
        cluster.mutate(1, |_| {
            let mut d = GSet::new();
            d.insert(2);
            d
        });

        // Not converged yet
        assert!(!cluster.is_converged());

        // Full sync
        cluster.full_sync_round();

        // Now should be converged
        assert!(cluster.is_converged());

        // All replicas should have both elements
        for i in 0..3 {
            assert!(cluster.replica(i).state().contains(&1));
            assert!(cluster.replica(i).state().contains(&2));
        }
    }

    #[test]
    fn test_convergence_under_loss() {
        let mut cluster: AntiEntropyCluster<GSet<i32>> =
            AntiEntropyCluster::new(3, NetworkConfig::lossy(0.5));

        // Add different elements to each replica
        for i in 0..3 {
            let val = (i + 1) as i32;
            cluster.mutate(i, move |_| {
                let mut d = GSet::new();
                d.insert(val);
                d
            });
        }

        // Do multiple sync rounds with retransmission
        for _ in 0..10 {
            cluster.full_sync_round();
            cluster.retransmit_and_process();
        }

        // Should eventually converge
        assert!(cluster.is_converged());

        // All elements should be present
        for i in 0..3 {
            for val in 1..=3 {
                assert!(cluster.replica(i).state().contains(&val));
            }
        }
    }

    #[test]
    fn test_convergence_with_duplicates() {
        let mut cluster: AntiEntropyCluster<GSet<i32>> =
            AntiEntropyCluster::new(2, NetworkConfig::with_dups(0.5));

        cluster.mutate(0, |_| {
            let mut d = GSet::new();
            d.insert(1);
            d
        });

        cluster.mutate(1, |_| {
            let mut d = GSet::new();
            d.insert(2);
            d
        });

        // Sync multiple times (duplicates should be handled by idempotence)
        for _ in 0..5 {
            cluster.full_sync_round();
        }

        assert!(cluster.is_converged());

        // Both elements present
        assert!(cluster.replica(0).state().contains(&1));
        assert!(cluster.replica(0).state().contains(&2));
    }

    #[test]
    fn test_convergence_chaotic_network() {
        let mut cluster: AntiEntropyCluster<GSet<i32>> =
            AntiEntropyCluster::new(4, NetworkConfig::chaotic());

        // Each replica adds multiple elements
        for i in 0..4 {
            for j in 0..5 {
                let val = (i * 10 + j) as i32;
                cluster.mutate(i, move |_| {
                    let mut d = GSet::new();
                    d.insert(val);
                    d
                });
            }
        }

        // Many sync rounds with retransmission
        for _ in 0..20 {
            cluster.full_sync_round();
            cluster.retransmit_and_process();
        }

        // Should converge eventually
        assert!(cluster.is_converged());

        // Verify all 20 elements are present in all replicas
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..5 {
                    let val = (j * 10 + k) as i32;
                    assert!(
                        cluster.replica(i).state().contains(&val),
                        "Replica {} missing value {}", i, val
                    );
                }
            }
        }
    }

    #[test]
    fn test_idempotence_repeated_resends() {
        let mut cluster: AntiEntropyCluster<GSet<i32>> =
            AntiEntropyCluster::new(2, NetworkConfig::default());

        cluster.mutate(0, |_| {
            let mut d = GSet::new();
            d.insert(42);
            d
        });

        // Initial state
        let initial_state = cluster.replica(1).state().clone();

        // Sync once
        cluster.full_sync_round();
        let after_one = cluster.replica(1).state().clone();

        // Sync many more times (simulating re-sends)
        for _ in 0..10 {
            cluster.full_sync_round();
        }
        let after_many = cluster.replica(1).state().clone();

        // State should be the same after first sync and after many syncs
        assert_eq!(after_one, after_many);

        // But different from initial
        assert_ne!(initial_state, after_one);
    }
}

