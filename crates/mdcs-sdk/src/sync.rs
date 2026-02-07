//! Synchronization primitives for the SDK.

use crate::error::SdkError;
use crate::network::{Message, NetworkTransport, PeerId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Configuration for sync behavior.
#[derive(Clone, Debug)]
pub struct SyncConfig {
    /// How often to send sync requests (in milliseconds).
    pub sync_interval_ms: u64,
    /// How often to send presence updates (in milliseconds).
    pub presence_interval_ms: u64,
    /// Timeout for sync requests (in milliseconds).
    pub sync_timeout_ms: u64,
    /// Maximum batch size for delta updates.
    pub max_batch_size: usize,
    /// Enable automatic background sync.
    pub auto_sync: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_interval_ms: 1000,
            presence_interval_ms: 500,
            sync_timeout_ms: 5000,
            max_batch_size: 100,
            auto_sync: true,
        }
    }
}

/// Builder for sync configuration.
pub struct SyncConfigBuilder {
    config: SyncConfig,
}

impl SyncConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: SyncConfig::default(),
        }
    }

    pub fn sync_interval(mut self, ms: u64) -> Self {
        self.config.sync_interval_ms = ms;
        self
    }

    pub fn presence_interval(mut self, ms: u64) -> Self {
        self.config.presence_interval_ms = ms;
        self
    }

    pub fn sync_timeout(mut self, ms: u64) -> Self {
        self.config.sync_timeout_ms = ms;
        self
    }

    pub fn max_batch_size(mut self, size: usize) -> Self {
        self.config.max_batch_size = size;
        self
    }

    pub fn auto_sync(mut self, enabled: bool) -> Self {
        self.config.auto_sync = enabled;
        self
    }

    pub fn build(self) -> SyncConfig {
        self.config
    }
}

impl Default for SyncConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Events emitted by the sync manager.
#[derive(Clone, Debug)]
pub enum SyncEvent {
    /// Sync started with a peer.
    SyncStarted(PeerId),
    /// Sync completed with a peer.
    SyncCompleted(PeerId),
    /// Received update from peer.
    ReceivedUpdate {
        peer_id: PeerId,
        document_id: String,
    },
    /// Sent update to peer.
    SentUpdate {
        peer_id: PeerId,
        document_id: String,
    },
    /// Sync error occurred.
    SyncError { peer_id: PeerId, error: String },
}

/// Sync state for a peer.
#[derive(Clone, Debug, Default)]
pub struct PeerSyncState {
    /// Last known version for each document.
    pub document_versions: HashMap<String, u64>,
    /// Last sync time.
    pub last_sync: Option<Instant>,
}

/// Manages synchronization between peers.
pub struct SyncManager<T: NetworkTransport> {
    transport: Arc<T>,
    config: SyncConfig,
    peer_states: HashMap<PeerId, PeerSyncState>,
}

impl<T: NetworkTransport> SyncManager<T> {
    /// Create a new sync manager.
    pub fn new(transport: Arc<T>, config: SyncConfig) -> Self {
        Self {
            transport,
            config,
            peer_states: HashMap::new(),
        }
    }

    /// Get the sync configuration.
    pub fn config(&self) -> &SyncConfig {
        &self.config
    }

    /// Broadcast a document update to all connected peers.
    pub async fn broadcast_update(
        &mut self,
        document_id: &str,
        delta: Vec<u8>,
        version: u64,
    ) -> Result<(), SdkError> {
        let message = Message::Update {
            document_id: document_id.to_string(),
            delta,
            version,
        };

        self.transport
            .broadcast(message)
            .await
            .map_err(|e| SdkError::SyncError(e.to_string()))
    }

    /// Send a sync request to a specific peer.
    pub async fn request_sync(
        &mut self,
        peer_id: &PeerId,
        document_id: &str,
        version: u64,
    ) -> Result<(), SdkError> {
        let message = Message::SyncRequest {
            document_id: document_id.to_string(),
            version,
        };

        self.transport
            .send(peer_id, message)
            .await
            .map_err(|e| SdkError::SyncError(e.to_string()))
    }

    /// Update sync state for a peer.
    pub fn update_peer_state(&mut self, peer_id: &PeerId, document_id: &str, version: u64) {
        let state = self.peer_states.entry(peer_id.clone()).or_default();
        state
            .document_versions
            .insert(document_id.to_string(), version);
        state.last_sync = Some(Instant::now());
    }

    /// Get sync state for a peer.
    pub fn get_peer_state(&self, peer_id: &PeerId) -> Option<&PeerSyncState> {
        self.peer_states.get(peer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::MemoryTransport;

    #[test]
    fn test_sync_config_builder() {
        let config = SyncConfigBuilder::new()
            .sync_interval(500)
            .presence_interval(250)
            .sync_timeout(3000)
            .max_batch_size(50)
            .auto_sync(false)
            .build();

        assert_eq!(config.sync_interval_ms, 500);
        assert_eq!(config.presence_interval_ms, 250);
        assert_eq!(config.sync_timeout_ms, 3000);
        assert_eq!(config.max_batch_size, 50);
        assert!(!config.auto_sync);
    }

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let transport = Arc::new(MemoryTransport::new(PeerId::new("peer-1")));
        let config = SyncConfig::default();
        let manager = SyncManager::new(transport, config);

        assert!(manager.config().auto_sync);
    }
}
