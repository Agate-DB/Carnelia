//! Presence and awareness for collaborative editing.

use mdcs_db::presence::{Cursor, PresenceTracker, UserId, UserInfo, UserStatus};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Cursor information for a user.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CursorInfo {
    pub user_id: String,
    pub user_name: String,
    pub document_id: String,
    pub position: usize,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub color: String,
}

/// User presence information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPresenceInfo {
    pub user_id: String,
    pub name: String,
    pub status: UserStatus,
    pub color: String,
    pub cursors: HashMap<String, CursorInfo>,
}

/// Events for presence changes.
#[derive(Clone, Debug)]
pub enum AwarenessEvent {
    /// A user's presence was updated.
    UserUpdated(UserPresenceInfo),
    /// A user went offline.
    UserOffline(String),
    /// Cursor moved.
    CursorMoved(CursorInfo),
}

/// Awareness manager for a document or session.
pub struct Awareness {
    local_user_id: String,
    local_user_name: String,
    local_color: String,
    tracker: Arc<RwLock<PresenceTracker>>,
    event_tx: broadcast::Sender<AwarenessEvent>,
}

impl Awareness {
    /// Create a new awareness manager.
    pub fn new(local_user_id: impl Into<String>, local_user_name: impl Into<String>) -> Self {
        let local_user_id = local_user_id.into();
        let local_user_name = local_user_name.into();
        let user_id = UserId::new(&local_user_id);
        let info = UserInfo::new(&local_user_name, "#0066cc");

        let (event_tx, _) = broadcast::channel(100);

        Self {
            local_user_id,
            local_user_name,
            local_color: "#0066cc".to_string(),
            tracker: Arc::new(RwLock::new(PresenceTracker::new(user_id, info))),
            event_tx,
        }
    }

    /// Get the local user ID.
    pub fn local_user_id(&self) -> &str {
        &self.local_user_id
    }

    /// Get the local user name.
    pub fn local_user_name(&self) -> &str {
        &self.local_user_name
    }

    /// Set the local user's cursor position.
    pub fn set_cursor(&self, document_id: &str, position: usize) {
        let cursor = Cursor::at(position);
        self.tracker.write().set_cursor(document_id, cursor);

        let cursor_info = CursorInfo {
            user_id: self.local_user_id.clone(),
            user_name: self.local_user_name.clone(),
            document_id: document_id.to_string(),
            position,
            selection_start: None,
            selection_end: None,
            color: self.local_color.clone(),
        };

        let _ = self.event_tx.send(AwarenessEvent::CursorMoved(cursor_info));
    }

    /// Set the local user's selection.
    pub fn set_selection(&self, document_id: &str, start: usize, end: usize) {
        let cursor = Cursor::with_selection(start, end);
        self.tracker.write().set_cursor(document_id, cursor);

        let cursor_info = CursorInfo {
            user_id: self.local_user_id.clone(),
            user_name: self.local_user_name.clone(),
            document_id: document_id.to_string(),
            position: end,
            selection_start: Some(start),
            selection_end: Some(end),
            color: self.local_color.clone(),
        };

        let _ = self.event_tx.send(AwarenessEvent::CursorMoved(cursor_info));
    }

    /// Set the local user's status.
    pub fn set_status(&self, status: UserStatus) {
        self.tracker.write().set_status(status);
    }

    /// Get all users' presence information.
    pub fn get_users(&self) -> Vec<UserPresenceInfo> {
        let tracker = self.tracker.read();

        tracker
            .all_users()
            .map(|presence| {
                let cursors: HashMap<String, CursorInfo> = presence
                    .cursors
                    .iter()
                    .map(|(doc_id, cursor): (&String, &Cursor)| {
                        let (sel_start, sel_end) = cursor
                            .selection_range()
                            .map(|(s, e)| (Some(s), Some(e)))
                            .unwrap_or((None, None));
                        (
                            doc_id.clone(),
                            CursorInfo {
                                user_id: presence.user_id.0.clone(),
                                user_name: presence.info.name.clone(),
                                document_id: doc_id.clone(),
                                position: cursor.position,
                                selection_start: sel_start,
                                selection_end: sel_end,
                                color: presence.info.color.clone(),
                            },
                        )
                    })
                    .collect();

                UserPresenceInfo {
                    user_id: presence.user_id.0.clone(),
                    name: presence.info.name.clone(),
                    status: presence.status.clone(),
                    color: presence.info.color.clone(),
                    cursors,
                }
            })
            .collect()
    }

    /// Get cursors for a specific document.
    pub fn get_cursors(&self, document_id: &str) -> Vec<CursorInfo> {
        self.get_users()
            .into_iter()
            .filter_map(|u| u.cursors.get(document_id).cloned())
            .collect()
    }

    /// Get the local user's assigned color.
    pub fn get_local_color(&self) -> &str {
        &self.local_color
    }

    /// Subscribe to awareness events.
    pub fn subscribe(&self) -> broadcast::Receiver<AwarenessEvent> {
        self.event_tx.subscribe()
    }

    /// Remove stale users who haven't been active.
    pub fn cleanup_stale(&self) {
        self.tracker.write().cleanup_stale();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awareness_basic() {
        let awareness = Awareness::new("user-1", "Alice");

        awareness.set_cursor("doc-1", 42);
        awareness.set_status(UserStatus::Online);

        let users = awareness.get_users();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].user_id, "user-1");
    }

    #[test]
    fn test_cursor_tracking() {
        let awareness = Awareness::new("user-1", "Alice");

        awareness.set_cursor("doc-1", 10);
        awareness.set_selection("doc-1", 10, 20);

        let cursors = awareness.get_cursors("doc-1");
        assert_eq!(cursors.len(), 1);
    }
}
