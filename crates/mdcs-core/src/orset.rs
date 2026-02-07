//!  Observed-Remove Set (OR-Set / Add-Wins Set)
//!
//! Each add generates a unique tag.  Remove only removes currently observed tags.
//!  Concurrent add and remove of the same element:  add wins.

use crate::lattice::{DeltaCRDT, Lattice};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use ulid::Ulid;

/// A unique tag for each add operation
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tag {
    /// The replica that created this tag
    pub replica_id: String,
    /// Unique identifier for this specific add
    pub unique_id: Ulid,
}

impl Tag {
    pub fn new(replica_id: impl Into<String>) -> Self {
        Self {
            replica_id: replica_id.into(),
            unique_id: Ulid::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ORSet<T: Ord + Clone> {
    /// Maps elements to their active tags
    entries: BTreeMap<T, BTreeSet<Tag>>,
    /// Tombstones:  tags that have been removed
    /// (Required for distributed consistency)
    tombstones: BTreeSet<Tag>,
    /// Pending delta for delta-state replication
    #[serde(skip)]
    pending_delta: Option<ORSetDelta<T>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ORSetDelta<T: Ord + Clone> {
    pub additions: BTreeMap<T, BTreeSet<Tag>>,
    pub removals: BTreeSet<Tag>,
}

impl<T: Ord + Clone> ORSet<T> {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            tombstones: BTreeSet::new(),
            pending_delta: None,
        }
    }

    /// Add an element with a new unique tag
    pub fn add(&mut self, replica_id: &str, value: T) {
        let tag = Tag::new(replica_id);

        self.entries
            .entry(value.clone())
            .or_default()
            .insert(tag.clone());

        // Record in delta
        let delta = self.pending_delta.get_or_insert_with(|| ORSetDelta {
            additions: BTreeMap::new(),
            removals: BTreeSet::new(),
        });
        delta.additions.entry(value).or_default().insert(tag);
    }

    /// Remove all observed instances of an element
    pub fn remove(&mut self, value: &T) {
        if let Some(tags) = self.entries.remove(value) {
            // Move tags to tombstones
            for tag in tags.iter() {
                self.tombstones.insert(tag.clone());
            }

            // Record in delta
            let delta = self.pending_delta.get_or_insert_with(|| ORSetDelta {
                additions: BTreeMap::new(),
                removals: BTreeSet::new(),
            });
            delta.removals.extend(tags);
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        self.entries
            .get(value)
            .is_some_and(|tags| !tags.is_empty())
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries.keys()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<T: Ord + Clone> Default for ORSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone> Lattice for ORSet<T> {
    fn bottom() -> Self {
        Self::new()
    }

    fn join(&self, other: &Self) -> Self {
        let mut result = Self::new();

        // Merge tombstones first
        result.tombstones = self.tombstones.union(&other.tombstones).cloned().collect();

        // Merge entries, filtering out tombstoned tags
        let all_keys: BTreeSet<_> = self
            .entries
            .keys()
            .chain(other.entries.keys())
            .cloned()
            .collect();

        for key in all_keys {
            let self_tags = self.entries.get(&key).cloned().unwrap_or_default();
            let other_tags = other.entries.get(&key).cloned().unwrap_or_default();

            let merged_tags: BTreeSet<Tag> = self_tags
                .union(&other_tags)
                .filter(|tag| !result.tombstones.contains(tag))
                .cloned()
                .collect();

            if !merged_tags.is_empty() {
                result.entries.insert(key, merged_tags);
            }
        }

        result
    }
}

impl<T: Ord + Clone> Lattice for ORSetDelta<T> {
    fn bottom() -> Self {
        Self {
            additions: BTreeMap::new(),
            removals: BTreeSet::new(),
        }
    }

    fn join(&self, other: &Self) -> Self {
        let mut additions = self.additions.clone();
        for (k, v) in &other.additions {
            additions.entry(k.clone()).or_default().extend(v.clone());
        }

        Self {
            additions,
            removals: self.removals.union(&other.removals).cloned().collect(),
        }
    }
}

impl<T: Ord + Clone> DeltaCRDT for ORSet<T> {
    type Delta = ORSetDelta<T>;

    fn split_delta(&mut self) -> Option<Self::Delta> {
        self.pending_delta.take()
    }

    fn apply_delta(&mut self, delta: &Self::Delta) {
        // Apply removals to tombstones
        self.tombstones.extend(delta.removals.iter().cloned());

        // Apply additions, filtering tombstones
        for (value, tags) in &delta.additions {
            let entry = self.entries.entry(value.clone()).or_default();
            for tag in tags {
                if !self.tombstones.contains(tag) {
                    entry.insert(tag.clone());
                }
            }
        }

        // Clean up empty entries
        self.entries.retain(|_, tags| !tags.is_empty());
    }
}
