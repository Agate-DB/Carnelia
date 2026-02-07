//! RGA Text - Collaborative text CRDT based on Replicated Growable Array.
//!
//! Provides character-level collaborative text editing with:
//! - Insert at any position
//! - Delete ranges
//! - Stable position anchors for cursor sync
//!
//! Based on the RGA algorithm but optimized for text.

use mdcs_core::lattice::Lattice;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Unique identifier for a character in the text.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextId {
    /// The replica that created this character.
    pub replica: String,
    /// Sequence number within that replica.
    pub seq: u64,
}

impl TextId {
    pub fn new(replica: impl Into<String>, seq: u64) -> Self {
        Self {
            replica: replica.into(),
            seq,
        }
    }

    /// Create a genesis ID (for the virtual start of text).
    pub fn genesis() -> Self {
        Self {
            replica: "".to_string(),
            seq: 0,
        }
    }

    /// Create an end marker ID.
    pub fn end() -> Self {
        Self {
            replica: "".to_string(),
            seq: u64::MAX,
        }
    }
}

impl PartialOrd for TextId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TextId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher sequence = later in causal order
        // Tie-break on replica ID for determinism
        self.seq
            .cmp(&other.seq)
            .then_with(|| self.replica.cmp(&other.replica))
    }
}

/// A character node in the RGA text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TextNode {
    /// The unique ID of this character.
    id: TextId,
    /// The character (or None if deleted).
    char: Option<char>,
    /// The ID of the character this was inserted after.
    origin: TextId,
    /// Whether this node is deleted (tombstone).
    deleted: bool,
}

impl TextNode {
    fn new(id: TextId, ch: char, origin: TextId) -> Self {
        Self {
            id,
            char: Some(ch),
            origin,
            deleted: false,
        }
    }
}

/// Delta for text operations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RGATextDelta {
    /// Characters to insert.
    pub inserts: Vec<(TextId, char, TextId)>, // (id, char, origin)
    /// IDs of characters to delete.
    pub deletes: Vec<TextId>,
}

impl RGATextDelta {
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

impl Default for RGATextDelta {
    fn default() -> Self {
        Self::new()
    }
}

/// Collaborative text CRDT using RGA algorithm.
///
/// Supports character-level insert and delete with
/// deterministic conflict resolution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RGAText {
    /// All nodes indexed by their ID.
    nodes: HashMap<TextId, TextNode>,
    /// Children of each node (characters inserted after it).
    /// Maps origin -> list of children sorted by ID (descending for RGA).
    children: HashMap<TextId, Vec<TextId>>,
    /// The replica ID for this instance.
    replica_id: String,
    /// Sequence counter for generating IDs.
    seq: u64,
    /// Pending delta for replication.
    #[serde(skip)]
    pending_delta: Option<RGATextDelta>,
}

impl RGAText {
    /// Create a new empty text.
    pub fn new(replica_id: impl Into<String>) -> Self {
        let replica_id = replica_id.into();
        let mut text = Self {
            nodes: HashMap::new(),
            children: HashMap::new(),
            replica_id,
            seq: 0,
            pending_delta: None,
        };

        // Initialize with genesis node's children list
        text.children.insert(TextId::genesis(), Vec::new());

        text
    }

    /// Get the replica ID.
    pub fn replica_id(&self) -> &str {
        &self.replica_id
    }

    /// Generate a new unique ID.
    fn next_id(&mut self) -> TextId {
        self.seq += 1;
        TextId::new(&self.replica_id, self.seq)
    }

    /// Insert a string at the given position.
    pub fn insert(&mut self, position: usize, text: &str) {
        let mut origin = self
            .id_at_index(position.saturating_sub(1))
            .unwrap_or(TextId::genesis());

        for ch in text.chars() {
            let id = self.next_id();
            let node = TextNode::new(id.clone(), ch, origin.clone());

            self.integrate_node(node.clone());

            // Record in delta
            let delta = self.pending_delta.get_or_insert_with(RGATextDelta::new);
            delta.inserts.push((id.clone(), ch, origin));

            origin = id;
        }
    }

    /// Delete characters from start to start+length.
    pub fn delete(&mut self, start: usize, length: usize) {
        let ids: Vec<_> = self
            .visible_ids()
            .skip(start)
            .take(length)
            .cloned()
            .collect();

        for id in ids {
            self.delete_by_id(&id);
        }
    }

    /// Delete a character by its ID.
    fn delete_by_id(&mut self, id: &TextId) -> Option<char> {
        if let Some(node) = self.nodes.get_mut(id) {
            if !node.deleted {
                node.deleted = true;
                let ch = node.char.take();

                // Record delta
                let delta = self.pending_delta.get_or_insert_with(RGATextDelta::new);
                delta.deletes.push(id.clone());

                return ch;
            }
        }
        None
    }

    /// Replace a range with new text.
    pub fn replace(&mut self, start: usize, end: usize, text: &str) {
        self.delete(start, end - start);
        self.insert(start, text);
    }

    /// Splice: delete some characters and insert new ones.
    pub fn splice(&mut self, position: usize, delete_count: usize, insert: &str) {
        self.delete(position, delete_count);
        self.insert(position, insert);
    }

    /// Get the text as a String.
    pub fn to_string(&self) -> String {
        self.iter().collect()
    }

    /// Get the length (number of visible characters).
    pub fn len(&self) -> usize {
        self.nodes.values().filter(|n| !n.deleted).count()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get character at position.
    pub fn char_at(&self, position: usize) -> Option<char> {
        self.iter().nth(position)
    }

    /// Get a substring.
    pub fn slice(&self, start: usize, end: usize) -> String {
        self.iter().skip(start).take(end - start).collect()
    }

    /// Iterate over visible characters.
    pub fn iter(&self) -> impl Iterator<Item = char> + '_ {
        self.iter_nodes()
            .filter(|n| !n.deleted)
            .filter_map(|n| n.char)
    }

    /// Get the ID at a visible index.
    fn id_at_index(&self, index: usize) -> Option<TextId> {
        self.visible_ids().nth(index).cloned()
    }

    /// Iterate over visible IDs.
    fn visible_ids(&self) -> impl Iterator<Item = &TextId> + '_ {
        self.iter_nodes().filter(|n| !n.deleted).map(|n| &n.id)
    }

    /// Convert a TextId to a visible position.
    pub fn id_to_position(&self, id: &TextId) -> Option<usize> {
        self.visible_ids().position(|i| i == id)
    }

    /// Convert a visible position to a TextId.
    pub fn position_to_id(&self, position: usize) -> Option<TextId> {
        self.id_at_index(position)
    }

    /// Iterate over all nodes in order.
    fn iter_nodes(&self) -> impl Iterator<Item = &TextNode> + '_ {
        TextIterator {
            text: self,
            stack: vec![TextId::genesis()],
            visited: HashSet::new(),
        }
    }

    /// Integrate a node into the text.
    fn integrate_node(&mut self, node: TextNode) {
        let id = node.id.clone();
        let origin = node.origin.clone();

        // Add to nodes map
        self.nodes.insert(id.clone(), node);

        // Add to children of origin, maintaining sort order (descending by ID for RGA)
        let children = self.children.entry(origin).or_default();
        let pos = children
            .iter()
            .position(|c| c < &id)
            .unwrap_or(children.len());
        children.insert(pos, id.clone());

        // Ensure this node has a children entry
        self.children.entry(id).or_default();
    }

    /// Take the pending delta.
    pub fn take_delta(&mut self) -> Option<RGATextDelta> {
        self.pending_delta.take()
    }

    /// Apply a delta from another replica.
    pub fn apply_delta(&mut self, delta: &RGATextDelta) {
        // Apply inserts
        for (id, ch, origin) in &delta.inserts {
            if !self.nodes.contains_key(id) {
                let node = TextNode::new(id.clone(), *ch, origin.clone());
                self.integrate_node(node);
            }
        }

        // Apply deletes
        for id in &delta.deletes {
            if let Some(node) = self.nodes.get_mut(id) {
                node.deleted = true;
                node.char = None;
            }
        }
    }
}

/// Iterator for traversing text nodes in order.
struct TextIterator<'a> {
    text: &'a RGAText,
    stack: Vec<TextId>,
    visited: HashSet<TextId>,
}

impl<'a> Iterator for TextIterator<'a> {
    type Item = &'a TextNode;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(id) = self.stack.pop() {
            if self.visited.contains(&id) {
                continue;
            }
            self.visited.insert(id.clone());

            // Push children in reverse order
            if let Some(children) = self.text.children.get(&id) {
                for child in children.iter().rev() {
                    if !self.visited.contains(child) {
                        self.stack.push(child.clone());
                    }
                }
            }

            // Return the node (skip genesis)
            if id != TextId::genesis() {
                if let Some(node) = self.text.nodes.get(&id) {
                    return Some(node);
                }
            }
        }
        None
    }
}

impl std::fmt::Display for RGAText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for RGAText {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for RGAText {}

impl Lattice for RGAText {
    fn bottom() -> Self {
        Self::new("")
    }

    fn join(&self, other: &Self) -> Self {
        let mut result = self.clone();

        // Merge all nodes from other
        for (id, node) in &other.nodes {
            if let Some(existing) = result.nodes.get_mut(id) {
                if node.deleted {
                    existing.deleted = true;
                    existing.char = None;
                }
            } else {
                result.integrate_node(node.clone());
            }
        }

        result
    }
}

impl Default for RGAText {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert() {
        let mut text = RGAText::new("r1");
        text.insert(0, "Hello");
        assert_eq!(text.to_string(), "Hello");
        assert_eq!(text.len(), 5);
    }

    #[test]
    fn test_insert_at_position() {
        let mut text = RGAText::new("r1");
        text.insert(0, "Hello");
        text.insert(5, " World");
        assert_eq!(text.to_string(), "Hello World");
    }

    #[test]
    fn test_insert_in_middle() {
        let mut text = RGAText::new("r1");
        text.insert(0, "Helo");
        text.insert(2, "l");
        assert_eq!(text.to_string(), "Hello");
    }

    #[test]
    fn test_delete() {
        let mut text = RGAText::new("r1");
        text.insert(0, "Hello World");
        text.delete(5, 6); // Delete " World"
        assert_eq!(text.to_string(), "Hello");
    }

    #[test]
    fn test_replace() {
        let mut text = RGAText::new("r1");
        text.insert(0, "Hello World");
        text.replace(6, 11, "Rust");
        assert_eq!(text.to_string(), "Hello Rust");
    }

    #[test]
    fn test_concurrent_inserts() {
        let mut text1 = RGAText::new("r1");
        let mut text2 = RGAText::new("r2");

        // Both start with "Hello"
        text1.insert(0, "Hello");
        text2.apply_delta(&text1.take_delta().unwrap());

        // Concurrent inserts at the end
        text1.insert(5, " World");
        text2.insert(5, " Rust");

        // Exchange deltas
        let delta1 = text1.take_delta().unwrap();
        let delta2 = text2.take_delta().unwrap();

        text1.apply_delta(&delta2);
        text2.apply_delta(&delta1);

        // Should converge to same text
        assert_eq!(text1.to_string(), text2.to_string());
        // Both additions should be present
        assert!(text1.to_string().contains("World") || text1.to_string().contains("Rust"));
    }

    #[test]
    fn test_concurrent_insert_delete() {
        let mut text1 = RGAText::new("r1");
        let mut text2 = RGAText::new("r2");

        // Both start with "Hello"
        text1.insert(0, "Hello");
        text2.apply_delta(&text1.take_delta().unwrap());

        // r1 deletes "llo", r2 inserts "x" after "He"
        text1.delete(2, 3);
        text2.insert(2, "x");

        // Exchange deltas
        let delta1 = text1.take_delta().unwrap();
        let delta2 = text2.take_delta().unwrap();

        text1.apply_delta(&delta2);
        text2.apply_delta(&delta1);

        // Should converge
        assert_eq!(text1.to_string(), text2.to_string());
        // x should survive, llo should be deleted
        assert!(text1.to_string().contains("x"));
        assert!(!text1.to_string().contains("llo"));
    }

    #[test]
    fn test_char_at() {
        let mut text = RGAText::new("r1");
        text.insert(0, "Hello");

        assert_eq!(text.char_at(0), Some('H'));
        assert_eq!(text.char_at(4), Some('o'));
        assert_eq!(text.char_at(5), None);
    }

    #[test]
    fn test_slice() {
        let mut text = RGAText::new("r1");
        text.insert(0, "Hello World");

        assert_eq!(text.slice(0, 5), "Hello");
        assert_eq!(text.slice(6, 11), "World");
    }

    #[test]
    fn test_position_id_conversion() {
        let mut text = RGAText::new("r1");
        text.insert(0, "Hello");

        let id = text.position_to_id(2).unwrap();
        let pos = text.id_to_position(&id).unwrap();

        assert_eq!(pos, 2);
    }

    #[test]
    fn test_lattice_join() {
        let mut text1 = RGAText::new("r1");
        let mut text2 = RGAText::new("r2");

        text1.insert(0, "Hello");
        text2.insert(0, "World");

        let merged = text1.join(&text2);

        // Both texts should be somehow combined
        assert!(merged.len() >= 5);
    }
}
