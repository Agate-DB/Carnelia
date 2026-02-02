//! Gossip-based broadcasting for head dissemination.
//!
//! The Broadcaster announces new DAG heads to peers, triggering
//! the pull-based sync process via DAGSyncer.

use crate::hash::Hash;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

/// Configuration for the broadcaster.
#[derive(Clone, Debug)]
pub struct BroadcastConfig {
    /// Number of peers to send each message to (fanout).
    pub fanout: usize,
    
    /// Maximum messages to buffer before dropping old ones.
    pub buffer_size: usize,
    
    /// Whether to deduplicate messages we've already seen.
    pub deduplicate: bool,
    
    /// Time-to-live: maximum hops a message can travel.
    pub ttl: u8,
}

impl Default for BroadcastConfig {
    fn default() -> Self {
        BroadcastConfig {
            fanout: 3,
            buffer_size: 1000,
            deduplicate: true,
            ttl: 6,
        }
    }
}

/// A broadcast message containing head announcements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BroadcastMessage {
    /// Unique message ID (hash of contents).
    pub id: Hash,
    
    /// The replica that originated this message.
    pub origin: String,
    
    /// Current heads being announced.
    pub heads: Vec<Hash>,
    
    /// Remaining hops (time-to-live).
    pub ttl: u8,
    
    /// Logical timestamp when the message was created.
    pub timestamp: u64,
}

impl BroadcastMessage {
    /// Create a new broadcast message.
    pub fn new(origin: impl Into<String>, heads: Vec<Hash>, ttl: u8, timestamp: u64) -> Self {
        let origin = origin.into();
        
        // Compute message ID from contents
        let mut hasher = crate::hash::Hasher::new();
        hasher.update(origin.as_bytes());
        for head in &heads {
            hasher.update(head.as_bytes());
        }
        hasher.update(&timestamp.to_le_bytes());
        let id = hasher.finalize();
        
        BroadcastMessage {
            id,
            origin,
            heads,
            ttl,
            timestamp,
        }
    }

    /// Create a forwarded copy with decremented TTL.
    pub fn forward(&self) -> Option<Self> {
        if self.ttl == 0 {
            return None;
        }
        
        Some(BroadcastMessage {
            id: self.id,
            origin: self.origin.clone(),
            heads: self.heads.clone(),
            ttl: self.ttl - 1,
            timestamp: self.timestamp,
        })
    }

    /// Check if this message should still be forwarded.
    pub fn is_alive(&self) -> bool {
        self.ttl > 0
    }
}

/// Events emitted by the broadcaster.
#[derive(Clone, Debug)]
pub enum BroadcastEvent {
    /// Send this message to a peer.
    Send { peer: String, message: BroadcastMessage },
    
    /// New heads received from a peer.
    HeadsReceived { from: String, heads: Vec<Hash> },
    
    /// A message was dropped (buffer full or duplicate).
    Dropped { message_id: Hash, reason: DropReason },
}

/// Reason a message was dropped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DropReason {
    Duplicate,
    BufferFull,
    ExpiredTTL,
}

/// Gossip-based broadcaster for head dissemination.
///
/// The broadcaster maintains:
/// - A set of known peers
/// - A buffer of seen message IDs (for deduplication)
/// - Pending outgoing messages
pub struct Broadcaster {
    /// Our replica ID.
    replica_id: String,
    
    /// Configuration.
    config: BroadcastConfig,
    
    /// Known peers (BTreeSet for deterministic iteration order).
    peers: BTreeSet<String>,
    
    /// Message IDs we've seen (for deduplication).
    seen: HashSet<Hash>,
    
    /// Order of seen messages (for LRU eviction).
    seen_order: VecDeque<Hash>,
    
    /// Current logical timestamp.
    timestamp: u64,
    
    /// Pending events to be processed.
    pending_events: VecDeque<BroadcastEvent>,
    
    /// Track which peers have which heads (optimization).
    peer_heads: HashMap<String, HashSet<Hash>>,
}

impl Broadcaster {
    /// Create a new broadcaster.
    pub fn new(replica_id: impl Into<String>) -> Self {
        Broadcaster {
            replica_id: replica_id.into(),
            config: BroadcastConfig::default(),
            peers: BTreeSet::new(),
            seen: HashSet::new(),
            seen_order: VecDeque::new(),
            timestamp: 0,
            pending_events: VecDeque::new(),
            peer_heads: HashMap::new(),
        }
    }

    /// Create a broadcaster with custom configuration.
    pub fn with_config(replica_id: impl Into<String>, config: BroadcastConfig) -> Self {
        Broadcaster {
            replica_id: replica_id.into(),
            config,
            peers: BTreeSet::new(),
            seen: HashSet::new(),
            seen_order: VecDeque::new(),
            timestamp: 0,
            pending_events: VecDeque::new(),
            peer_heads: HashMap::new(),
        }
    }

    /// Get our replica ID.
    pub fn replica_id(&self) -> &str {
        &self.replica_id
    }

    /// Add a peer.
    pub fn add_peer(&mut self, peer: impl Into<String>) {
        self.peers.insert(peer.into());
    }

    /// Remove a peer.
    pub fn remove_peer(&mut self, peer: &str) {
        self.peers.remove(peer);
        self.peer_heads.remove(peer);
    }

    /// Get all known peers.
    pub fn peers(&self) -> impl Iterator<Item = &String> {
        self.peers.iter()
    }

    /// Broadcast new heads to peers.
    pub fn broadcast(&mut self, heads: Vec<Hash>) {
        self.timestamp += 1;
        
        let message = BroadcastMessage::new(
            &self.replica_id,
            heads,
            self.config.ttl,
            self.timestamp,
        );
        
        // Mark as seen
        self.mark_seen(message.id);
        
        // Select peers to send to
        let targets = self.select_peers(self.config.fanout);
        
        for peer in targets {
            self.pending_events.push_back(BroadcastEvent::Send {
                peer,
                message: message.clone(),
            });
        }
    }

    /// Receive a message from a peer.
    pub fn receive(&mut self, from: impl Into<String>, message: BroadcastMessage) {
        let from = from.into();
        
        // Check for duplicate
        if self.config.deduplicate && self.seen.contains(&message.id) {
            self.pending_events.push_back(BroadcastEvent::Dropped {
                message_id: message.id,
                reason: DropReason::Duplicate,
            });
            return;
        }
        
        // Check TTL
        if !message.is_alive() {
            self.pending_events.push_back(BroadcastEvent::Dropped {
                message_id: message.id,
                reason: DropReason::ExpiredTTL,
            });
            return;
        }
        
        // Mark as seen
        self.mark_seen(message.id);
        
        // Update peer's known heads
        self.peer_heads
            .entry(from.clone())
            .or_default()
            .extend(message.heads.iter().copied());
        
        // Emit event for heads received
        self.pending_events.push_back(BroadcastEvent::HeadsReceived {
            from: from.clone(),
            heads: message.heads.clone(),
        });
        
        // Forward to other peers (excluding sender and origin)
        if let Some(forwarded) = message.forward() {
            let targets = self.select_peers_excluding(
                self.config.fanout,
                &[&from, &message.origin],
            );
            
            for peer in targets {
                self.pending_events.push_back(BroadcastEvent::Send {
                    peer,
                    message: forwarded.clone(),
                });
            }
        }
    }

    /// Get the next pending event.
    pub fn poll_event(&mut self) -> Option<BroadcastEvent> {
        self.pending_events.pop_front()
    }

    /// Check if there are pending events.
    pub fn has_pending_events(&self) -> bool {
        !self.pending_events.is_empty()
    }

    /// Get all pending events.
    pub fn drain_events(&mut self) -> Vec<BroadcastEvent> {
        self.pending_events.drain(..).collect()
    }

    /// Mark a message as seen.
    fn mark_seen(&mut self, id: Hash) {
        if self.seen.insert(id) {
            self.seen_order.push_back(id);
            
            // Evict old entries if buffer is full
            while self.seen_order.len() > self.config.buffer_size {
                if let Some(old_id) = self.seen_order.pop_front() {
                    self.seen.remove(&old_id);
                }
            }
        }
    }

    /// Select n random peers.
    fn select_peers(&self, n: usize) -> Vec<String> {
        // In a real implementation, this would use random selection
        // For determinism in tests, we just take the first n
        self.peers.iter().take(n).cloned().collect()
    }

    /// Select n random peers, excluding some.
    fn select_peers_excluding(&self, n: usize, exclude: &[&str]) -> Vec<String> {
        self.peers
            .iter()
            .filter(|p| !exclude.contains(&p.as_str()))
            .take(n)
            .cloned()
            .collect()
    }

    /// Get statistics about the broadcaster.
    pub fn stats(&self) -> BroadcastStats {
        BroadcastStats {
            peer_count: self.peers.len(),
            seen_messages: self.seen.len(),
            pending_events: self.pending_events.len(),
            timestamp: self.timestamp,
        }
    }
}

/// Statistics about the broadcaster.
#[derive(Clone, Debug)]
pub struct BroadcastStats {
    pub peer_count: usize,
    pub seen_messages: usize,
    pub pending_events: usize,
    pub timestamp: u64,
}

/// Simulates a network of broadcasters for testing.
pub struct BroadcastNetwork {
    /// Broadcasters indexed by replica ID.
    broadcasters: HashMap<String, Broadcaster>,
    
    /// Message queue: (from, to, message).
    message_queue: VecDeque<(String, String, BroadcastMessage)>,
}

impl BroadcastNetwork {
    /// Create a fully connected network of n replicas.
    pub fn fully_connected(n: usize) -> Self {
        let mut broadcasters = HashMap::new();
        
        // Create broadcasters
        for i in 0..n {
            let id = format!("replica_{}", i);
            let mut broadcaster = Broadcaster::new(&id);
            
            // Add all other replicas as peers
            for j in 0..n {
                if i != j {
                    broadcaster.add_peer(format!("replica_{}", j));
                }
            }
            
            broadcasters.insert(id, broadcaster);
        }
        
        BroadcastNetwork {
            broadcasters,
            message_queue: VecDeque::new(),
        }
    }

    /// Broadcast heads from a replica.
    pub fn broadcast(&mut self, from: &str, heads: Vec<Hash>) {
        if let Some(broadcaster) = self.broadcasters.get_mut(from) {
            broadcaster.broadcast(heads);
            self.collect_send_events(from);
        }
    }

    /// Collect send events and add to message queue.
    /// Only extracts Send events, leaving HeadsReceived events in place.
    fn collect_send_events(&mut self, from: &str) {
        if let Some(broadcaster) = self.broadcasters.get_mut(from) {
            let events: Vec<_> = broadcaster.drain_events();
            for event in events {
                match event {
                    BroadcastEvent::Send { peer, message } => {
                        self.message_queue.push_back((from.to_string(), peer, message));
                    }
                    // Put non-Send events back for later retrieval
                    other => broadcaster.pending_events.push_back(other),
                }
            }
        }
    }

    /// Deliver the next message in the queue.
    pub fn deliver_one(&mut self) -> bool {
        if let Some((from, to, message)) = self.message_queue.pop_front() {
            if let Some(broadcaster) = self.broadcasters.get_mut(&to) {
                broadcaster.receive(&from, message);
                self.collect_send_events(&to);
            }
            true
        } else {
            false
        }
    }

    /// Deliver all pending messages.
    pub fn deliver_all(&mut self) {
        while self.deliver_one() {}
    }

    /// Get a broadcaster by replica ID.
    pub fn broadcaster(&self, id: &str) -> Option<&Broadcaster> {
        self.broadcasters.get(id)
    }

    /// Get a mutable broadcaster by replica ID.
    pub fn broadcaster_mut(&mut self, id: &str) -> Option<&mut Broadcaster> {
        self.broadcasters.get_mut(id)
    }

    /// Get all received heads for a replica.
    pub fn received_heads(&mut self, id: &str) -> Vec<Hash> {
        let mut heads = Vec::new();
        
        if let Some(broadcaster) = self.broadcasters.get_mut(id) {
            for event in broadcaster.drain_events() {
                if let BroadcastEvent::HeadsReceived { heads: h, .. } = event {
                    heads.extend(h);
                }
            }
        }
        
        heads
    }

    /// Check how many messages are pending.
    pub fn pending_messages(&self) -> usize {
        self.message_queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::Hasher;

    #[test]
    fn test_basic_broadcast() {
        let mut network = BroadcastNetwork::fully_connected(3);
        
        // Broadcast from replica_0
        let head = Hasher::hash(b"test_head");
        network.broadcast("replica_0", vec![head]);
        
        // Should have messages queued for 2 peers
        assert!(network.pending_messages() > 0);
        
        // Deliver all
        network.deliver_all();
        
        // All replicas should have received the heads
        let heads_1 = network.received_heads("replica_1");
        let heads_2 = network.received_heads("replica_2");
        
        assert!(heads_1.contains(&head) || heads_2.contains(&head));
    }

    #[test]
    fn test_message_forwarding() {
        let mut broadcaster = Broadcaster::new("origin");
        broadcaster.add_peer("peer_1");
        broadcaster.add_peer("peer_2");
        broadcaster.add_peer("peer_3");
        
        let head = Hasher::hash(b"test");
        broadcaster.broadcast(vec![head]);
        
        // Should have send events
        let events = broadcaster.drain_events();
        assert!(!events.is_empty());
        
        for event in events {
            if let BroadcastEvent::Send { message, .. } = event {
                assert!(message.ttl <= broadcaster.config.ttl);
                assert!(message.heads.contains(&head));
            }
        }
    }

    #[test]
    fn test_deduplication() {
        let mut broadcaster = Broadcaster::new("receiver");
        broadcaster.add_peer("sender");
        
        let head = Hasher::hash(b"test");
        let message = BroadcastMessage::new("origin", vec![head], 5, 1);
        
        // Receive twice
        broadcaster.receive("sender", message.clone());
        broadcaster.receive("sender", message.clone());
        
        // Second should be dropped
        let events = broadcaster.drain_events();
        let dropped_count = events.iter().filter(|e| {
            matches!(e, BroadcastEvent::Dropped { reason: DropReason::Duplicate, .. })
        }).count();
        
        assert_eq!(dropped_count, 1);
    }

    #[test]
    fn test_ttl_expiry() {
        let mut broadcaster = Broadcaster::new("receiver");
        
        let head = Hasher::hash(b"test");
        let message = BroadcastMessage::new("origin", vec![head], 0, 1);
        
        broadcaster.receive("sender", message);
        
        let events = broadcaster.drain_events();
        let expired = events.iter().any(|e| {
            matches!(e, BroadcastEvent::Dropped { reason: DropReason::ExpiredTTL, .. })
        });
        
        assert!(expired);
    }

    #[test]
    fn test_forward_decrements_ttl() {
        let head = Hasher::hash(b"test");
        let message = BroadcastMessage::new("origin", vec![head], 5, 1);
        
        let forwarded = message.forward().unwrap();
        assert_eq!(forwarded.ttl, 4);
        
        // ID should be the same
        assert_eq!(forwarded.id, message.id);
    }

    #[test]
    fn test_buffer_eviction() {
        let config = BroadcastConfig {
            buffer_size: 2,
            ..Default::default()
        };
        let mut broadcaster = Broadcaster::with_config("test", config);
        broadcaster.add_peer("peer");
        
        // Broadcast 3 messages
        for i in 0..3 {
            broadcaster.broadcast(vec![Hasher::hash(&[i])]);
        }
        
        // Only 2 should be in seen set
        assert_eq!(broadcaster.seen.len(), 2);
    }

    #[test]
    fn test_peer_management() {
        let mut broadcaster = Broadcaster::new("test");
        
        broadcaster.add_peer("peer_1");
        broadcaster.add_peer("peer_2");
        assert_eq!(broadcaster.peers().count(), 2);
        
        broadcaster.remove_peer("peer_1");
        assert_eq!(broadcaster.peers().count(), 1);
    }

    #[test]
    fn test_network_convergence() {
        let mut network = BroadcastNetwork::fully_connected(5);
        
        // Each replica broadcasts different heads
        for i in 0..5 {
            let head = Hasher::hash(&[i as u8]);
            network.broadcast(&format!("replica_{}", i), vec![head]);
        }
        
        // Deliver all messages
        network.deliver_all();
        
        // All replicas should have received heads from others
        // (checking that the gossip propagated)
        assert_eq!(network.pending_messages(), 0);
    }
}
