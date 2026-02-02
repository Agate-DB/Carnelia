//! Network Simulation Example
//!
//! This example demonstrates the network layer abstraction
//! and how peers connect and communicate using the SDK.
//!
//! Run with: cargo run --example network_simulation

use mdcs_sdk::network::{create_network, MemoryTransport, NetworkTransport, PeerId, Message};

#[tokio::main]
async fn main() {
    println!("=== Network Simulation Example ===\n");

    // Create a network of 4 fully-connected peers
    println!("Creating a mesh network of 4 peers...\n");
    let transports = create_network(4);
    
    for transport in &transports {
        let peers = transport.connected_peers().await;
        println!("Peer {} connected to {} other peers:", 
            transport.local_id(),
            peers.len()
        );
        for peer in &peers {
            println!("  - {}", peer.id);
        }
    }
    println!();

    // Demonstrate message sending
    println!("=== Message Passing Demo ===\n");

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
    
    println!("Sending Hello from {} to {}...", sender_id, target);
    sender.send(&target, hello_msg.clone()).await.expect("send failed");
    
    // Receive the message on peer-1
    match tokio::time::timeout(
        std::time::Duration::from_millis(100),
        receiver_rx.recv()
    ).await {
        Ok(Some((from, msg))) => {
            println!("Received message from {}:", from);
            println!("  {:?}\n", msg);
        }
        _ => println!("  No message received (this is expected in some test setups)\n"),
    }

    // Demonstrate broadcast
    println!("=== Broadcast Demo ===\n");
    
    let update_msg = Message::Update {
        document_id: "shared-doc".to_string(),
        delta: vec![1, 2, 3, 4], // Simulated delta bytes
        version: 1,
    };
    
    println!("{} broadcasting document update...", sender_id);
    sender.broadcast(update_msg.clone()).await.expect("broadcast failed");
    println!("  Broadcast sent to all connected peers\n");

    // Demonstrate peer management
    println!("=== Peer Management Demo ===\n");
    
    // Create a new isolated transport
    let new_peer = MemoryTransport::new(PeerId::new("peer-new"));
    println!("Created new peer: {}", new_peer.local_id());
    
    // Initially not connected
    let peers = new_peer.connected_peers().await;
    println!("  Initial connections: {}", peers.len());
    
    // Connect to an existing peer
    new_peer.connect_to(&transports[0]);
    let peers = new_peer.connected_peers().await;
    println!("  After connecting to peer-0: {} connection(s)", peers.len());
    
    // The new peer can now communicate with peer-0
    println!("  Can now communicate with: {:?}", 
        peers.iter().map(|p| p.id.to_string()).collect::<Vec<_>>()
    );
    println!();

    // Message types overview
    println!("=== Available Message Types ===\n");
    
    let messages = vec![
        ("Hello", "Initial handshake with replica ID and user name"),
        ("SyncRequest", "Request sync state for a document"),
        ("SyncResponse", "Response with delta history"),
        ("Update", "Incremental document update"),
        ("Presence", "User cursor/selection update"),
        ("Ack", "Acknowledgment of received message"),
        ("Ping/Pong", "Keepalive messages"),
    ];
    
    for (name, desc) in messages {
        println!("  {:12} - {}", name, desc);
    }

    // Network topology
    println!("\n=== Network Topology ===\n");
    println!("The create_network() function creates a fully-connected mesh:");
    println!();
    println!("  peer-0 ←→ peer-1");
    println!("    ↕ \\   / ↕");
    println!("  peer-3 ←→ peer-2");
    println!();
    println!("Each peer can send messages directly to any other peer.");
    println!("Messages are delivered asynchronously via tokio channels.");

    println!("\n=== Demo Complete ===");
}
