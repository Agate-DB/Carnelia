//! Collaborative Text Editing Example
//!
//! This example demonstrates how multiple users can collaboratively
//! edit a shared text document using the MDCS SDK.
//!
//! Run with: cargo run --example collaborative_text

use mdcs_sdk::client::quick::create_collaborative_clients;

fn main() {
    println!("=== Collaborative Text Editing Example ===\n");

    // Create 3 connected clients (Alice, Bob, Charlie)
    let clients = create_collaborative_clients(&["Alice", "Bob", "Charlie"]);

    println!("Created {} connected clients:", clients.len());
    for client in &clients {
        println!("  - {} (peer: {})", client.user_name(), client.peer_id());
    }
    println!();

    // Each client creates a session for the same shared document
    let sessions: Vec<_> = clients
        .iter()
        .map(|c| c.create_session("meeting-notes"))
        .collect();

    // Each client opens the same document
    let docs: Vec<_> = sessions
        .iter()
        .map(|s| s.open_text_doc("meeting-notes.txt"))
        .collect();

    // Alice adds the title
    println!("Step 1: Alice adds the title...");
    {
        let mut doc = docs[0].write();
        doc.insert(0, "# Team Meeting Notes\n\n");
    }
    println!("  Alice's view: {:?}\n", docs[0].read().get_text());

    // Sync: Alice → Bob, Charlie
    println!("  [SYNC] Broadcasting Alice's changes to all peers...");
    {
        let alice_state = docs[0].read().clone_state();
        for i in 1..docs.len() {
            docs[i].write().merge(&alice_state);
        }
    }
    println!("  [SYNC] Complete\n");

    // Bob adds an agenda item
    println!("Step 2: Bob adds an agenda item...");
    {
        let mut doc = docs[1].write();
        let content = doc.get_text();
        doc.insert(content.len(), "## Agenda\n- Review Q4 goals\n");
    }
    println!("  Bob's view: {:?}\n", docs[1].read().get_text());

    // Sync: Bob → Alice, Charlie
    println!("  [SYNC] Broadcasting Bob's changes to all peers...");
    {
        let bob_state = docs[1].read().clone_state();
        for i in [0, 2] {
            docs[i].write().merge(&bob_state);
        }
    }
    println!("  [SYNC] Complete\n");

    // Charlie adds another agenda item
    println!("Step 3: Charlie adds another agenda item...");
    {
        let mut doc = docs[2].write();
        let content = doc.get_text();
        doc.insert(content.len(), "- Discuss team expansion\n");
    }
    println!("  Charlie's view: {:?}\n", docs[2].read().get_text());

    // Sync: Charlie → Alice, Bob
    println!("  [SYNC] Broadcasting Charlie's changes to all peers...");
    {
        let charlie_state = docs[2].read().clone_state();
        for i in [0, 1] {
            docs[i].write().merge(&charlie_state);
        }
    }
    println!("  [SYNC] Complete\n");

    // === FINAL RESULT: Show synchronized state ===
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              FINAL SYNCHRONIZED DOCUMENT                     ║");
    println!("╠══════════════════════════════════════════════════════════════╣");

    let final_text = docs[0].read().get_text();
    println!("║");
    for line in final_text.lines() {
        println!("║  {}", line);
    }
    println!("║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    // Verify all clients have identical state
    println!("\n=== Verification: All Clients Synchronized ===\n");
    let reference = docs[0].read().get_text();
    let mut all_match = true;
    for (i, doc) in docs.iter().enumerate() {
        let user = clients[i].user_name();
        let text = doc.read().get_text();
        let matches = text == reference;
        let icon = if matches { "✓" } else { "✗" };
        println!("  {} {}: {} chars", icon, user, text.len());
        if !matches {
            all_match = false;
        }
    }

    if all_match {
        println!(
            "\n  ✓ All {} clients have identical document state!",
            docs.len()
        );
    } else {
        println!("\n  ✗ Warning: Documents are not synchronized!");
    }

    // Demonstrate presence awareness
    println!("\n=== Presence Awareness ===\n");

    // Alice sets her cursor position
    sessions[0].awareness().set_cursor("meeting-notes.txt", 10);
    println!("  Alice's cursor at position 10");

    // Bob sets a selection
    sessions[1]
        .awareness()
        .set_selection("meeting-notes.txt", 5, 15);
    println!("  Bob selected text from 5 to 15");

    // Charlie checks who's in the document
    let users = sessions[2].awareness().get_users();
    println!("\n  Charlie sees {} user(s) in the session:", users.len());
    for user in users {
        println!("    - {} (status: {:?})", user.name, user.status);
    }

    println!("\n=== Demo Complete ===");
}
