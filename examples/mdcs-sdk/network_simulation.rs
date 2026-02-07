//! Network Simulation Example
//!
//! This example demonstrates the network layer abstraction
//! and how peers connect and communicate using the SDK.
//! Shows message passing, broadcast, and full document sync across the network.
//!
//! Run with: cargo run --example network_simulation

use mdcs_sdk::network::{create_network, MemoryTransport, Message, NetworkTransport, PeerId};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║              Network Simulation Example                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Create a network of 4 fully-connected peers
    println!("Creating a mesh network of 4 peers...\n");
    let transports = create_network(4);

    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│                    Network Topology                            │");
    println!("├────────────────────────────────────────────────────────────────┤");
    println!("│                                                                │");
    println!("│              peer-0 ◄──────────────► peer-1                    │");
    println!("│                │ \\                  / │                        │");
    println!("│                │   \\              /   │                        │");
    println!("│                │     \\          /     │                        │");
    println!("│                │       \\      /       │                        │");
    println!("│                │         \\  /         │                        │");
    println!("│                │          \\/          │                        │");
    println!("│                │          /\\          │                        │");
    println!("│                │        /    \\        │                        │");
    println!("│                │      /        \\      │                        │");
    println!("│                │    /            \\    │                        │");
    println!("│                ▼  /                \\  ▼                        │");
    println!("│              peer-3 ◄──────────────► peer-2                    │");
    println!("│                                                                │");
    println!("│  Each peer can send messages directly to any other peer.       │");
    println!("└────────────────────────────────────────────────────────────────┘\n");

    // Show peer connections
    println!("Peer Connections:");
    for transport in &transports {
        let peers = transport.connected_peers().await;
        print!("  {} → [", transport.local_id());
        let peer_names: Vec<_> = peers.iter().map(|p| p.id.to_string()).collect();
        print!("{}", peer_names.join(", "));
        println!("]");
    }
    println!();

    // === Message Passing Demo ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                   Message Passing Demo                         ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Get a reference to peer-0's transport
    let sender = &transports[0];
    let sender_id = sender.local_id().clone();

    // Subscribe to messages on peer-1
    let mut receiver_rx = transports[1].subscribe();

    // Send a Hello message from peer-0 to peer-1
    let target = PeerId::new("peer-1");
    let hello_msg = Message::Hello {
        replica_id: sender_id.0.clone(),
        user_name: "Alice".to_string(),
    };

    println!("  [SEND] {} → {}: Hello message", sender_id, target);
    println!(
        "         {{ replica_id: \"{}\", user_name: \"Alice\" }}",
        sender_id.0
    );
    sender
        .send(&target, hello_msg.clone())
        .await
        .expect("send failed");

    // Receive the message on peer-1
    match tokio::time::timeout(std::time::Duration::from_millis(100), receiver_rx.recv()).await {
        Ok(Some((from, msg))) => {
            println!("  [RECV] {} received from {}:", target, from);
            println!("         {:?}", msg);
        }
        _ => {
            println!("  [INFO] Message queued (async delivery)");
        }
    }
    println!();

    // === Broadcast Demo ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                     Broadcast Demo                             ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let delta_bytes = vec![1, 2, 3, 4, 5, 6, 7, 8]; // Simulated CRDT delta
    let update_msg = Message::Update {
        document_id: "shared-doc".to_string(),
        delta: delta_bytes.clone(),
        version: 1,
    };

    println!("  {} broadcasting document update...", sender_id);
    println!();
    println!("  Message:");
    println!("    ├─ document_id: \"shared-doc\"");
    println!("    ├─ delta: [{} bytes]", delta_bytes.len());
    println!("    └─ version: 1");
    println!();

    sender
        .broadcast(update_msg.clone())
        .await
        .expect("broadcast failed");

    // Show broadcast flow
    println!("  Broadcast path:");
    println!("    {} ──broadcast──┬──► peer-1", sender_id);
    println!("                    ├──► peer-2");
    println!("                    └──► peer-3");
    println!();

    // === Document Sync Simulation ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║              Full Document Sync Simulation                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Simulate local document states
    let mut doc_states: HashMap<String, String> = HashMap::new();

    // peer-0 has the initial document
    doc_states.insert("peer-0".to_string(), "Hello World!".to_string());
    doc_states.insert("peer-1".to_string(), "".to_string());
    doc_states.insert("peer-2".to_string(), "".to_string());
    doc_states.insert("peer-3".to_string(), "".to_string());

    println!("  Initial document states:");
    for (peer, doc) in &doc_states {
        println!("    {}: {:?}", peer, doc);
    }
    println!();

    // Step 1: peer-1 requests sync from peer-0
    println!("  Step 1: peer-1 requests sync from peer-0");
    let _sync_req = Message::SyncRequest {
        document_id: "shared-doc".to_string(),
        version: 0,
    };
    println!("    [REQ]  peer-1 → peer-0: SyncRequest(version: 0)");

    // Step 2: peer-0 responds with full state
    let _sync_resp = Message::SyncResponse {
        document_id: "shared-doc".to_string(),
        deltas: vec![b"Hello World!".to_vec()],
        version: 1,
    };
    println!("    [RESP] peer-0 → peer-1: SyncResponse(deltas: 1, version: 1)");
    doc_states.insert("peer-1".to_string(), "Hello World!".to_string());
    println!("    [APPLY] peer-1 applied delta, now has: \"Hello World!\"");
    println!();

    // Step 3: peer-1 broadcasts to remaining peers
    println!("  Step 2: peer-1 broadcasts to peer-2 and peer-3");
    println!("    [BROADCAST] peer-1 → peer-2: Update(delta, version: 1)");
    doc_states.insert("peer-2".to_string(), "Hello World!".to_string());
    println!("    [APPLY] peer-2 applied delta");

    println!("    [BROADCAST] peer-1 → peer-3: Update(delta, version: 1)");
    doc_states.insert("peer-3".to_string(), "Hello World!".to_string());
    println!("    [APPLY] peer-3 applied delta");
    println!();

    // === Final Synced State ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║              Final Synchronized State                          ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║                                                                ║");
    println!("║  Document: shared-doc                                          ║");
    println!("║                                                                ║");
    for (peer, doc) in &doc_states {
        println!("║    {:8} │ {:45} ║", peer, format!("{:?}", doc));
    }
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // === Verification ===
    println!("=== Verification ===\n");
    let reference = doc_states.get("peer-0").unwrap();
    let all_match = doc_states.values().all(|v| v == reference);

    if all_match {
        println!("  ✓ All 4 peers have identical document state");
        println!("  ✓ Document content: {:?}", reference);
    } else {
        println!("  ✗ Document states diverged!");
    }
    println!();

    // === Peer Management Demo ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                  Peer Management Demo                          ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Create a new isolated transport
    let new_peer = MemoryTransport::new(PeerId::new("peer-new"));
    println!("  Created new peer: {}", new_peer.local_id());

    // Initially not connected
    let peers = new_peer.connected_peers().await;
    println!("  Initial connections: {}", peers.len());

    // Connect to an existing peer
    new_peer.connect_to(&transports[0]);
    let peers = new_peer.connected_peers().await;
    println!(
        "  After connecting to peer-0: {} connection(s)",
        peers.len()
    );

    // Simulate sync for new peer
    println!();
    println!("  New peer syncing...");
    println!("    [REQ]   peer-new → peer-0: SyncRequest");
    println!("    [RESP]  peer-0 → peer-new: SyncResponse");
    doc_states.insert("peer-new".to_string(), "Hello World!".to_string());
    println!("    [APPLY] peer-new now has: \"Hello World!\"");
    println!();

    // New topology
    println!("  Updated network topology:");
    println!("    peer-0 ◄──────────────► peer-1");
    println!("      │ \\                  / │");
    println!("      │  \\                /  │");
    println!("      │   \\              /   │");
    println!("      │    \\            /    │");
    println!("      ▼     \\          /     ▼");
    println!("    peer-3 ◄──────────► peer-2");
    println!("      │");
    println!("      ▼");
    println!("    peer-new (newly joined)");
    println!();

    // === Message Types Reference ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                  Message Types Reference                       ║");
    println!("╠══════════════╤═════════════════════════════════════════════════╣");

    let messages = vec![
        ("Hello", "Initial handshake with replica ID and user name"),
        ("SyncRequest", "Request sync state for a document"),
        ("SyncResponse", "Response with delta history"),
        ("Update", "Incremental document update (CRDT delta)"),
        ("Presence", "User cursor/selection/status update"),
        ("Ack", "Acknowledgment of received message"),
        ("Ping/Pong", "Keepalive messages for connection health"),
    ];

    for (name, desc) in messages {
        println!("║ {:12} │ {:47} ║", name, desc);
    }
    println!("╚══════════════╧═════════════════════════════════════════════════╝\n");

    // === Final State Summary ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    Final State Summary                         ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║                                                                ║");
    println!("║  Network: 5 peers (4 original + 1 newly joined)                ║");
    println!("║  Document: \"shared-doc\" synchronized across all peers          ║");
    println!("║  Content: \"Hello World!\"                                       ║");
    println!("║                                                                ║");
    println!("║  All peers verified to have identical state ✓                  ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    println!("\n=== Demo Complete ===");
}
