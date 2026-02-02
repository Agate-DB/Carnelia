//! Presence and Awareness Example
//!
//! This example demonstrates the presence system for tracking
//! users' cursor positions, selections, and online status.
//!
//! Run with: cargo run --example presence_demo

use mdcs_sdk::client::quick::create_collaborative_clients;
use mdcs_sdk::UserStatus;

fn main() {
    println!("=== Presence and Awareness Demo ===\n");

    // Create 4 connected clients
    let clients = create_collaborative_clients(&["Alice", "Bob", "Charlie", "Diana"]);
    
    println!("Connected users:");
    for client in &clients {
        println!("  - {} (color: {})", 
            client.user_name(),
            "#0066cc" // Default color
        );
    }
    println!();

    // Create sessions
    let sessions: Vec<_> = clients
        .iter()
        .map(|c| c.create_session("collaborative-document"))
        .collect();

    // Open a shared document
    let docs: Vec<_> = sessions
        .iter()
        .map(|s| s.open_text_doc("shared-doc.txt"))
        .collect();

    // Add some content
    {
        let mut doc = docs[0].write();
        doc.insert(0, "Hello, this is a collaborative document!\n");
        let pos1 = doc.len();
        doc.insert(pos1, "Multiple users can edit simultaneously.\n");
        let pos2 = doc.len();
        doc.insert(pos2, "Each user has a cursor position tracked.\n");
    }

    println!("Document content:");
    println!("---");
    println!("{}", docs[0].read().get_text());
    println!("---\n");

    // Simulate different user activities
    println!("=== User Activities ===\n");

    // Alice is at the beginning, actively typing
    println!("Alice: Typing at position 0");
    sessions[0].awareness().set_cursor("shared-doc.txt", 0);
    sessions[0].awareness().set_status(UserStatus::Typing);

    // Bob has selected some text
    println!("Bob: Selecting text (positions 7-38)");
    sessions[1].awareness().set_selection("shared-doc.txt", 7, 38);
    sessions[1].awareness().set_status(UserStatus::Online);

    // Charlie is idle, cursor at end
    println!("Charlie: Idle at end of document");
    sessions[2].awareness().set_cursor("shared-doc.txt", docs[2].read().len());
    sessions[2].awareness().set_status(UserStatus::Idle);

    // Diana is away
    println!("Diana: Away");
    sessions[3].awareness().set_status(UserStatus::Away);

    // View presence from each user's perspective
    println!("\n=== Presence Views ===\n");

    for (i, session) in sessions.iter().enumerate() {
        let user = clients[i].user_name();
        println!("--- {}'s view ---", user);
        
        // Get all users
        let users = session.awareness().get_users();
        println!("  Users in session: {}", users.len());
        
        for u in &users {
            let status_str = match &u.status {
                UserStatus::Online => "🟢 online",
                UserStatus::Typing => "⌨️  typing",
                UserStatus::Idle => "💤 idle",
                UserStatus::Away => "🔴 away",
                UserStatus::Offline => "⚫ offline",
                UserStatus::Custom(s) => s,
            };
            println!("    {} - {}", u.name, status_str);
        }
        
        // Get cursors for the document
        let cursors = session.awareness().get_cursors("shared-doc.txt");
        if !cursors.is_empty() {
            println!("  Cursors in document:");
            for cursor in &cursors {
                if let Some(start) = cursor.selection_start {
                    println!("    {} at {} (selection: {}-{})",
                        cursor.user_name, cursor.position, start, cursor.selection_end.unwrap_or(0));
                } else {
                    println!("    {} at position {}", cursor.user_name, cursor.position);
                }
            }
        }
        println!();
    }

    // Demonstrate real-time cursor movement
    println!("=== Simulating Cursor Movement ===\n");

    println!("Alice moves cursor from 0 → 10 → 20 → 30");
    for pos in [0, 10, 20, 30] {
        sessions[0].awareness().set_cursor("shared-doc.txt", pos);
        // In a real app, these updates would broadcast to all peers
        println!("  Alice's cursor now at {}", pos);
    }

    println!("\nBob changes selection: 7-38 → 50-70");
    sessions[1].awareness().set_selection("shared-doc.txt", 50, 70);
    println!("  Bob's new selection: 50-70");

    // Local user info
    println!("\n=== Local User Info ===\n");
    for session in &sessions {
        let awareness = session.awareness();
        println!("{}: color = {}", 
            awareness.local_user_name(),
            awareness.get_local_color()
        );
    }

    println!("\n=== Demo Complete ===");
}
