//! DAG synchronization with gap-repair logic.
//!
//! The DAGSyncer handles reconciliation between replicas by:
//! 1. Discovering missing nodes via head comparison
//! 2. Fetching missing nodes from peers recursively
//! 3. Handling concurrent heads (multi-root scenarios)

use crate::hash::Hash;
use crate::node::MerkleNode;
use crate::store::{DAGError, DAGStore};
use std::collections::{HashSet, VecDeque};

/// Errors that can occur during synchronization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncError {
    /// Failed to fetch a node from peers.
    FetchFailed(Hash),

    /// Node verification failed.
    VerificationFailed(Hash),

    /// DAG store error.
    StoreError(DAGError),

    /// No peers available for sync.
    NoPeers,

    /// Sync timeout.
    Timeout,

    /// Maximum depth exceeded during traversal.
    MaxDepthExceeded,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::FetchFailed(h) => write!(f, "Failed to fetch node: {}", h.short()),
            SyncError::VerificationFailed(h) => write!(f, "Verification failed: {}", h.short()),
            SyncError::StoreError(e) => write!(f, "Store error: {}", e),
            SyncError::NoPeers => write!(f, "No peers available"),
            SyncError::Timeout => write!(f, "Sync timeout"),
            SyncError::MaxDepthExceeded => write!(f, "Maximum traversal depth exceeded"),
        }
    }
}

impl std::error::Error for SyncError {}

impl From<DAGError> for SyncError {
    fn from(e: DAGError) -> Self {
        SyncError::StoreError(e)
    }
}

/// Request to fetch nodes from a peer.
#[derive(Clone, Debug)]
pub struct SyncRequest {
    /// CIDs of nodes we need.
    pub want: Vec<Hash>,

    /// Our current heads (for the peer to determine what to send).
    pub have: Vec<Hash>,

    /// Maximum number of nodes to return.
    pub limit: Option<usize>,
}

impl SyncRequest {
    /// Create a new sync request for specific nodes.
    pub fn want(cids: Vec<Hash>) -> Self {
        SyncRequest {
            want: cids,
            have: Vec::new(),
            limit: None,
        }
    }

    /// Create a sync request with our current heads.
    pub fn with_heads(mut self, heads: Vec<Hash>) -> Self {
        self.have = heads;
        self
    }

    /// Limit the response size.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Response containing nodes from a peer.
#[derive(Clone, Debug)]
pub struct SyncResponse {
    /// Nodes being sent.
    pub nodes: Vec<MerkleNode>,

    /// Additional nodes that could be sent (pagination).
    pub more: Vec<Hash>,

    /// Peer's current heads.
    pub heads: Vec<Hash>,
}

impl SyncResponse {
    /// Create an empty response.
    pub fn empty() -> Self {
        SyncResponse {
            nodes: Vec::new(),
            more: Vec::new(),
            heads: Vec::new(),
        }
    }

    /// Create a response with nodes.
    pub fn with_nodes(nodes: Vec<MerkleNode>) -> Self {
        SyncResponse {
            nodes,
            more: Vec::new(),
            heads: Vec::new(),
        }
    }
}

/// Configuration for the DAG syncer.
#[derive(Clone, Debug)]
pub struct SyncConfig {
    /// Maximum depth to traverse when fetching missing ancestors.
    pub max_depth: usize,

    /// Maximum number of nodes to fetch in a single request.
    pub batch_size: usize,

    /// Whether to verify nodes before storing.
    pub verify_nodes: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        SyncConfig {
            max_depth: 1000,
            batch_size: 100,
            verify_nodes: true,
        }
    }
}

/// DAG synchronizer for gap-repair and reconciliation.
///
/// The syncer uses a pull-based approach:
/// 1. Compare heads with a peer
/// 2. Identify missing nodes
/// 3. Fetch missing nodes recursively until reaching common ancestors
pub struct DAGSyncer<S: DAGStore> {
    /// The local DAG store.
    store: S,

    /// Configuration.
    config: SyncConfig,
}

impl<S: DAGStore> DAGSyncer<S> {
    /// Create a new syncer with a store.
    pub fn new(store: S) -> Self {
        DAGSyncer {
            store,
            config: SyncConfig::default(),
        }
    }

    /// Create a syncer with custom configuration.
    pub fn with_config(store: S, config: SyncConfig) -> Self {
        DAGSyncer { store, config }
    }

    /// Get a reference to the store.
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Get a mutable reference to the store.
    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }

    /// Get our current heads.
    pub fn heads(&self) -> Vec<Hash> {
        self.store.heads()
    }

    /// Determine which of the given CIDs we need (don't have locally).
    pub fn need(&self, cids: &[Hash]) -> Vec<Hash> {
        cids.iter()
            .filter(|cid| !self.store.contains(cid))
            .copied()
            .collect()
    }

    /// Create a sync request for reconciliation with a peer.
    pub fn create_request(&self, peer_heads: &[Hash]) -> SyncRequest {
        let need = self.need(peer_heads);
        SyncRequest::want(need)
            .with_heads(self.heads())
            .with_limit(self.config.batch_size)
    }

    /// Handle an incoming sync request from a peer.
    pub fn handle_request(&self, request: &SyncRequest) -> SyncResponse {
        let mut nodes = Vec::new();
        let mut more = Vec::new();
        let limit = request.limit.unwrap_or(self.config.batch_size);

        // Collect requested nodes
        for cid in &request.want {
            if let Some(node) = self.store.get(cid) {
                if nodes.len() < limit {
                    nodes.push(node.clone());
                } else {
                    more.push(*cid);
                }
            }
        }

        // If peer provided their heads, we can proactively send nodes they're missing
        if !request.have.is_empty() && nodes.len() < limit {
            let peer_has: HashSet<_> = self.collect_known(&request.have);

            // Find nodes we have that the peer doesn't
            for cid in self.store.topological_order() {
                if !peer_has.contains(&cid) {
                    if let Some(node) = self.store.get(&cid) {
                        if nodes.len() < limit {
                            // Check if peer has the parents
                            let has_parents = node
                                .parents
                                .iter()
                                .all(|p| peer_has.contains(p) || nodes.iter().any(|n| n.cid == *p));

                            if has_parents && !nodes.iter().any(|n| n.cid == cid) {
                                nodes.push(node.clone());
                            }
                        } else {
                            more.push(cid);
                        }
                    }
                }
            }
        }

        SyncResponse {
            nodes,
            more,
            heads: self.heads(),
        }
    }

    /// Apply a sync response, storing received nodes.
    ///
    /// Returns the CIDs of successfully stored nodes.
    pub fn apply_response(&mut self, response: SyncResponse) -> Result<Vec<Hash>, SyncError> {
        let mut stored = Vec::new();
        let mut pending: VecDeque<MerkleNode> = response.nodes.into_iter().collect();
        let mut attempts = 0;
        let max_attempts = pending.len() * 2;

        // Process nodes, retrying if parents aren't available yet
        while let Some(node) = pending.pop_front() {
            attempts += 1;
            if attempts > max_attempts {
                break;
            }

            if self.store.contains(&node.cid) {
                stored.push(node.cid);
                continue;
            }

            if self.config.verify_nodes && !node.verify() {
                return Err(SyncError::VerificationFailed(node.cid));
            }

            // Try to store with parent check
            match self.store.put(node.clone()) {
                Ok(cid) => stored.push(cid),
                Err(DAGError::MissingParents(_)) => {
                    // Parents not yet available, retry later
                    pending.push_back(node);
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(stored)
    }

    /// Apply nodes without strict parent checking.
    ///
    /// Used when nodes may arrive out of order.
    pub fn apply_nodes_unchecked(
        &mut self,
        nodes: Vec<MerkleNode>,
    ) -> Result<Vec<Hash>, SyncError> {
        let mut stored = Vec::new();

        for node in nodes {
            if self.config.verify_nodes && !node.verify() {
                return Err(SyncError::VerificationFailed(node.cid));
            }

            let cid = self.store.put_unchecked(node)?;
            stored.push(cid);
        }

        Ok(stored)
    }

    /// Collect all CIDs reachable from the given heads (including the heads).
    fn collect_known(&self, heads: &[Hash]) -> HashSet<Hash> {
        let mut known = HashSet::new();
        let mut queue: VecDeque<Hash> = heads.iter().copied().collect();

        while let Some(cid) = queue.pop_front() {
            if known.insert(cid) {
                if let Some(node) = self.store.get(&cid) {
                    queue.extend(node.parents.iter().copied());
                }
            }
        }

        known
    }

    /// Find the missing ancestors of the given CIDs.
    ///
    /// This performs gap detection by traversing backwards from the given CIDs
    /// and identifying nodes that aren't in our store.
    pub fn find_missing_ancestors(&self, cids: &[Hash]) -> Vec<Hash> {
        let mut missing = Vec::new();
        let mut visited = HashSet::new();
        let mut queue: VecDeque<(Hash, usize)> = cids.iter().map(|cid| (*cid, 0)).collect();

        while let Some((cid, depth)) = queue.pop_front() {
            if depth > self.config.max_depth {
                continue;
            }

            if !visited.insert(cid) {
                continue;
            }

            if !self.store.contains(&cid) {
                missing.push(cid);
            } else if let Some(node) = self.store.get(&cid) {
                // Traverse to parents
                for parent in &node.parents {
                    if !visited.contains(parent) {
                        queue.push_back((*parent, depth + 1));
                    }
                }
            }
        }

        missing
    }

    /// Check if we're synchronized with a peer (have all their nodes).
    pub fn is_synced_with(&self, peer_heads: &[Hash]) -> bool {
        // We're synced if we have all peer heads and their ancestors
        for head in peer_heads {
            if !self.store.contains(head) {
                return false;
            }
        }

        // Check we have no missing nodes
        self.store.missing_nodes().is_empty()
    }

    /// Get statistics about sync status.
    pub fn sync_status(&self) -> SyncStatus {
        SyncStatus {
            local_heads: self.heads().len(),
            missing_nodes: self.store.missing_nodes().len(),
            total_nodes: self.store.len(),
        }
    }
}

/// Status information about synchronization.
#[derive(Clone, Debug)]
pub struct SyncStatus {
    pub local_heads: usize,
    pub missing_nodes: usize,
    pub total_nodes: usize,
}

/// Simulator for testing sync between multiple replicas.
pub struct SyncSimulator {
    /// Syncers for each replica.
    syncers: Vec<DAGSyncer<crate::store::MemoryDAGStore>>,
}

impl SyncSimulator {
    /// Create a simulator with n replicas, each with their own genesis.
    pub fn new(n: usize) -> Self {
        let syncers = (0..n)
            .map(|i| {
                let (store, _) =
                    crate::store::MemoryDAGStore::with_genesis(format!("replica_{}", i));
                DAGSyncer::new(store)
            })
            .collect();

        SyncSimulator { syncers }
    }

    /// Create a simulator where all replicas share the same genesis.
    pub fn with_shared_genesis(n: usize) -> Self {
        let genesis = crate::node::NodeBuilder::genesis("shared");
        let genesis_cid = genesis.cid;

        let syncers = (0..n)
            .map(|_| {
                let mut store = crate::store::MemoryDAGStore::new();
                store.put(genesis.clone()).unwrap();
                DAGSyncer::new(store)
            })
            .collect();

        let _ = genesis_cid; // Used in shared setup
        SyncSimulator { syncers }
    }

    /// Get a reference to a syncer.
    pub fn syncer(&self, idx: usize) -> &DAGSyncer<crate::store::MemoryDAGStore> {
        &self.syncers[idx]
    }

    /// Get a mutable reference to a syncer.
    pub fn syncer_mut(&mut self, idx: usize) -> &mut DAGSyncer<crate::store::MemoryDAGStore> {
        &mut self.syncers[idx]
    }

    /// Perform one sync round between two replicas.
    pub fn sync_pair(&mut self, from: usize, to: usize) {
        let from_heads = self.syncers[from].heads();
        let request = self.syncers[to].create_request(&from_heads);
        let response = self.syncers[from].handle_request(&request);
        let _ = self.syncers[to].apply_response(response);
    }

    /// Perform a full sync round (all pairs).
    pub fn full_sync_round(&mut self) {
        let n = self.syncers.len();
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    self.sync_pair(i, j);
                }
            }
        }
    }

    /// Check if all replicas have converged (same heads).
    pub fn is_converged(&self) -> bool {
        if self.syncers.is_empty() {
            return true;
        }

        let reference_heads: HashSet<_> = self.syncers[0].heads().into_iter().collect();

        self.syncers.iter().skip(1).all(|s| {
            let heads: HashSet<_> = s.heads().into_iter().collect();
            heads == reference_heads
        })
    }

    /// Get the number of replicas.
    pub fn replica_count(&self) -> usize {
        self.syncers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{NodeBuilder, Payload};
    use crate::store::MemoryDAGStore;

    #[test]
    fn test_basic_sync() {
        let mut sim = SyncSimulator::with_shared_genesis(2);

        // Add a node to replica 0
        let heads = sim.syncer(0).heads();
        let node = NodeBuilder::new()
            .with_parent(heads[0])
            .with_payload(Payload::delta(vec![1, 2, 3]))
            .with_timestamp(1)
            .with_creator("replica_0")
            .build();
        sim.syncer_mut(0).store_mut().put(node).unwrap();

        // Before sync
        assert!(!sim.is_converged());

        // Sync
        sim.sync_pair(0, 1);

        // After sync
        assert!(sim.is_converged());
    }

    #[test]
    fn test_concurrent_updates_sync() {
        let mut sim = SyncSimulator::with_shared_genesis(2);
        let genesis = sim.syncer(0).heads()[0];

        // Both replicas add concurrent nodes
        let node_a = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(b"from_0".to_vec()))
            .with_timestamp(1)
            .with_creator("replica_0")
            .build();
        sim.syncer_mut(0).store_mut().put(node_a).unwrap();

        let node_b = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(b"from_1".to_vec()))
            .with_timestamp(1)
            .with_creator("replica_1")
            .build();
        sim.syncer_mut(1).store_mut().put(node_b).unwrap();

        // Each has 2 nodes, but different heads
        assert_eq!(sim.syncer(0).store().len(), 2);
        assert_eq!(sim.syncer(1).store().len(), 2);
        assert!(!sim.is_converged());

        // Sync both ways
        sim.full_sync_round();

        // Now both have 3 nodes and same heads
        assert_eq!(sim.syncer(0).store().len(), 3);
        assert_eq!(sim.syncer(1).store().len(), 3);

        // Both should have 2 heads (the concurrent updates)
        assert_eq!(sim.syncer(0).heads().len(), 2);
        assert!(sim.is_converged());
    }

    #[test]
    fn test_find_missing_ancestors() {
        let (mut store, genesis) = MemoryDAGStore::with_genesis("r1");

        // Build a chain: genesis -> a -> b -> c
        let node_a = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(vec![1]))
            .with_timestamp(1)
            .with_creator("r1")
            .build();
        let cid_a = store.put(node_a.clone()).unwrap();

        let node_b = NodeBuilder::new()
            .with_parent(cid_a)
            .with_payload(Payload::delta(vec![2]))
            .with_timestamp(2)
            .with_creator("r1")
            .build();
        let cid_b = store.put(node_b.clone()).unwrap();

        let node_c = NodeBuilder::new()
            .with_parent(cid_b)
            .with_payload(Payload::delta(vec![3]))
            .with_timestamp(3)
            .with_creator("r1")
            .build();
        let cid_c = node_c.cid;
        store.put(node_c).unwrap();

        // Create another store with only genesis
        let (store2, _) = MemoryDAGStore::with_genesis("r1");
        let syncer = DAGSyncer::new(store2);

        // Find missing from perspective of store2
        let missing = syncer.find_missing_ancestors(&[cid_c]);

        // Should find cid_c as missing (we don't have it)
        assert!(missing.contains(&cid_c));
    }

    #[test]
    fn test_sync_request_response() {
        let (mut store, genesis) = MemoryDAGStore::with_genesis("r1");

        let node = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(vec![1]))
            .with_timestamp(1)
            .with_creator("r1")
            .build();
        let cid = store.put(node).unwrap();

        let syncer = DAGSyncer::new(store);

        // Create a request asking for the node
        let request = SyncRequest::want(vec![cid]);
        let response = syncer.handle_request(&request);

        assert_eq!(response.nodes.len(), 1);
        assert_eq!(response.nodes[0].cid, cid);
    }

    #[test]
    fn test_apply_response() {
        let (_store1, genesis) = MemoryDAGStore::with_genesis("r1");

        let node = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(vec![1]))
            .with_timestamp(1)
            .with_creator("r1")
            .build();
        let cid = node.cid;

        // Store2 doesn't have the node
        let (store2, _) = MemoryDAGStore::with_genesis("r1");
        let mut syncer2 = DAGSyncer::new(store2);

        // Apply a response containing the node
        let response = SyncResponse::with_nodes(vec![node]);
        let stored = syncer2.apply_response(response).unwrap();

        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0], cid);
        assert!(syncer2.store().contains(&cid));
    }

    #[test]
    fn test_is_synced_with() {
        let mut sim = SyncSimulator::with_shared_genesis(2);
        let genesis = sim.syncer(0).heads()[0];

        // Initially synced (both have only genesis)
        assert!(sim.syncer(0).is_synced_with(&sim.syncer(1).heads()));

        // Add node to replica 0
        let node = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(vec![1]))
            .with_timestamp(1)
            .with_creator("r0")
            .build();
        sim.syncer_mut(0).store_mut().put(node).unwrap();

        // Replica 1 is not synced with replica 0's heads
        assert!(!sim.syncer(1).is_synced_with(&sim.syncer(0).heads()));

        // After sync, they should be synced
        sim.sync_pair(0, 1);
        assert!(sim.syncer(1).is_synced_with(&sim.syncer(0).heads()));
    }
}
