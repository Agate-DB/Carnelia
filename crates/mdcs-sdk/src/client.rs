//! High-level client for the MDCS SDK.

use crate::error::SdkError;
use crate::network::{MemoryTransport, NetworkTransport, Peer, PeerId};
use crate::session::Session;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration for the MDCS client.
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// User name for presence.
    pub user_name: String,
    /// Enable automatic reconnection.
    pub auto_reconnect: bool,
    /// Maximum reconnection attempts.
    pub max_reconnect_attempts: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            user_name: "Anonymous".to_string(),
            auto_reconnect: true,
            max_reconnect_attempts: 5,
        }
    }
}

/// Builder for client configuration.
pub struct ClientConfigBuilder {
    config: ClientConfig,
}

impl ClientConfigBuilder {
    /// Create a new builder initialized with [`ClientConfig::default`].
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
        }
    }

    /// Set the display name advertised to other collaborators.
    pub fn user_name(mut self, name: impl Into<String>) -> Self {
        self.config.user_name = name.into();
        self
    }

    /// Enable or disable automatic reconnection attempts.
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.config.auto_reconnect = enabled;
        self
    }

    /// Set the maximum number of reconnect attempts.
    pub fn max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.config.max_reconnect_attempts = attempts;
        self
    }

    /// Build and return the final [`ClientConfig`].
    pub fn build(self) -> ClientConfig {
        self.config
    }
}

impl Default for ClientConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The main MDCS client for collaborative editing.
///
/// The client manages sessions, documents, and network connections.
///
/// # Example
///
/// ```rust
/// use mdcs_sdk::{Client, ClientConfig};
///
/// // Create a client
/// let config = ClientConfig {
///     user_name: "Alice".to_string(),
///     ..Default::default()
/// };
/// let client = Client::new_with_memory_transport(config);
///
/// // Create a session
/// let session = client.create_session("my-session");
///
/// // Open a document
/// let doc = session.open_text_doc("shared-doc");
/// doc.write().insert(0, "Hello, world!");
/// ```
pub struct Client<T: NetworkTransport> {
    peer_id: PeerId,
    config: ClientConfig,
    transport: Arc<T>,
    sessions: Arc<RwLock<HashMap<String, Arc<Session<T>>>>>,
}

impl Client<MemoryTransport> {
    /// Create a new client backed by [`MemoryTransport`].
    ///
    /// This constructor is ideal for tests, local demos, and examples where
    /// all peers run in the same process.
    pub fn new_with_memory_transport(config: ClientConfig) -> Self {
        let peer_id = PeerId::new(format!("peer-{}", uuid_simple()));
        let transport = Arc::new(MemoryTransport::new(peer_id.clone()));

        Self {
            peer_id,
            config,
            transport,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<T: NetworkTransport> Client<T> {
    /// Create a new client with a custom transport implementation.
    ///
    /// Use this constructor when integrating with a real networking backend
    /// (WebSocket, QUIC, custom RPC, etc.).
    pub fn new(peer_id: PeerId, transport: Arc<T>, config: ClientConfig) -> Self {
        Self {
            peer_id,
            config,
            transport,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Return the unique identifier of this local peer.
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    /// Return the configured local user name.
    pub fn user_name(&self) -> &str {
        &self.config.user_name
    }

    /// Return the transport used by this client.
    pub fn transport(&self) -> &Arc<T> {
        &self.transport
    }

    /// Create or fetch a collaborative session by ID.
    ///
    /// If the session already exists, this returns the same shared instance.
    pub fn create_session(&self, session_id: impl Into<String>) -> Arc<Session<T>> {
        let session_id = session_id.into();
        let mut sessions = self.sessions.write();

        if let Some(session) = sessions.get(&session_id) {
            session.clone()
        } else {
            let session = Arc::new(Session::new(
                session_id.clone(),
                self.peer_id.clone(),
                self.config.user_name.clone(),
                self.transport.clone(),
            ));
            sessions.insert(session_id, session.clone());
            session
        }
    }

    /// Get an existing session if it has already been created on this client.
    pub fn get_session(&self, session_id: &str) -> Option<Arc<Session<T>>> {
        self.sessions.read().get(session_id).cloned()
    }

    /// Close a local session handle and remove it from the client cache.
    ///
    /// This does not notify remote peers directly; use higher-level app
    /// signaling if your protocol requires explicit leave semantics.
    pub fn close_session(&self, session_id: &str) {
        self.sessions.write().remove(session_id);
    }

    /// List all currently active local session IDs.
    pub fn session_ids(&self) -> Vec<String> {
        self.sessions.read().keys().cloned().collect()
    }

    /// Establish a transport-level connection to a peer.
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::ConnectionFailed`] if the underlying transport
    /// cannot connect to the target peer.
    pub async fn connect_peer(&self, peer_id: &PeerId) -> Result<(), SdkError> {
        self.transport
            .connect(peer_id)
            .await
            .map_err(|e| SdkError::ConnectionFailed(e.to_string()))
    }

    /// Disconnect from a peer.
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::NetworkError`] if the transport reports a failure
    /// while disconnecting.
    pub async fn disconnect_peer(&self, peer_id: &PeerId) -> Result<(), SdkError> {
        self.transport
            .disconnect(peer_id)
            .await
            .map_err(|e| SdkError::NetworkError(e.to_string()))
    }

    /// Return the current list of connected peers reported by the transport.
    pub async fn connected_peers(&self) -> Vec<Peer> {
        self.transport.connected_peers().await
    }
}

/// Simple UUID-like string generator.
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}

/// Convenience functions for quickly creating collaborative sessions.
pub mod quick {
    use super::*;
    use crate::network::create_network;

    /// Create a fully connected in-memory set of collaborative clients.
    ///
    /// The returned clients can immediately open the same session/document IDs
    /// and exchange updates in examples or tests.
    ///
    /// # Panics
    ///
    /// Panics if `user_names` and generated transports differ in length, which
    /// should not happen for a correctly constructed in-memory network.
    pub fn create_collaborative_clients(user_names: &[&str]) -> Vec<Client<MemoryTransport>> {
        let network = create_network(user_names.len());

        user_names
            .iter()
            .zip(network)
            .map(|(name, transport)| {
                let peer_id = transport.local_id().clone();
                let config = ClientConfig {
                    user_name: name.to_string(),
                    ..Default::default()
                };
                Client::new(peer_id, Arc::new(transport), config)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = ClientConfig {
            user_name: "Alice".to_string(),
            ..Default::default()
        };
        let client = Client::new_with_memory_transport(config);

        assert_eq!(client.user_name(), "Alice");
    }

    #[test]
    fn test_session_management() {
        let config = ClientConfig::default();
        let client = Client::new_with_memory_transport(config);

        let session1 = client.create_session("session-1");
        let _session2 = client.create_session("session-2");

        assert_eq!(client.session_ids().len(), 2);

        // Getting same session returns same instance
        let session1_again = client.create_session("session-1");
        assert!(Arc::ptr_eq(&session1, &session1_again));

        // Close session
        client.close_session("session-1");
        assert_eq!(client.session_ids().len(), 1);
    }

    #[test]
    fn test_config_builder() {
        let config = ClientConfigBuilder::new()
            .user_name("Bob")
            .auto_reconnect(false)
            .max_reconnect_attempts(3)
            .build();

        assert_eq!(config.user_name, "Bob");
        assert!(!config.auto_reconnect);
        assert_eq!(config.max_reconnect_attempts, 3);
    }

    #[test]
    fn test_quick_collaborative_clients() {
        let clients = quick::create_collaborative_clients(&["Alice", "Bob", "Charlie"]);

        assert_eq!(clients.len(), 3);
        assert_eq!(clients[0].user_name(), "Alice");
        assert_eq!(clients[1].user_name(), "Bob");
        assert_eq!(clients[2].user_name(), "Charlie");
    }
}
