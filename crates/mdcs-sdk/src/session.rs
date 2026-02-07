//! Session management for collaborative editing sessions.

use crate::document::{JsonDoc, RichTextDoc, TextDoc};
use crate::error::SdkError;
use crate::network::{Message, NetworkTransport, Peer, PeerId};
use crate::presence::Awareness;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Events emitted by a session.
#[derive(Clone, Debug)]
pub enum SessionEvent {
    /// A peer joined the session.
    PeerJoined { peer_id: PeerId, user_name: String },
    /// A peer left the session.
    PeerLeft { peer_id: PeerId },
    /// A document was opened.
    DocumentOpened { document_id: String },
    /// A document was closed.
    DocumentClosed { document_id: String },
    /// Session connected.
    Connected,
    /// Session disconnected.
    Disconnected,
}

/// A collaborative session that manages documents and peers.
pub struct Session<T: NetworkTransport> {
    session_id: String,
    local_peer_id: PeerId,
    user_name: String,
    transport: Arc<T>,
    awareness: Arc<Awareness>,
    text_docs: Arc<RwLock<HashMap<String, Arc<RwLock<TextDoc>>>>>,
    rich_text_docs: Arc<RwLock<HashMap<String, Arc<RwLock<RichTextDoc>>>>>,
    json_docs: Arc<RwLock<HashMap<String, Arc<RwLock<JsonDoc>>>>>,
    event_tx: broadcast::Sender<SessionEvent>,
}

impl<T: NetworkTransport> Session<T> {
    /// Create a new session.
    pub fn new(
        session_id: impl Into<String>,
        local_peer_id: PeerId,
        user_name: impl Into<String>,
        transport: Arc<T>,
    ) -> Self {
        let session_id = session_id.into();
        let user_name = user_name.into();
        let (event_tx, _) = broadcast::channel(100);

        let awareness = Arc::new(Awareness::new(local_peer_id.0.clone(), user_name.clone()));

        Self {
            session_id,
            local_peer_id,
            user_name,
            transport,
            awareness,
            text_docs: Arc::new(RwLock::new(HashMap::new())),
            rich_text_docs: Arc::new(RwLock::new(HashMap::new())),
            json_docs: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the local peer ID.
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    /// Get the user name.
    pub fn user_name(&self) -> &str {
        &self.user_name
    }

    /// Get the awareness manager.
    pub fn awareness(&self) -> &Arc<Awareness> {
        &self.awareness
    }

    /// Subscribe to session events.
    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.event_tx.subscribe()
    }

    /// Connect to the session (announce presence to peers).
    pub async fn connect(&self) -> Result<(), SdkError> {
        let message = Message::Hello {
            replica_id: self.local_peer_id.0.clone(),
            user_name: self.user_name.clone(),
        };

        // Send hello to all connected peers
        self.transport
            .broadcast(message)
            .await
            .map_err(|e| SdkError::NetworkError(e.to_string()))?;

        let _ = self.event_tx.send(SessionEvent::Connected);

        Ok(())
    }

    /// Disconnect from the session.
    pub async fn disconnect(&self) -> Result<(), SdkError> {
        let _ = self.event_tx.send(SessionEvent::Disconnected);
        Ok(())
    }

    /// Create or open a text document.
    pub fn open_text_doc(&self, document_id: impl Into<String>) -> Arc<RwLock<TextDoc>> {
        let document_id = document_id.into();
        let mut docs = self.text_docs.write();

        if let Some(doc) = docs.get(&document_id) {
            doc.clone()
        } else {
            let doc = Arc::new(RwLock::new(TextDoc::new(
                document_id.clone(),
                self.local_peer_id.0.clone(),
            )));
            docs.insert(document_id.clone(), doc.clone());

            let _ = self
                .event_tx
                .send(SessionEvent::DocumentOpened { document_id });

            doc
        }
    }

    /// Create or open a rich text document.
    pub fn open_rich_text_doc(&self, document_id: impl Into<String>) -> Arc<RwLock<RichTextDoc>> {
        let document_id = document_id.into();
        let mut docs = self.rich_text_docs.write();

        if let Some(doc) = docs.get(&document_id) {
            doc.clone()
        } else {
            let doc = Arc::new(RwLock::new(RichTextDoc::new(
                document_id.clone(),
                self.local_peer_id.0.clone(),
            )));
            docs.insert(document_id.clone(), doc.clone());

            let _ = self
                .event_tx
                .send(SessionEvent::DocumentOpened { document_id });

            doc
        }
    }

    /// Create or open a JSON document.
    pub fn open_json_doc(&self, document_id: impl Into<String>) -> Arc<RwLock<JsonDoc>> {
        let document_id = document_id.into();
        let mut docs = self.json_docs.write();

        if let Some(doc) = docs.get(&document_id) {
            doc.clone()
        } else {
            let doc = Arc::new(RwLock::new(JsonDoc::new(
                document_id.clone(),
                self.local_peer_id.0.clone(),
            )));
            docs.insert(document_id.clone(), doc.clone());

            let _ = self
                .event_tx
                .send(SessionEvent::DocumentOpened { document_id });

            doc
        }
    }

    /// Close a document.
    pub fn close_doc(&self, document_id: &str) {
        self.text_docs.write().remove(document_id);
        self.rich_text_docs.write().remove(document_id);
        self.json_docs.write().remove(document_id);

        let _ = self.event_tx.send(SessionEvent::DocumentClosed {
            document_id: document_id.to_string(),
        });
    }

    /// Get list of open document IDs.
    pub fn open_documents(&self) -> Vec<String> {
        let mut docs = Vec::new();
        docs.extend(self.text_docs.read().keys().cloned());
        docs.extend(self.rich_text_docs.read().keys().cloned());
        docs.extend(self.json_docs.read().keys().cloned());
        docs
    }

    /// Get connected peers.
    pub async fn peers(&self) -> Vec<Peer> {
        self.transport.connected_peers().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::MemoryTransport;

    #[tokio::test]
    async fn test_session_creation() {
        let peer_id = PeerId::new("peer-1");
        let transport = Arc::new(MemoryTransport::new(peer_id.clone()));

        let session = Session::new("session-1", peer_id, "Alice", transport);

        assert_eq!(session.session_id(), "session-1");
        assert_eq!(session.user_name(), "Alice");
    }

    #[tokio::test]
    async fn test_document_management() {
        let peer_id = PeerId::new("peer-1");
        let transport = Arc::new(MemoryTransport::new(peer_id.clone()));

        let session = Session::new("session-1", peer_id, "Alice", transport);

        // Open documents
        let _text = session.open_text_doc("doc-1");
        let _rich = session.open_rich_text_doc("doc-2");
        let _json = session.open_json_doc("doc-3");

        let docs = session.open_documents();
        assert_eq!(docs.len(), 3);

        // Close a document
        session.close_doc("doc-1");

        let docs = session.open_documents();
        assert_eq!(docs.len(), 2);
    }
}
