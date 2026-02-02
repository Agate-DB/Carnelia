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
    println!("Alice adds the title...");
    {
        let mut doc = docs[0].write();
        doc.insert(0, "# Team Meeting Notes\n\n");
    }
    println!("  Alice's view: {:?}", docs[0].read().get_text());

    // Bob adds an agenda item
    println!("\nBob adds an agenda item...");
    {
        let mut doc = docs[1].write();
        let content = doc.get_text();
        doc.insert(content.len(), "## Agenda\n- Review Q4 goals\n");
    }
    println!("  Bob's view: {:?}", docs[1].read().get_text());

    // Charlie adds another agenda item
    println!("\nCharlie adds another agenda item...");
    {
        let mut doc = docs[2].write();
        let content = doc.get_text();
        doc.insert(content.len(), "- Discuss team expansion\n");
    }
    println!("  Charlie's view: {:?}", docs[2].read().get_text());

    // In a real networked scenario, these changes would sync automatically.
    // For this demo, we show each user's local view.
    
    println!("\n=== Final Document Views ===\n");
    for (i, doc) in docs.iter().enumerate() {
        let user = clients[i].user_name();
        println!("--- {}'s view ---", user);
        println!("{}", doc.read().get_text());
    }

    // Demonstrate presence awareness
    println!("\n=== Presence Awareness ===\n");
    
    // Alice sets her cursor position
    sessions[0].awareness().set_cursor("meeting-notes.txt", 10);
    println!("Alice's cursor at position 10");
    
    // Bob sets a selection
    sessions[1].awareness().set_selection("meeting-notes.txt", 5, 15);
    println!("Bob selected text from 5 to 15");
    
    // Charlie checks who's in the document
    let users = sessions[2].awareness().get_users();
    println!("\nCharlie sees {} user(s) in the session:", users.len());
    for user in users {
        println!("  - {} (status: {:?})", user.name, user.status);
    }

    println!("\n=== Demo Complete ===");
}
