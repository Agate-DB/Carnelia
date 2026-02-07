//! DAG storage trait and implementations.
//!
//! The DAGStore provides content-addressed storage for Merkle nodes,
//! tracking heads (nodes without children) automatically.

use crate::hash::Hash;
use crate::node::MerkleNode;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Errors that can occur during DAG operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DAGError {
    /// Node not found in the store.
    NotFound(Hash),

    /// Node failed verification (CID doesn't match contents).
    VerificationFailed(Hash),

    /// Missing parent nodes (gap in the DAG).
    MissingParents(Vec<Hash>),

    /// Duplicate node (already exists).
    Duplicate(Hash),
}

impl std::fmt::Display for DAGError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DAGError::NotFound(h) => write!(f, "Node not found: {}", h.short()),
            DAGError::VerificationFailed(h) => write!(f, "Verification failed for: {}", h.short()),
            DAGError::MissingParents(parents) => {
                write!(
                    f,
                    "Missing parents: {:?}",
                    parents.iter().map(|h| h.short()).collect::<Vec<_>>()
                )
            }
            DAGError::Duplicate(h) => write!(f, "Duplicate node: {}", h.short()),
        }
    }
}

impl std::error::Error for DAGError {}

/// Trait for content-addressed DAG storage.
pub trait DAGStore {
    /// Get a node by its CID.
    fn get(&self, cid: &Hash) -> Option<&MerkleNode>;

    /// Store a node, returning its CID.
    ///
    /// The node's CID is verified before storage.
    /// Returns an error if verification fails or parents are missing.
    fn put(&mut self, node: MerkleNode) -> Result<Hash, DAGError>;

    /// Store a node without checking for missing parents.
    ///
    /// Used during sync when parents may arrive out of order.
    fn put_unchecked(&mut self, node: MerkleNode) -> Result<Hash, DAGError>;

    /// Get the current heads (nodes without children).
    fn heads(&self) -> Vec<Hash>;

    /// Check if a node exists in the store.
    fn contains(&self, cid: &Hash) -> bool;

    /// Get all ancestors of a node (transitive closure).
    fn ancestors(&self, cid: &Hash) -> HashSet<Hash>;

    /// Get immediate children of a node.
    fn children(&self, cid: &Hash) -> Vec<Hash>;

    /// Get all nodes in topological order (parents before children).
    fn topological_order(&self) -> Vec<Hash>;

    /// Get nodes that are missing (referenced but not present).
    fn missing_nodes(&self) -> HashSet<Hash>;

    /// Get the total number of nodes.
    fn len(&self) -> usize;

    /// Check if the store is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// In-memory implementation of DAGStore.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryDAGStore {
    /// All nodes indexed by CID.
    nodes: HashMap<Hash, MerkleNode>,

    /// Current heads (nodes without children).
    heads: HashSet<Hash>,

    /// Reverse index: parent -> children.
    children_index: HashMap<Hash, HashSet<Hash>>,

    /// Referenced but missing nodes.
    missing: HashSet<Hash>,
}

impl MemoryDAGStore {
    /// Create a new empty DAG store.
    pub fn new() -> Self {
        MemoryDAGStore {
            nodes: HashMap::new(),
            heads: HashSet::new(),
            children_index: HashMap::new(),
            missing: HashSet::new(),
        }
    }

    /// Create a store with a genesis node.
    pub fn with_genesis(creator: impl Into<String>) -> (Self, Hash) {
        let mut store = Self::new();
        let genesis = crate::node::NodeBuilder::genesis(creator);
        let cid = store.put(genesis).expect("Genesis node should be valid");
        (store, cid)
    }

    /// Update the heads set after adding a node.
    fn update_heads(&mut self, node: &MerkleNode) {
        // The new node becomes a head
        self.heads.insert(node.cid);

        // Its parents are no longer heads
        for parent in &node.parents {
            self.heads.remove(parent);
        }
    }

    /// Update the children index after adding a node.
    fn update_children_index(&mut self, node: &MerkleNode) {
        for parent in &node.parents {
            self.children_index
                .entry(*parent)
                .or_default()
                .insert(node.cid);
        }
    }

    /// Get statistics about the DAG.
    pub fn stats(&self) -> DAGStats {
        let max_depth = self.compute_max_depth();
        let branching = self.compute_branching_stats();

        DAGStats {
            total_nodes: self.nodes.len(),
            head_count: self.heads.len(),
            missing_count: self.missing.len(),
            max_depth,
            avg_branching: branching,
        }
    }

    /// Compute the maximum depth of the DAG.
    fn compute_max_depth(&self) -> usize {
        let mut depths: HashMap<Hash, usize> = HashMap::new();

        for cid in self.topological_order() {
            if let Some(node) = self.nodes.get(&cid) {
                let parent_depth = node
                    .parents
                    .iter()
                    .filter_map(|p| depths.get(p))
                    .max()
                    .copied()
                    .unwrap_or(0);
                depths.insert(cid, parent_depth + 1);
            }
        }

        depths.values().max().copied().unwrap_or(0)
    }

    /// Compute average branching factor.
    fn compute_branching_stats(&self) -> f64 {
        if self.children_index.is_empty() {
            return 0.0;
        }

        let total_children: usize = self.children_index.values().map(|c| c.len()).sum();

        total_children as f64 / self.children_index.len() as f64
    }
}

impl DAGStore for MemoryDAGStore {
    fn get(&self, cid: &Hash) -> Option<&MerkleNode> {
        self.nodes.get(cid)
    }

    fn put(&mut self, node: MerkleNode) -> Result<Hash, DAGError> {
        // Verify the node's CID
        if !node.verify() {
            return Err(DAGError::VerificationFailed(node.cid));
        }

        // Check if already exists
        if self.nodes.contains_key(&node.cid) {
            return Ok(node.cid);
        }

        // Check for missing parents (unless this is a genesis node)
        if !node.is_genesis() {
            let missing: Vec<Hash> = node
                .parents
                .iter()
                .filter(|p| !self.nodes.contains_key(p))
                .copied()
                .collect();

            if !missing.is_empty() {
                return Err(DAGError::MissingParents(missing));
            }
        }

        let cid = node.cid;

        // Update indices
        self.update_heads(&node);
        self.update_children_index(&node);

        // Remove from missing if it was there
        self.missing.remove(&cid);

        // Store the node
        self.nodes.insert(cid, node);

        Ok(cid)
    }

    fn put_unchecked(&mut self, node: MerkleNode) -> Result<Hash, DAGError> {
        // Verify the node's CID
        if !node.verify() {
            return Err(DAGError::VerificationFailed(node.cid));
        }

        // Check if already exists
        if self.nodes.contains_key(&node.cid) {
            return Ok(node.cid);
        }

        let cid = node.cid;

        // Track missing parents
        for parent in &node.parents {
            if !self.nodes.contains_key(parent) {
                self.missing.insert(*parent);
            }
        }

        // Update children index FIRST (before heads update)
        self.update_children_index(&node);

        // Update heads - but only add this node as a head if it has no children
        // (This handles out-of-order insertion where children arrive before parents)
        if !self.children_index.contains_key(&cid) {
            self.heads.insert(cid);
        }
        // Its parents are no longer heads (if they were)
        for parent in &node.parents {
            self.heads.remove(parent);
        }

        // Remove from missing if it was there
        self.missing.remove(&cid);

        // Store the node
        self.nodes.insert(cid, node);

        Ok(cid)
    }

    fn heads(&self) -> Vec<Hash> {
        let mut heads: Vec<_> = self.heads.iter().copied().collect();
        heads.sort();
        heads
    }

    fn contains(&self, cid: &Hash) -> bool {
        self.nodes.contains_key(cid)
    }

    fn ancestors(&self, cid: &Hash) -> HashSet<Hash> {
        let mut result = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(node) = self.nodes.get(cid) {
            queue.extend(node.parents.iter().copied());
        }

        while let Some(current) = queue.pop_front() {
            if result.insert(current) {
                if let Some(node) = self.nodes.get(&current) {
                    queue.extend(node.parents.iter().copied());
                }
            }
        }

        result
    }

    fn children(&self, cid: &Hash) -> Vec<Hash> {
        self.children_index
            .get(cid)
            .map(|c| c.iter().copied().collect())
            .unwrap_or_default()
    }

    fn topological_order(&self) -> Vec<Hash> {
        // Kahn's algorithm for topological sort
        let mut in_degree: HashMap<Hash, usize> = HashMap::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        // Calculate in-degrees (number of parents in the store)
        for (cid, node) in &self.nodes {
            let degree = node
                .parents
                .iter()
                .filter(|p| self.nodes.contains_key(p))
                .count();
            in_degree.insert(*cid, degree);

            if degree == 0 {
                queue.push_back(*cid);
            }
        }

        // Process nodes with no dependencies
        while let Some(cid) = queue.pop_front() {
            result.push(cid);

            if let Some(children) = self.children_index.get(&cid) {
                for child in children {
                    if let Some(degree) = in_degree.get_mut(child) {
                        *degree = degree.saturating_sub(1);
                        if *degree == 0 {
                            queue.push_back(*child);
                        }
                    }
                }
            }
        }

        result
    }

    fn missing_nodes(&self) -> HashSet<Hash> {
        self.missing.clone()
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }
}

/// Statistics about a DAG.
#[derive(Clone, Debug)]
pub struct DAGStats {
    pub total_nodes: usize,
    pub head_count: usize,
    pub missing_count: usize,
    pub max_depth: usize,
    pub avg_branching: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{NodeBuilder, Payload};

    #[test]
    fn test_genesis_store() {
        let (store, genesis_cid) = MemoryDAGStore::with_genesis("replica_1");

        assert_eq!(store.len(), 1);
        assert!(store.contains(&genesis_cid));
        assert_eq!(store.heads(), vec![genesis_cid]);
    }

    #[test]
    fn test_linear_chain() {
        let (mut store, genesis) = MemoryDAGStore::with_genesis("r1");

        let node1 = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(vec![1]))
            .with_timestamp(1)
            .with_creator("r1")
            .build();
        let cid1 = store.put(node1).unwrap();

        let node2 = NodeBuilder::new()
            .with_parent(cid1)
            .with_payload(Payload::delta(vec![2]))
            .with_timestamp(2)
            .with_creator("r1")
            .build();
        let cid2 = store.put(node2).unwrap();

        assert_eq!(store.len(), 3);
        assert_eq!(store.heads(), vec![cid2]);
        assert_eq!(store.ancestors(&cid2), HashSet::from([genesis, cid1]));
    }

    #[test]
    fn test_concurrent_branches() {
        let (mut store, genesis) = MemoryDAGStore::with_genesis("r1");

        // Two concurrent branches
        let branch_a = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(b"a".to_vec()))
            .with_timestamp(1)
            .with_creator("r1")
            .build();
        let cid_a = store.put(branch_a).unwrap();

        let branch_b = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(b"b".to_vec()))
            .with_timestamp(1)
            .with_creator("r2")
            .build();
        let cid_b = store.put(branch_b).unwrap();

        // Both should be heads
        let heads = store.heads();
        assert_eq!(heads.len(), 2);
        assert!(heads.contains(&cid_a));
        assert!(heads.contains(&cid_b));
    }

    #[test]
    fn test_merge_node() {
        let (mut store, genesis) = MemoryDAGStore::with_genesis("r1");

        let branch_a = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(b"a".to_vec()))
            .with_timestamp(1)
            .with_creator("r1")
            .build();
        let cid_a = store.put(branch_a).unwrap();

        let branch_b = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(b"b".to_vec()))
            .with_timestamp(1)
            .with_creator("r2")
            .build();
        let cid_b = store.put(branch_b).unwrap();

        // Merge node
        let merge = NodeBuilder::new()
            .with_parents(vec![cid_a, cid_b])
            .with_payload(Payload::delta(b"merge".to_vec()))
            .with_timestamp(2)
            .with_creator("r1")
            .build();
        let merge_cid = store.put(merge).unwrap();

        // Only merge should be a head now
        assert_eq!(store.heads(), vec![merge_cid]);

        // Ancestors should include both branches and genesis
        let ancestors = store.ancestors(&merge_cid);
        assert!(ancestors.contains(&cid_a));
        assert!(ancestors.contains(&cid_b));
        assert!(ancestors.contains(&genesis));
    }

    #[test]
    fn test_missing_parents_error() {
        let mut store = MemoryDAGStore::new();

        let fake_parent = crate::hash::Hasher::hash(b"fake");

        let node = NodeBuilder::new()
            .with_parent(fake_parent)
            .with_payload(Payload::delta(vec![1]))
            .with_timestamp(1)
            .with_creator("r1")
            .build();

        let result = store.put(node);
        assert!(matches!(result, Err(DAGError::MissingParents(_))));
    }

    #[test]
    fn test_put_unchecked() {
        let mut store = MemoryDAGStore::new();

        let fake_parent = crate::hash::Hasher::hash(b"fake");

        let node = NodeBuilder::new()
            .with_parent(fake_parent)
            .with_payload(Payload::delta(vec![1]))
            .with_timestamp(1)
            .with_creator("r1")
            .build();

        // Should succeed with put_unchecked
        let cid = store.put_unchecked(node).unwrap();

        // Should track the missing parent
        assert!(store.missing_nodes().contains(&fake_parent));
        assert!(store.contains(&cid));
    }

    #[test]
    fn test_topological_order() {
        let (mut store, genesis) = MemoryDAGStore::with_genesis("r1");

        let node1 = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(vec![1]))
            .with_timestamp(1)
            .with_creator("r1")
            .build();
        let cid1 = store.put(node1).unwrap();

        let node2 = NodeBuilder::new()
            .with_parent(cid1)
            .with_payload(Payload::delta(vec![2]))
            .with_timestamp(2)
            .with_creator("r1")
            .build();
        let cid2 = store.put(node2).unwrap();

        let order = store.topological_order();

        // Genesis should come before node1, node1 before node2
        let genesis_pos = order.iter().position(|&c| c == genesis).unwrap();
        let cid1_pos = order.iter().position(|&c| c == cid1).unwrap();
        let cid2_pos = order.iter().position(|&c| c == cid2).unwrap();

        assert!(genesis_pos < cid1_pos);
        assert!(cid1_pos < cid2_pos);
    }

    #[test]
    fn test_children_index() {
        let (mut store, genesis) = MemoryDAGStore::with_genesis("r1");

        let child1 = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(vec![1]))
            .with_timestamp(1)
            .with_creator("r1")
            .build();
        let cid1 = store.put(child1).unwrap();

        let child2 = NodeBuilder::new()
            .with_parent(genesis)
            .with_payload(Payload::delta(vec![2]))
            .with_timestamp(1)
            .with_creator("r2")
            .build();
        let cid2 = store.put(child2).unwrap();

        let children = store.children(&genesis);
        assert_eq!(children.len(), 2);
        assert!(children.contains(&cid1));
        assert!(children.contains(&cid2));
    }

    #[test]
    fn test_dag_stats() {
        let (mut store, _genesis) = MemoryDAGStore::with_genesis("r1");

        for i in 0..5 {
            let last_head = store.heads()[0];
            let node = NodeBuilder::new()
                .with_parent(last_head)
                .with_payload(Payload::delta(vec![i]))
                .with_timestamp(i as u64 + 1)
                .with_creator("r1")
                .build();
            store.put(node).unwrap();
        }

        let stats = store.stats();
        assert_eq!(stats.total_nodes, 6);
        assert_eq!(stats.head_count, 1);
        assert_eq!(stats.max_depth, 6);
    }
}
