//! Network transport abstractions for MDCS synchronization.

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Unique identifier for a peer.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub String);

impl PeerId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Peer connection state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PeerState {
    Disconnected,
    Connecting,
    Connected,
}

/// Information about a connected peer.
#[derive(Clone, Debug)]
pub struct Peer {
    pub id: PeerId,
    pub name: String,
    pub state: PeerState,
}

/// Messages exchanged between peers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    /// Hello/handshake message.
    Hello {
        replica_id: String,
        user_name: String,
    },
    /// Request sync for a document.
    SyncRequest { document_id: String, version: u64 },
    /// Response with deltas.
    SyncResponse {
        document_id: String,
        deltas: Vec<Vec<u8>>,
        version: u64,
    },
    /// Incremental update.
    Update {
        document_id: String,
        delta: Vec<u8>,
        version: u64,
    },
    /// Presence update.
    Presence {
        user_id: String,
        document_id: String,
        cursor_pos: Option<usize>,
    },
    /// Acknowledgment.
    Ack { message_id: u64 },
    /// Ping for keepalive.
    Ping,
    /// Pong response.
    Pong,
}

/// Network error type.
#[derive(Clone, Debug)]
pub enum NetworkError {
    ConnectionFailed(String),
    PeerNotFound(String),
    SendFailed(String),
    Disconnected,
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::ConnectionFailed(e) => write!(f, "Connection failed: {}", e),
            NetworkError::PeerNotFound(id) => write!(f, "Peer not found: {}", id),
            NetworkError::SendFailed(e) => write!(f, "Send failed: {}", e),
            NetworkError::Disconnected => write!(f, "Disconnected"),
        }
    }
}

impl std::error::Error for NetworkError {}

/// Abstract network transport trait.
#[async_trait]
pub trait NetworkTransport: Send + Sync + 'static {
    /// Connect to a peer.
    async fn connect(&self, peer_id: &PeerId) -> Result<(), NetworkError>;

    /// Disconnect from a peer.
    async fn disconnect(&self, peer_id: &PeerId) -> Result<(), NetworkError>;

    /// Send a message to a specific peer.
    async fn send(&self, peer_id: &PeerId, message: Message) -> Result<(), NetworkError>;

    /// Broadcast a message to all connected peers.
    async fn broadcast(&self, message: Message) -> Result<(), NetworkError>;

    /// Get list of connected peers.
    async fn connected_peers(&self) -> Vec<Peer>;

    /// Subscribe to incoming messages.
    fn subscribe(&self) -> mpsc::Receiver<(PeerId, Message)>;
}

/// Type alias for the message receiver shared across threads.
type SharedMessageReceiver = Arc<RwLock<Option<mpsc::Receiver<(PeerId, Message)>>>>;
/// Type alias for the outgoing message senders shared across threads.
type SharedOutgoing = Arc<RwLock<HashMap<PeerId, mpsc::Sender<(PeerId, Message)>>>>;

/// In-memory transport for testing and simulation.
pub struct MemoryTransport {
    local_id: PeerId,
    peers: Arc<RwLock<HashMap<PeerId, Peer>>>,
    message_tx: mpsc::Sender<(PeerId, Message)>,
    message_rx: SharedMessageReceiver,
    outgoing: SharedOutgoing,
}

impl MemoryTransport {
    pub fn new(local_id: PeerId) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            local_id,
            peers: Arc::new(RwLock::new(HashMap::new())),
            message_tx: tx,
            message_rx: Arc::new(RwLock::new(Some(rx))),
            outgoing: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn local_id(&self) -> &PeerId {
        &self.local_id
    }

    /// Connect two memory transports together (for testing).
    pub fn connect_to(&self, other: &MemoryTransport) {
        // Add peer to our list
        self.peers.write().insert(
            other.local_id.clone(),
            Peer {
                id: other.local_id.clone(),
                name: other.local_id.0.clone(),
                state: PeerState::Connected,
            },
        );

        // Set up channel to send to other
        self.outgoing
            .write()
            .insert(other.local_id.clone(), other.message_tx.clone());

        // Add us to other's peer list
        other.peers.write().insert(
            self.local_id.clone(),
            Peer {
                id: self.local_id.clone(),
                name: self.local_id.0.clone(),
                state: PeerState::Connected,
            },
        );

        // Set up channel for other to send to us
        other
            .outgoing
            .write()
            .insert(self.local_id.clone(), self.message_tx.clone());
    }
}

#[async_trait]
impl NetworkTransport for MemoryTransport {
    async fn connect(&self, peer_id: &PeerId) -> Result<(), NetworkError> {
        self.peers.write().insert(
            peer_id.clone(),
            Peer {
                id: peer_id.clone(),
                name: peer_id.0.clone(),
                state: PeerState::Connected,
            },
        );
        Ok(())
    }

    async fn disconnect(&self, peer_id: &PeerId) -> Result<(), NetworkError> {
        self.peers.write().remove(peer_id);
        self.outgoing.write().remove(peer_id);
        Ok(())
    }

    async fn send(&self, peer_id: &PeerId, message: Message) -> Result<(), NetworkError> {
        let tx = {
            let outgoing = self.outgoing.read();
            outgoing.get(peer_id).cloned()
        };

        if let Some(tx) = tx {
            tx.send((self.local_id.clone(), message))
                .await
                .map_err(|e| NetworkError::SendFailed(e.to_string()))?;
            Ok(())
        } else {
            Err(NetworkError::PeerNotFound(peer_id.to_string()))
        }
    }

    async fn broadcast(&self, message: Message) -> Result<(), NetworkError> {
        let senders: Vec<_> = {
            let outgoing = self.outgoing.read();
            outgoing.values().cloned().collect()
        };

        for tx in senders {
            let _ = tx.send((self.local_id.clone(), message.clone())).await;
        }
        Ok(())
    }

    async fn connected_peers(&self) -> Vec<Peer> {
        self.peers.read().values().cloned().collect()
    }

    fn subscribe(&self) -> mpsc::Receiver<(PeerId, Message)> {
        self.message_rx
            .write()
            .take()
            .expect("subscribe can only be called once")
    }
}

/// Create a network of connected memory transports for testing.
pub fn create_network(count: usize) -> Vec<MemoryTransport> {
    let transports: Vec<_> = (0..count)
        .map(|i| MemoryTransport::new(PeerId::new(format!("peer-{}", i))))
        .collect();

    // Connect all peers to each other
    for i in 0..count {
        for j in (i + 1)..count {
            transports[i].connect_to(&transports[j]);
        }
    }

    transports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_transport() {
        let transport1 = MemoryTransport::new(PeerId::new("peer-1"));
        let transport2 = MemoryTransport::new(PeerId::new("peer-2"));

        transport1.connect_to(&transport2);

        let peers1 = transport1.connected_peers().await;
        let peers2 = transport2.connected_peers().await;

        assert_eq!(peers1.len(), 1);
        assert_eq!(peers2.len(), 1);
    }

    #[tokio::test]
    async fn test_network_creation() {
        let network = create_network(3);
        assert_eq!(network.len(), 3);

        // Each peer should be connected to 2 others
        for transport in &network {
            let peers = transport.connected_peers().await;
            assert_eq!(peers.len(), 2);
        }
    }
}
