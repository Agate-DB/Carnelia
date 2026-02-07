//! RGA List - Replicated Growable Array for ordered sequences.
//!
//! RGA provides a CRDT list that supports:
//! - Insert at any position
//! - Delete at any position
//! - Move elements (delete + insert)
//!
//! Uses unique IDs to maintain consistent ordering across replicas.

use mdcs_core::lattice::Lattice;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

/// Unique identifier for a list element.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ListId {
    /// The replica that created this element.
    pub replica: String,
    /// Sequence number within that replica.
    pub seq: u64,
    /// Unique identifier for disambiguation.
    pub ulid: Ulid,
}

impl ListId {
    pub fn new(replica: impl Into<String>, seq: u64) -> Self {
        Self {
            replica: replica.into(),
            seq,
            ulid: Ulid::new(),
        }
    }

    /// Create a genesis ID (for the virtual head).
    pub fn genesis() -> Self {
        Self {
            replica: "".to_string(),
            seq: 0,
            ulid: Ulid::nil(),
        }
    }
}

impl PartialOrd for ListId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ListId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher sequence = later in causal order
        // Tie-break on replica ID, then ULID
        self.seq
            .cmp(&other.seq)
            .then_with(|| self.replica.cmp(&other.replica))
            .then_with(|| self.ulid.cmp(&other.ulid))
    }
}

/// A node in the RGA list.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListNode<T> {
    /// The unique ID of this node.
    pub id: ListId,
    /// The value stored (None if deleted - tombstone).
    pub value: Option<T>,
    /// The ID of the element this was inserted after.
    pub origin: ListId,
    /// Whether this node is deleted (tombstone).
    pub deleted: bool,
}

impl<T> ListNode<T> {
    pub fn new(id: ListId, value: T, origin: ListId) -> Self {
        Self {
            id,
            value: Some(value),
            origin,
            deleted: false,
        }
    }
}

/// Delta for RGA list operations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RGAListDelta<T: Clone + PartialEq> {
    /// Nodes to insert.
    pub inserts: Vec<ListNode<T>>,
    /// IDs of nodes to delete.
    pub deletes: Vec<ListId>,
}

impl<T: Clone + PartialEq> RGAListDelta<T> {
    pub fn new() -> Self {
        Self {
            inserts: Vec::new(),
            deletes: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inserts.is_empty() && self.deletes.is_empty()
    }
}

impl<T: Clone + PartialEq> Default for RGAListDelta<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Replicated Growable Array - an ordered list CRDT.
///
/// Supports insert, delete, and move operations with
/// deterministic conflict resolution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RGAList<T: Clone + PartialEq> {
    /// All nodes indexed by their ID.
    nodes: HashMap<ListId, ListNode<T>>,
    /// Children of each node (for ordering).
    /// Maps origin -> list of children sorted by ID.
    children: HashMap<ListId, Vec<ListId>>,
    /// The replica ID for this instance.
    replica_id: String,
    /// Sequence counter for generating IDs.
    seq: u64,
    /// Pending delta for replication.
    #[serde(skip)]
    pending_delta: Option<RGAListDelta<T>>,
}

impl<T: Clone + PartialEq> RGAList<T> {
    /// Create a new empty RGA list.
    pub fn new(replica_id: impl Into<String>) -> Self {
        let replica_id = replica_id.into();
        let mut list = Self {
            nodes: HashMap::new(),
            children: HashMap::new(),
            replica_id,
            seq: 0,
            pending_delta: None,
        };

        // Insert virtual head node
        let genesis = ListId::genesis();
        list.children.insert(genesis, Vec::new());

        list
    }

    /// Get the replica ID.
    pub fn replica_id(&self) -> &str {
        &self.replica_id
    }

    /// Generate a new unique ID.
    fn next_id(&mut self) -> ListId {
        self.seq += 1;
        ListId::new(&self.replica_id, self.seq)
    }

    /// Insert a value at the given index.
    pub fn insert(&mut self, index: usize, value: T) {
        let origin = self
            .id_at_index(index.saturating_sub(1))
            .unwrap_or(ListId::genesis());
        self.insert_after(&origin, value);
    }

    /// Insert a value after the given origin ID.
    pub fn insert_after(&mut self, origin: &ListId, value: T) {
        let id = self.next_id();
        let node = ListNode::new(id.clone(), value, origin.clone());

        self.integrate_node(node.clone());

        // Record delta
        let delta = self.pending_delta.get_or_insert_with(RGAListDelta::new);
        delta.inserts.push(node);
    }

    /// Insert at the beginning.
    pub fn push_front(&mut self, value: T) {
        self.insert(0, value);
    }

    /// Insert at the end.
    pub fn push_back(&mut self, value: T) {
        let len = self.len();
        self.insert(len, value);
    }

    /// Delete the element at the given index.
    pub fn delete(&mut self, index: usize) -> Option<T> {
        let id = self.id_at_index(index)?;
        self.delete_by_id(&id)
    }

    /// Delete an element by its ID.
    pub fn delete_by_id(&mut self, id: &ListId) -> Option<T> {
        if let Some(node) = self.nodes.get_mut(id) {
            if !node.deleted {
                node.deleted = true;
                let value = node.value.take();

                // Record delta
                let delta = self.pending_delta.get_or_insert_with(RGAListDelta::new);
                delta.deletes.push(id.clone());

                return value;
            }
        }
        None
    }

    /// Move an element from one index to another.
    pub fn move_element(&mut self, from: usize, to: usize) -> bool {
        if let Some(value) = self.delete(from) {
            // Adjust target index if moving forward
            let adjusted_to = if to > from { to - 1 } else { to };
            self.insert(adjusted_to, value);
            true
        } else {
            false
        }
    }

    /// Get the element at the given index.
    pub fn get(&self, index: usize) -> Option<&T> {
        let id = self.id_at_index(index)?;
        self.nodes.get(&id).and_then(|n| n.value.as_ref())
    }

    /// Get a mutable reference to the element at the given index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let id = self.id_at_index(index)?;
        self.nodes.get_mut(&id).and_then(|n| n.value.as_mut())
    }

    /// Get the number of non-deleted elements.
    pub fn len(&self) -> usize {
        self.nodes.values().filter(|n| !n.deleted).count()
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over values in order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.iter_nodes()
            .filter(|n| !n.deleted)
            .filter_map(|n| n.value.as_ref())
    }

    /// Iterate over (index, value) pairs.
    pub fn iter_indexed(&self) -> impl Iterator<Item = (usize, &T)> {
        self.iter().enumerate()
    }

    /// Convert to a Vec.
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    /// Get the ID at a given visible index.
    fn id_at_index(&self, index: usize) -> Option<ListId> {
        self.iter_nodes()
            .filter(|n| !n.deleted)
            .nth(index)
            .map(|n| n.id.clone())
    }

    /// Get the visible index for an ID.
    pub fn index_of_id(&self, id: &ListId) -> Option<usize> {
        self.iter_nodes()
            .filter(|n| !n.deleted)
            .position(|n| &n.id == id)
    }

    /// Iterate over all nodes in order (including tombstones).
    fn iter_nodes(&self) -> impl Iterator<Item = &ListNode<T>> {
        RGAIterator {
            list: self,
            stack: vec![ListId::genesis()],
            visited: std::collections::HashSet::new(),
        }
    }

    /// Integrate a node into the list.
    fn integrate_node(&mut self, node: ListNode<T>) {
        let id = node.id.clone();
        let origin = node.origin.clone();

        // Add to nodes map
        self.nodes.insert(id.clone(), node);

        // Add to children of origin, maintaining sort order
        let children = self.children.entry(origin).or_default();

        // Find insertion position (maintain descending order by ID for RGA)
        let pos = children
            .iter()
            .position(|c| c < &id)
            .unwrap_or(children.len());
        children.insert(pos, id.clone());

        // Ensure this node has a children entry
        self.children.entry(id).or_default();
    }

    /// Take the pending delta.
    pub fn take_delta(&mut self) -> Option<RGAListDelta<T>> {
        self.pending_delta.take()
    }

    /// Apply a delta from another replica.
    pub fn apply_delta(&mut self, delta: &RGAListDelta<T>) {
        // Apply inserts
        for node in &delta.inserts {
            if !self.nodes.contains_key(&node.id) {
                self.integrate_node(node.clone());
            }
        }

        // Apply deletes
        for id in &delta.deletes {
            if let Some(node) = self.nodes.get_mut(id) {
                node.deleted = true;
                node.value = None;
            }
        }
    }
}

/// Iterator for traversing the RGA list in order.
struct RGAIterator<'a, T: Clone + PartialEq> {
    list: &'a RGAList<T>,
    stack: Vec<ListId>,
    visited: std::collections::HashSet<ListId>,
}

impl<'a, T: Clone + PartialEq> Iterator for RGAIterator<'a, T> {
    type Item = &'a ListNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(id) = self.stack.pop() {
            if self.visited.contains(&id) {
                continue;
            }
            self.visited.insert(id.clone());

            // Push children in reverse order (so first child is processed first)
            if let Some(children) = self.list.children.get(&id) {
                for child in children.iter().rev() {
                    if !self.visited.contains(child) {
                        self.stack.push(child.clone());
                    }
                }
            }

            // Return the node (skip genesis)
            if id != ListId::genesis() {
                if let Some(node) = self.list.nodes.get(&id) {
                    return Some(node);
                }
            }
        }
        None
    }
}

impl<T: Clone + PartialEq> PartialEq for RGAList<T> {
    fn eq(&self, other: &Self) -> bool {
        // Compare visible content
        self.to_vec() == other.to_vec()
    }
}

impl<T: Clone + PartialEq> Lattice for RGAList<T> {
    fn bottom() -> Self {
        Self::new("")
    }

    fn join(&self, other: &Self) -> Self {
        let mut result = self.clone();

        // Merge all nodes from other
        for (id, node) in &other.nodes {
            if let Some(existing) = result.nodes.get_mut(id) {
                // If deleted in either, mark as deleted
                if node.deleted {
                    existing.deleted = true;
                    existing.value = None;
                }
            } else {
                // Add new node
                result.integrate_node(node.clone());
            }
        }

        result
    }
}

impl<T: Clone + PartialEq> Default for RGAList<T> {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut list: RGAList<String> = RGAList::new("r1");

        list.push_back("a".to_string());
        list.push_back("b".to_string());
        list.push_back("c".to_string());

        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(&"a".to_string()));
        assert_eq!(list.get(1), Some(&"b".to_string()));
        assert_eq!(list.get(2), Some(&"c".to_string()));
    }

    #[test]
    fn test_insert_at_index() {
        let mut list: RGAList<i32> = RGAList::new("r1");

        list.push_back(1);
        list.push_back(3);
        list.insert(1, 2);

        assert_eq!(list.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_delete() {
        let mut list: RGAList<i32> = RGAList::new("r1");

        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let deleted = list.delete(1);
        assert_eq!(deleted, Some(2));
        assert_eq!(list.to_vec(), vec![1, 3]);
    }

    #[test]
    fn test_concurrent_inserts() {
        let mut list1: RGAList<&str> = RGAList::new("r1");
        let mut list2: RGAList<&str> = RGAList::new("r2");

        // Both start with "a"
        list1.push_back("a");
        list2.apply_delta(&list1.take_delta().unwrap());

        // Concurrent inserts after "a"
        list1.push_back("b"); // r1 inserts "b"
        list2.push_back("c"); // r2 inserts "c"

        // Exchange deltas
        let delta1 = list1.take_delta().unwrap();
        let delta2 = list2.take_delta().unwrap();

        list1.apply_delta(&delta2);
        list2.apply_delta(&delta1);

        // Should converge to same order
        assert_eq!(list1.to_vec(), list2.to_vec());
    }

    #[test]
    fn test_move_element() {
        let mut list: RGAList<i32> = RGAList::new("r1");

        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        list.move_element(0, 2);
        assert_eq!(list.to_vec(), vec![2, 1, 3]);
    }

    #[test]
    fn test_lattice_join() {
        let mut list1: RGAList<i32> = RGAList::new("r1");
        let mut list2: RGAList<i32> = RGAList::new("r2");

        list1.push_back(1);
        list2.push_back(2);

        let merged = list1.join(&list2);

        // Both elements should be present
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_iter() {
        let mut list: RGAList<i32> = RGAList::new("r1");

        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let collected: Vec<_> = list.iter().cloned().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }
}
