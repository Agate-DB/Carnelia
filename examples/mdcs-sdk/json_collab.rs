//! JSON Document Collaboration Example
//!
//! This example demonstrates collaborative editing of structured
//! JSON data, useful for configuration, forms, or data records.
//!
//! Run with: cargo run --example json_collab

use mdcs_sdk::client::quick::create_collaborative_clients;
use mdcs_sdk::JsonValue;

fn main() {
    println!("=== JSON Document Collaboration Example ===\n");

    // Create 3 connected clients for a team
    let clients = create_collaborative_clients(&["ProjectManager", "Developer", "Designer"]);
    
    println!("Team members connected:");
    for client in &clients {
        println!("  - {}", client.user_name());
    }
    println!();

    // Create sessions for the project
    let sessions: Vec<_> = clients
        .iter()
        .map(|c| c.create_session("project-alpha"))
        .collect();

    // All team members open the project configuration
    let docs: Vec<_> = sessions
        .iter()
        .map(|s| s.open_json_doc("project-config.json"))
        .collect();

    // Project Manager sets up the basic structure
    println!("ProjectManager creates the project structure...");
    {
        let mut doc = docs[0].write();
        doc.set("name", JsonValue::String("Project Alpha".to_string()));
        doc.set("version", JsonValue::String("1.0.0".to_string()));
        doc.set("status", JsonValue::String("in-progress".to_string()));
        doc.set("deadline", JsonValue::String("2025-03-01".to_string()));
    }
    
    println!("  Project: {:?}", docs[0].read().get("name"));
    println!("  Version: {:?}", docs[0].read().get("version"));
    println!("  Status: {:?}", docs[0].read().get("status"));
    println!();

    // Developer adds technical configuration
    println!("Developer adds technical settings...");
    {
        let mut doc = docs[1].write();
        doc.set("tech.language", JsonValue::String("Rust".to_string()));
        doc.set("tech.framework", JsonValue::String("MDCS".to_string()));
        doc.set("tech.min_rust_version", JsonValue::String("1.75.0".to_string()));
    }
    
    println!("  Language: {:?}", docs[1].read().get("tech.language"));
    println!("  Framework: {:?}", docs[1].read().get("tech.framework"));
    println!();

    // Designer adds UI configuration
    println!("Designer adds UI settings...");
    {
        let mut doc = docs[2].write();
        doc.set("ui.theme", JsonValue::String("dark".to_string()));
        doc.set("ui.primary_color", JsonValue::String("#3498db".to_string()));
        doc.set("ui.font_family", JsonValue::String("Inter".to_string()));
    }
    
    println!("  Theme: {:?}", docs[2].read().get("ui.theme"));
    println!("  Primary Color: {:?}", docs[2].read().get("ui.primary_color"));
    println!();

    // Show all keys from Project Manager's view
    println!("=== All Configuration Keys ===");
    let keys = docs[0].read().keys();
    for key in keys {
        println!("  - {}", key);
    }

    // Show the full JSON
    println!("\n=== Full Project Configuration ===");
    println!("{}", docs[0].read().root());

    // Demonstrate live updates
    println!("\n=== Live Update Demo ===");
    
    // Project Manager changes status
    println!("ProjectManager updates status to 'review'...");
    {
        docs[0].write().set("status", JsonValue::String("review".to_string()));
    }
    
    // In a real networked scenario, this would sync to all clients
    // For demo purposes, each client sees their local view
    println!("\nStatus values (each client's local view):");
    for (i, doc) in docs.iter().enumerate() {
        let status = doc.read().get("status");
        println!("  {}: {:?}", clients[i].user_name(), status);
    }

    println!("\n=== Demo Complete ===");
}
