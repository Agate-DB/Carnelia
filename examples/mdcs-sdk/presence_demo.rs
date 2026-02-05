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
        println!("  - {} (peer: {})", 
            client.user_name(),
            client.peer_id()
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

    // Add some content and sync
    {
        let mut doc = docs[0].write();
        doc.insert(0, "Hello, this is a collaborative document!\n");
        let pos1 = doc.len();
        doc.insert(pos1, "Multiple users can edit simultaneously.\n");
        let pos2 = doc.len();
        doc.insert(pos2, "Each user has a cursor position tracked.\n");
    }
    
    // Sync to all clients via CRDT merge
    {
        let alice_state = docs[0].read().clone_state();
        for i in 1..docs.len() {
            docs[i].write().merge(&alice_state);
        }
    }

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    Document Content                            ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    for line in docs[0].read().get_text().lines() {
        println!("║  {:60}║", line);
    }
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // === Simulate different user activities ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    User Activities                             ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Alice is at the beginning, actively typing
    println!("  Alice: Typing at position 0");
    sessions[0].awareness().set_cursor("shared-doc.txt", 0);
    sessions[0].awareness().set_status(UserStatus::Typing);

    // Bob has selected some text "this is a collaborative"
    println!("  Bob: Selecting text (positions 7-31)");
    sessions[1].awareness().set_selection("shared-doc.txt", 7, 31);
    sessions[1].awareness().set_status(UserStatus::Online);

    // Charlie is idle, cursor at end
    let doc_len = docs[2].read().len();
    println!("  Charlie: Idle at end of document (position {})", doc_len);
    sessions[2].awareness().set_cursor("shared-doc.txt", doc_len);
    sessions[2].awareness().set_status(UserStatus::Idle);

    // Diana is away
    println!("  Diana: Away (no cursor)");
    sessions[3].awareness().set_status(UserStatus::Away);

    // === View presence from each user's perspective ===
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║               Presence Views (Per User)                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    for (i, session) in sessions.iter().enumerate() {
        let user = clients[i].user_name();
        println!("┌─── {}'s view ───────────────────────────────────────────────┐", user);
        
        // Get all users
        let users = session.awareness().get_users();
        println!("│  Users in session: {}                                        │", users.len());
        
        for u in &users {
            let status_str = match &u.status {
                UserStatus::Online => "🟢 online ",
                UserStatus::Typing => "⌨️  typing ",
                UserStatus::Idle => "💤 idle   ",
                UserStatus::Away => "🔴 away   ",
                UserStatus::Offline => "⚫ offline",
                UserStatus::Custom(s) => s,
            };
            println!("│    {:8} - {}                                      │", u.name, status_str);
        }
        
        // Get cursors for the document
        let cursors = session.awareness().get_cursors("shared-doc.txt");
        if !cursors.is_empty() {
            println!("│  Cursors:                                                    │");
            for cursor in &cursors {
                if let Some(start) = cursor.selection_start {
                    println!("│    {} at pos {} (sel: {}-{})                            │",
                        cursor.user_name, cursor.position, start, cursor.selection_end.unwrap_or(0));
                } else {
                    println!("│    {} at position {}                                   │", 
                        cursor.user_name, cursor.position);
                }
            }
        }
        println!("└────────────────────────────────────────────────────────────────┘\n");
    }

    // === Real-time cursor movement simulation ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║            Simulating Real-time Cursor Movement                ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("  Alice moves cursor: 0 → 10 → 20 → 30");
    for pos in [0, 10, 20, 30] {
        sessions[0].awareness().set_cursor("shared-doc.txt", pos);
        println!("    [tick] Alice now at position {}", pos);
    }

    println!("\n  Bob changes selection: 7-31 → 42-82");
    sessions[1].awareness().set_selection("shared-doc.txt", 42, 82);
    println!("    Bob's new selection: 42-82 (second line)");

    println!("\n  Charlie starts typing:");
    sessions[2].awareness().set_status(UserStatus::Typing);
    sessions[2].awareness().set_cursor("shared-doc.txt", doc_len);
    println!("    Charlie: status changed to Typing");

    // === Final presence state ===
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║              Final Presence State (Synced)                     ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    
    // Show from Alice's perspective (any would work, they're synced)
    let all_users = sessions[0].awareness().get_users();
    let all_cursors = sessions[0].awareness().get_cursors("shared-doc.txt");
    
    println!("║                                                                ║");
    println!("║  Users ({} online):                                            ║", all_users.len());
    for u in &all_users {
        let status_emoji = match &u.status {
            UserStatus::Typing => "⌨️ ",
            UserStatus::Online => "🟢",
            UserStatus::Idle => "💤",
            UserStatus::Away => "🔴",
            _ => "⚫",
        };
        println!("║    {} {:8} ({:12})                               ║", 
            status_emoji, u.name, format!("{:?}", u.status));
    }
    
    println!("║                                                                ║");
    println!("║  Cursor Positions:                                             ║");
    for cursor in &all_cursors {
        if let Some(start) = cursor.selection_start {
            println!("║    {:8}: selection [{:3} - {:3}]                          ║",
                cursor.user_name, start, cursor.selection_end.unwrap_or(0));
        } else {
            println!("║    {:8}: position {:3}                                    ║",
                cursor.user_name, cursor.position);
        }
    }
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    // === Verification ===
    println!("\n=== Verification ===\n");
    
    // Check all sessions see the same user list
    let ref_count = sessions[0].awareness().get_users().len();
    let all_match = sessions.iter().all(|s| s.awareness().get_users().len() == ref_count);
    
    if all_match {
        println!("  ✓ All {} clients see {} users in session", sessions.len(), ref_count);
    }
    
    // Check cursor sync
    let ref_cursors = sessions[0].awareness().get_cursors("shared-doc.txt").len();
    let cursors_match = sessions.iter().all(|s| 
        s.awareness().get_cursors("shared-doc.txt").len() == ref_cursors
    );
    
    if cursors_match {
        println!("  ✓ All clients see {} cursors in document", ref_cursors);
    }

    // Local user info
    println!("\n=== Local User Colors ===\n");
    for (i, session) in sessions.iter().enumerate() {
        let awareness = session.awareness();
        let color = awareness.get_local_color();
        println!("  {} → {} (used for cursor highlighting)", 
            clients[i].user_name(), color);
    }

    println!("\n=== Demo Complete ===");
}
