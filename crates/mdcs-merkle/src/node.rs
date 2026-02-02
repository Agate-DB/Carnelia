//! Merkle node definition and builder.
//!
//! Each node in the Merkle-DAG contains:
//! - A content identifier (CID) computed from its contents
//! - References to parent nodes (causal predecessors)
//! - A payload (delta-group or snapshot)
//! - A logical timestamp

use crate::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

/// The payload carried by a Merkle node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Payload {
    /// Genesis node - the root of the DAG.
    Genesis,
    
    /// A delta-group containing incremental state changes.
    /// The bytes are a serialized delta from the δ-CRDT layer.
    Delta(Vec<u8>),
    
    /// A snapshot of the full state at a point in time.
    /// Used for compaction and bootstrapping new replicas.
    Snapshot(Vec<u8>),
}

impl Payload {
    /// Create a genesis payload.
    pub fn genesis() -> Self {
        Payload::Genesis
    }

    /// Create a delta payload from serialized bytes.
    pub fn delta(data: Vec<u8>) -> Self {
        Payload::Delta(data)
    }

    /// Create a snapshot payload from serialized bytes.
    pub fn snapshot(data: Vec<u8>) -> Self {
        Payload::Snapshot(data)
    }

    /// Check if this is a genesis payload.
    pub fn is_genesis(&self) -> bool {
        matches!(self, Payload::Genesis)
    }

    /// Check if this is a delta payload.
    pub fn is_delta(&self) -> bool {
        matches!(self, Payload::Delta(_))
    }

    /// Check if this is a snapshot payload.
    pub fn is_snapshot(&self) -> bool {
        matches!(self, Payload::Snapshot(_))
    }

    /// Get the payload data as bytes (returns empty slice for Genesis).
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Payload::Genesis => &[],
            Payload::Delta(data) => data,
            Payload::Snapshot(data) => data,
        }
    }

    /// Get the payload type as a byte for hashing.
    fn type_byte(&self) -> u8 {
        match self {
            Payload::Genesis => 0,
            Payload::Delta(_) => 1,
            Payload::Snapshot(_) => 2,
        }
    }
}

/// A node in the Merkle-DAG representing a causal event.
///
/// The node's CID is computed from its contents, making it content-addressed
/// and tamper-proof. Any change to the node's data would change its CID.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleNode {
    /// Content identifier - SHA-256 hash of the node's contents.
    /// This is computed when the node is built.
    pub cid: Hash,
    
    /// Hashes of parent nodes (causal predecessors).
    /// Empty for genesis nodes.
    pub parents: Vec<Hash>,
    
    /// The payload carried by this node.
    pub payload: Payload,
    
    /// Logical timestamp (monotonically increasing per replica).
    pub timestamp: u64,
    
    /// The replica that created this node.
    pub creator: String,
}

impl MerkleNode {
    /// Check if this is a genesis node.
    pub fn is_genesis(&self) -> bool {
        self.parents.is_empty() && self.payload.is_genesis()
    }

    /// Check if this node is a descendant of another (has it as ancestor).
    /// Note: This only checks direct parents, not transitive ancestry.
    pub fn has_parent(&self, cid: &Hash) -> bool {
        self.parents.contains(cid)
    }

    /// Get the number of parents (branching factor for this node).
    pub fn parent_count(&self) -> usize {
        self.parents.len()
    }

    /// Compute the CID for a node with the given contents.
    /// This is used by the builder to generate the CID.
    fn compute_cid(parents: &[Hash], payload: &Payload, timestamp: u64, creator: &str) -> Hash {
        let mut hasher = Hasher::new();
        
        // Hash the number of parents
        hasher.update(&(parents.len() as u64).to_le_bytes());
        
        // Hash each parent CID (sorted for determinism)
        let mut sorted_parents = parents.to_vec();
        sorted_parents.sort();
        for parent in &sorted_parents {
            hasher.update(parent.as_bytes());
        }
        
        // Hash the payload type and data
        hasher.update(&[payload.type_byte()]);
        hasher.update(payload.as_bytes());
        
        // Hash the timestamp
        hasher.update(&timestamp.to_le_bytes());
        
        // Hash the creator
        hasher.update(creator.as_bytes());
        
        hasher.finalize()
    }

    /// Verify that the CID matches the node's contents.
    pub fn verify(&self) -> bool {
        let computed = Self::compute_cid(&self.parents, &self.payload, self.timestamp, &self.creator);
        computed == self.cid
    }
}

/// Builder for creating Merkle nodes.
#[derive(Clone, Debug, Default)]
pub struct NodeBuilder {
    parents: Vec<Hash>,
    payload: Option<Payload>,
    timestamp: u64,
    creator: String,
}

impl NodeBuilder {
    /// Create a new node builder.
    pub fn new() -> Self {
        NodeBuilder {
            parents: Vec::new(),
            payload: None,
            timestamp: 0,
            creator: String::new(),
        }
    }

    /// Set the parent nodes.
    pub fn with_parents(mut self, parents: Vec<Hash>) -> Self {
        self.parents = parents;
        self
    }

    /// Add a single parent node.
    pub fn with_parent(mut self, parent: Hash) -> Self {
        self.parents.push(parent);
        self
    }

    /// Set the payload.
    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Set the logical timestamp.
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set the creator replica ID.
    pub fn with_creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = creator.into();
        self
    }

    /// Build the node, computing its CID.
    pub fn build(self) -> MerkleNode {
        let payload = self.payload.unwrap_or(Payload::Genesis);
        let cid = MerkleNode::compute_cid(&self.parents, &payload, self.timestamp, &self.creator);
        
        MerkleNode {
            cid,
            parents: self.parents,
            payload,
            timestamp: self.timestamp,
            creator: self.creator,
        }
    }

    /// Build a genesis node for a replica.
    pub fn genesis(creator: impl Into<String>) -> MerkleNode {
        NodeBuilder::new()
            .with_payload(Payload::Genesis)
            .with_timestamp(0)
            .with_creator(creator)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_node() {
        let node = NodeBuilder::genesis("replica_1");
        assert!(node.is_genesis());
        assert!(node.parents.is_empty());
        assert!(node.payload.is_genesis());
        assert!(node.verify());
    }

    #[test]
    fn test_delta_node() {
        let genesis = NodeBuilder::genesis("replica_1");
        
        let delta = NodeBuilder::new()
            .with_parent(genesis.cid)
            .with_payload(Payload::delta(vec![1, 2, 3]))
            .with_timestamp(1)
            .with_creator("replica_1")
            .build();
        
        assert!(!delta.is_genesis());
        assert!(delta.has_parent(&genesis.cid));
        assert!(delta.payload.is_delta());
        assert!(delta.verify());
    }

    #[test]
    fn test_cid_deterministic() {
        let node1 = NodeBuilder::new()
            .with_payload(Payload::delta(vec![1, 2, 3]))
            .with_timestamp(42)
            .with_creator("test")
            .build();
        
        let node2 = NodeBuilder::new()
            .with_payload(Payload::delta(vec![1, 2, 3]))
            .with_timestamp(42)
            .with_creator("test")
            .build();
        
        assert_eq!(node1.cid, node2.cid);
    }

    #[test]
    fn test_cid_changes_with_content() {
        let node1 = NodeBuilder::new()
            .with_payload(Payload::delta(vec![1, 2, 3]))
            .with_timestamp(42)
            .with_creator("test")
            .build();
        
        let node2 = NodeBuilder::new()
            .with_payload(Payload::delta(vec![4, 5, 6]))
            .with_timestamp(42)
            .with_creator("test")
            .build();
        
        assert_ne!(node1.cid, node2.cid);
    }

    #[test]
    fn test_concurrent_parents() {
        let genesis = NodeBuilder::genesis("replica_1");
        
        // Two concurrent branches
        let branch_a = NodeBuilder::new()
            .with_parent(genesis.cid)
            .with_payload(Payload::delta(b"branch_a".to_vec()))
            .with_timestamp(1)
            .with_creator("replica_1")
            .build();
        
        let branch_b = NodeBuilder::new()
            .with_parent(genesis.cid)
            .with_payload(Payload::delta(b"branch_b".to_vec()))
            .with_timestamp(1)
            .with_creator("replica_2")
            .build();
        
        // Merge node with multiple parents
        let merge = NodeBuilder::new()
            .with_parents(vec![branch_a.cid, branch_b.cid])
            .with_payload(Payload::delta(b"merge".to_vec()))
            .with_timestamp(2)
            .with_creator("replica_1")
            .build();
        
        assert_eq!(merge.parent_count(), 2);
        assert!(merge.has_parent(&branch_a.cid));
        assert!(merge.has_parent(&branch_b.cid));
        assert!(merge.verify());
    }

    #[test]
    fn test_snapshot_node() {
        let genesis = NodeBuilder::genesis("replica_1");
        
        let snapshot = NodeBuilder::new()
            .with_parent(genesis.cid)
            .with_payload(Payload::snapshot(b"full state".to_vec()))
            .with_timestamp(100)
            .with_creator("replica_1")
            .build();
        
        assert!(snapshot.payload.is_snapshot());
        assert!(snapshot.verify());
    }

    #[test]
    fn test_verify_tampered_node() {
        let mut node = NodeBuilder::new()
            .with_payload(Payload::delta(vec![1, 2, 3]))
            .with_timestamp(42)
            .with_creator("test")
            .build();
        
        // Tamper with the payload
        node.payload = Payload::delta(vec![9, 9, 9]);
        
        // Verification should fail
        assert!(!node.verify());
    }
}
