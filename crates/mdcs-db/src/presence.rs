//! Presence System - Real-time user presence and cursor tracking.
//!
//! Provides collaborative awareness features:
//! - Cursor positions and selections
//! - User online/offline status
//! - Custom user state (e.g., "typing", "away")
//! - Automatic expiration of stale presence

use mdcs_core::lattice::Lattice;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a user.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A cursor position in a document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursor {
    /// The position (character offset) in the document.
    pub position: usize,
    /// Optional anchor for selection (selection goes from anchor to position).
    pub anchor: Option<usize>,
}

impl Cursor {
    /// Create a cursor at a position (no selection).
    pub fn at(position: usize) -> Self {
        Self {
            position,
            anchor: None,
        }
    }

    /// Create a cursor with a selection.
    pub fn with_selection(anchor: usize, position: usize) -> Self {
        Self {
            position,
            anchor: Some(anchor),
        }
    }

    /// Check if this cursor has a selection.
    pub fn has_selection(&self) -> bool {
        self.anchor.is_some() && self.anchor != Some(self.position)
    }

    /// Get the selection range (start, end).
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.anchor.map(|anchor| {
            if anchor < self.position {
                (anchor, self.position)
            } else {
                (self.position, anchor)
            }
        })
    }

    /// Get the selection length.
    pub fn selection_length(&self) -> usize {
        self.selection_range()
            .map(|(start, end)| end - start)
            .unwrap_or(0)
    }
}

/// User status.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    /// User is online and active.
    #[default]
    Online,
    /// User is online but idle.
    Idle,
    /// User is actively typing.
    Typing,
    /// User is away.
    Away,
    /// User is offline.
    Offline,
    /// Custom status.
    Custom(String),
}

/// User information for display.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserInfo {
    /// User's display name.
    pub name: String,
    /// User's color (for cursor highlighting).
    pub color: String,
    /// Optional avatar URL.
    pub avatar: Option<String>,
}

impl UserInfo {
    pub fn new(name: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
            avatar: None,
        }
    }

    pub fn with_avatar(mut self, avatar: impl Into<String>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }
}

/// Presence data for a single user.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserPresence {
    /// The user ID.
    pub user_id: UserId,
    /// User information.
    pub info: UserInfo,
    /// Current status.
    pub status: UserStatus,
    /// Cursor positions by document ID.
    pub cursors: HashMap<String, Cursor>,
    /// Custom user state data.
    pub state: HashMap<String, String>,
    /// Last update timestamp (milliseconds since epoch).
    pub last_updated: u64,
    /// Lamport timestamp for ordering.
    pub timestamp: u64,
}

impl UserPresence {
    /// Create new presence for a user.
    pub fn new(user_id: UserId, info: UserInfo) -> Self {
        Self {
            user_id,
            info,
            status: UserStatus::Online,
            cursors: HashMap::new(),
            state: HashMap::new(),
            last_updated: now_millis(),
            timestamp: 0,
        }
    }

    /// Update the cursor for a document.
    pub fn set_cursor(&mut self, document_id: impl Into<String>, cursor: Cursor) {
        self.cursors.insert(document_id.into(), cursor);
        self.touch();
    }

    /// Remove the cursor for a document.
    pub fn remove_cursor(&mut self, document_id: &str) {
        self.cursors.remove(document_id);
        self.touch();
    }

    /// Get the cursor for a document.
    pub fn get_cursor(&self, document_id: &str) -> Option<&Cursor> {
        self.cursors.get(document_id)
    }

    /// Set the status.
    pub fn set_status(&mut self, status: UserStatus) {
        self.status = status;
        self.touch();
    }

    /// Set custom state data.
    pub fn set_state(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.state.insert(key.into(), value.into());
        self.touch();
    }

    /// Get custom state data.
    pub fn get_state(&self, key: &str) -> Option<&String> {
        self.state.get(key)
    }

    /// Touch the update timestamp.
    fn touch(&mut self) {
        self.last_updated = now_millis();
        self.timestamp += 1;
    }

    /// Check if this presence is stale (not updated within timeout).
    pub fn is_stale(&self, timeout_ms: u64) -> bool {
        let now = now_millis();
        now.saturating_sub(self.last_updated) > timeout_ms
    }
}

/// Delta for presence updates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresenceDelta {
    /// Updated presence records.
    pub updates: Vec<UserPresence>,
    /// Users that have left.
    pub removals: Vec<UserId>,
}

impl PresenceDelta {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
            removals: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.updates.is_empty() && self.removals.is_empty()
    }
}

impl Default for PresenceDelta {
    fn default() -> Self {
        Self::new()
    }
}

/// Presence tracker for a collaborative session.
///
/// Tracks all users' cursors, selections, and status.
#[derive(Clone, Debug, PartialEq)]
pub struct PresenceTracker {
    /// The local user's ID.
    local_user: UserId,
    /// All user presence records.
    users: HashMap<UserId, UserPresence>,
    /// Timeout for stale presence (milliseconds).
    stale_timeout: u64,
    /// Pending delta for replication.
    pending_delta: Option<PresenceDelta>,
}

impl PresenceTracker {
    /// Create a new presence tracker.
    pub fn new(local_user: UserId, info: UserInfo) -> Self {
        let mut tracker = Self {
            local_user: local_user.clone(),
            users: HashMap::new(),
            stale_timeout: 30_000, // 30 seconds default
            pending_delta: None,
        };

        // Add local user
        let presence = UserPresence::new(local_user, info);
        tracker.users.insert(presence.user_id.clone(), presence);

        tracker
    }

    /// Get the local user ID.
    pub fn local_user(&self) -> &UserId {
        &self.local_user
    }

    /// Set the stale timeout.
    pub fn set_stale_timeout(&mut self, timeout_ms: u64) {
        self.stale_timeout = timeout_ms;
    }

    /// Get the local user's presence.
    pub fn local_presence(&self) -> Option<&UserPresence> {
        self.users.get(&self.local_user)
    }

    // === Local User Operations ===

    /// Update the local user's cursor.
    pub fn set_cursor(&mut self, document_id: impl Into<String>, cursor: Cursor) {
        let doc_id = document_id.into();
        let local_user = self.local_user.clone();
        if let Some(presence) = self.users.get_mut(&local_user) {
            presence.set_cursor(&doc_id, cursor);
            let presence_clone = presence.clone();
            let delta = self.pending_delta.get_or_insert_with(PresenceDelta::new);
            delta.updates.push(presence_clone);
        }
    }

    /// Remove the local user's cursor from a document.
    pub fn remove_cursor(&mut self, document_id: &str) {
        let local_user = self.local_user.clone();
        if let Some(presence) = self.users.get_mut(&local_user) {
            presence.remove_cursor(document_id);
            let presence_clone = presence.clone();
            let delta = self.pending_delta.get_or_insert_with(PresenceDelta::new);
            delta.updates.push(presence_clone);
        }
    }

    /// Set the local user's status.
    pub fn set_status(&mut self, status: UserStatus) {
        let local_user = self.local_user.clone();
        if let Some(presence) = self.users.get_mut(&local_user) {
            presence.set_status(status);
            let presence_clone = presence.clone();
            let delta = self.pending_delta.get_or_insert_with(PresenceDelta::new);
            delta.updates.push(presence_clone);
        }
    }

    /// Set local user's custom state.
    pub fn set_state(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let local_user = self.local_user.clone();
        if let Some(presence) = self.users.get_mut(&local_user) {
            presence.set_state(key, value);
            let presence_clone = presence.clone();
            let delta = self.pending_delta.get_or_insert_with(PresenceDelta::new);
            delta.updates.push(presence_clone);
        }
    }

    /// Send a heartbeat to keep presence alive.
    pub fn heartbeat(&mut self) {
        let local_user = self.local_user.clone();
        if let Some(presence) = self.users.get_mut(&local_user) {
            presence.touch();
            let presence_clone = presence.clone();
            let delta = self.pending_delta.get_or_insert_with(PresenceDelta::new);
            delta.updates.push(presence_clone);
        }
    }

    // === Query Operations ===

    /// Get a user's presence.
    pub fn get_user(&self, user_id: &UserId) -> Option<&UserPresence> {
        self.users.get(user_id)
    }

    /// Get all users.
    pub fn all_users(&self) -> impl Iterator<Item = &UserPresence> + '_ {
        self.users.values()
    }

    /// Get all online users.
    pub fn online_users(&self) -> impl Iterator<Item = &UserPresence> + '_ {
        self.users
            .values()
            .filter(|p| !p.is_stale(self.stale_timeout) && !matches!(p.status, UserStatus::Offline))
    }

    /// Get users with cursors in a document.
    pub fn users_in_document(&self, document_id: &str) -> Vec<&UserPresence> {
        self.online_users()
            .filter(|p| p.cursors.contains_key(document_id))
            .collect()
    }

    /// Get all cursors in a document (excluding local user).
    pub fn cursors_in_document(&self, document_id: &str) -> Vec<(&UserPresence, &Cursor)> {
        self.online_users()
            .filter(|p| p.user_id != self.local_user)
            .filter_map(|p| p.get_cursor(document_id).map(|c| (p, c)))
            .collect()
    }

    /// Count online users.
    pub fn online_count(&self) -> usize {
        self.online_users().count()
    }

    // === Sync Operations ===

    /// Take the pending delta.
    pub fn take_delta(&mut self) -> Option<PresenceDelta> {
        self.pending_delta.take()
    }

    /// Apply a delta from another replica.
    pub fn apply_delta(&mut self, delta: &PresenceDelta) {
        // Apply updates
        for presence in &delta.updates {
            // Don't overwrite with older data
            if let Some(existing) = self.users.get(&presence.user_id) {
                if presence.timestamp <= existing.timestamp {
                    continue;
                }
            }
            self.users
                .insert(presence.user_id.clone(), presence.clone());
        }

        // Apply removals
        for user_id in &delta.removals {
            if *user_id != self.local_user {
                self.users.remove(user_id);
            }
        }
    }

    /// Clean up stale presence records.
    pub fn cleanup_stale(&mut self) -> Vec<UserId> {
        let stale: Vec<_> = self
            .users
            .iter()
            .filter(|(id, p)| *id != &self.local_user && p.is_stale(self.stale_timeout))
            .map(|(id, _)| id.clone())
            .collect();

        for id in &stale {
            self.users.remove(id);
        }

        if !stale.is_empty() {
            let delta = self.pending_delta.get_or_insert_with(PresenceDelta::new);
            delta.removals.extend(stale.clone());
        }

        stale
    }

    /// Leave (remove local user).
    pub fn leave(&mut self) {
        let delta = self.pending_delta.get_or_insert_with(PresenceDelta::new);
        delta.removals.push(self.local_user.clone());
    }
}

impl Lattice for PresenceTracker {
    fn bottom() -> Self {
        Self {
            local_user: UserId::new(""),
            users: HashMap::new(),
            stale_timeout: 30_000,
            pending_delta: None,
        }
    }

    fn join(&self, other: &Self) -> Self {
        let mut result = self.clone();

        for (user_id, other_presence) in &other.users {
            result
                .users
                .entry(user_id.clone())
                .and_modify(|p| {
                    if other_presence.timestamp > p.timestamp {
                        *p = other_presence.clone();
                    }
                })
                .or_insert_with(|| other_presence.clone());
        }

        result
    }
}

/// Get current time in milliseconds.
fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Builder for creating cursors from selections.
pub struct CursorBuilder {
    document_id: String,
}

impl CursorBuilder {
    pub fn for_document(id: impl Into<String>) -> Self {
        Self {
            document_id: id.into(),
        }
    }

    pub fn at(self, position: usize) -> (String, Cursor) {
        (self.document_id, Cursor::at(position))
    }

    pub fn selection(self, anchor: usize, head: usize) -> (String, Cursor) {
        (self.document_id, Cursor::with_selection(anchor, head))
    }
}

/// Color palette for user cursors.
pub struct CursorColors;

impl CursorColors {
    pub const COLORS: [&'static str; 12] = [
        "#E91E63", // Pink
        "#9C27B0", // Purple
        "#3F51B5", // Indigo
        "#2196F3", // Blue
        "#00BCD4", // Cyan
        "#009688", // Teal
        "#4CAF50", // Green
        "#8BC34A", // Light Green
        "#CDDC39", // Lime
        "#FF9800", // Orange
        "#FF5722", // Deep Orange
        "#795548", // Brown
    ];

    /// Get a color for a user based on their ID.
    pub fn color_for_user(user_id: &UserId) -> &'static str {
        let hash: usize = user_id.0.bytes().map(|b| b as usize).sum();
        Self::COLORS[hash % Self::COLORS.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_creation() {
        let cursor = Cursor::at(10);
        assert_eq!(cursor.position, 10);
        assert!(!cursor.has_selection());

        let selection = Cursor::with_selection(5, 15);
        assert!(selection.has_selection());
        assert_eq!(selection.selection_range(), Some((5, 15)));
        assert_eq!(selection.selection_length(), 10);
    }

    #[test]
    fn test_cursor_selection_backwards() {
        let selection = Cursor::with_selection(15, 5);
        assert_eq!(selection.selection_range(), Some((5, 15)));
        assert_eq!(selection.selection_length(), 10);
    }

    #[test]
    fn test_presence_tracker() {
        let user_id = UserId::new("user1");
        let info = UserInfo::new("Alice", "#E91E63");
        let tracker = PresenceTracker::new(user_id.clone(), info);

        assert_eq!(tracker.local_user(), &user_id);
        assert!(tracker.local_presence().is_some());
    }

    #[test]
    fn test_cursor_tracking() {
        let user_id = UserId::new("user1");
        let info = UserInfo::new("Alice", "#E91E63");
        let mut tracker = PresenceTracker::new(user_id, info);

        tracker.set_cursor("doc1", Cursor::at(42));

        let presence = tracker.local_presence().unwrap();
        let cursor = presence.get_cursor("doc1").unwrap();
        assert_eq!(cursor.position, 42);
    }

    #[test]
    fn test_status_changes() {
        let user_id = UserId::new("user1");
        let info = UserInfo::new("Alice", "#E91E63");
        let mut tracker = PresenceTracker::new(user_id, info);

        tracker.set_status(UserStatus::Typing);

        let presence = tracker.local_presence().unwrap();
        assert_eq!(presence.status, UserStatus::Typing);
    }

    #[test]
    fn test_presence_sync() {
        let user1 = UserId::new("user1");
        let user2 = UserId::new("user2");

        let mut tracker1 = PresenceTracker::new(user1.clone(), UserInfo::new("Alice", "#E91E63"));
        let mut tracker2 = PresenceTracker::new(user2.clone(), UserInfo::new("Bob", "#2196F3"));

        // User 1 sets cursor
        tracker1.set_cursor("doc1", Cursor::at(10));

        // Sync to user 2
        let delta = tracker1.take_delta().unwrap();
        tracker2.apply_delta(&delta);

        // User 2 should see user 1's cursor
        let users = tracker2.users_in_document("doc1");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].user_id, user1);
    }

    #[test]
    fn test_multiple_users() {
        let user1 = UserId::new("user1");
        let info1 = UserInfo::new("Alice", "#E91E63");
        let mut tracker = PresenceTracker::new(user1.clone(), info1);

        // Simulate other users joining
        let user2 = UserId::new("user2");
        let presence2 = UserPresence::new(user2.clone(), UserInfo::new("Bob", "#2196F3"));
        tracker.users.insert(user2.clone(), presence2);

        let user3 = UserId::new("user3");
        let presence3 = UserPresence::new(user3.clone(), UserInfo::new("Charlie", "#4CAF50"));
        tracker.users.insert(user3.clone(), presence3);

        assert_eq!(tracker.online_count(), 3);
    }

    #[test]
    fn test_cursors_in_document() {
        let user1 = UserId::new("user1");
        let info1 = UserInfo::new("Alice", "#E91E63");
        let mut tracker = PresenceTracker::new(user1, info1);

        // Add another user with cursor
        let user2 = UserId::new("user2");
        let mut presence2 = UserPresence::new(user2.clone(), UserInfo::new("Bob", "#2196F3"));
        presence2.set_cursor("doc1", Cursor::at(50));
        tracker.users.insert(user2, presence2);

        // Get cursors (excluding local user)
        let cursors = tracker.cursors_in_document("doc1");
        assert_eq!(cursors.len(), 1);
        assert_eq!(cursors[0].1.position, 50);
    }

    #[test]
    fn test_color_assignment() {
        let user1 = UserId::new("alice");
        let user2 = UserId::new("bob");

        let color1 = CursorColors::color_for_user(&user1);
        let color2 = CursorColors::color_for_user(&user2);

        // Colors should be from the palette
        assert!(CursorColors::COLORS.contains(&color1));
        assert!(CursorColors::COLORS.contains(&color2));

        // Same user should get same color
        assert_eq!(color1, CursorColors::color_for_user(&user1));
    }

    #[test]
    fn test_custom_state() {
        let user_id = UserId::new("user1");
        let info = UserInfo::new("Alice", "#E91E63");
        let mut tracker = PresenceTracker::new(user_id, info);

        tracker.set_state("view", "editor");
        tracker.set_state("zoom", "100%");

        let presence = tracker.local_presence().unwrap();
        assert_eq!(presence.get_state("view"), Some(&"editor".to_string()));
        assert_eq!(presence.get_state("zoom"), Some(&"100%".to_string()));
    }

    #[test]
    fn test_cursor_builder() {
        let (doc, cursor) = CursorBuilder::for_document("doc1").at(42);
        assert_eq!(doc, "doc1");
        assert_eq!(cursor.position, 42);

        let (doc, cursor) = CursorBuilder::for_document("doc2").selection(10, 20);
        assert_eq!(doc, "doc2");
        assert_eq!(cursor.selection_range(), Some((10, 20)));
    }
}
