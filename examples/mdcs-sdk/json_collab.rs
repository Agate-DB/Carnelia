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
        println!("  - {} (peer: {})", client.user_name(), client.peer_id());
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

    // === Step 1: Project Manager sets up structure ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           Step 1: ProjectManager creates structure             ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    {
        let mut doc = docs[0].write();
        doc.set("name", JsonValue::String("Project Alpha".to_string()));
        doc.set("version", JsonValue::String("1.0.0".to_string()));
        doc.set("status", JsonValue::String("in-progress".to_string()));
        doc.set("deadline", JsonValue::String("2025-03-01".to_string()));
        doc.set("team_size", JsonValue::Float(3.0));
    }
    
    println!("ProjectManager created:");
    println!("  name: {:?}", docs[0].read().get("name"));
    println!("  version: {:?}", docs[0].read().get("version"));
    println!("  status: {:?}", docs[0].read().get("status"));
    println!("  deadline: {:?}", docs[0].read().get("deadline"));
    println!("  team_size: {:?}", docs[0].read().get("team_size"));
    
    // Sync to others via CRDT merge
    {
        let pm_state = docs[0].read().clone_state();
        for i in 1..docs.len() {
            docs[i].write().merge(&pm_state);
        }
    }
    println!("\n  [SYNC] → Developer, Designer\n");

    // === Step 2: Developer adds technical config ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           Step 2: Developer adds technical settings            ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    {
        let mut doc = docs[1].write();
        doc.set("tech.language", JsonValue::String("Rust".to_string()));
        doc.set("tech.framework", JsonValue::String("MDCS".to_string()));
        doc.set("tech.min_rust_version", JsonValue::String("1.75.0".to_string()));
        // Note: Arrays are created via array operations, setting individual feature flags instead
        doc.set("tech.feature_collaborative", JsonValue::Bool(true));
        doc.set("tech.feature_offline", JsonValue::Bool(true));
        doc.set("tech.feature_realtime", JsonValue::Bool(true));
    }
    
    println!("Developer added:");
    println!("  tech.language: {:?}", docs[1].read().get("tech.language"));
    println!("  tech.framework: {:?}", docs[1].read().get("tech.framework"));
    println!("  tech.min_rust_version: {:?}", docs[1].read().get("tech.min_rust_version"));
    println!("  tech.feature_collaborative: {:?}", docs[1].read().get("tech.feature_collaborative"));
    println!("  tech.feature_offline: {:?}", docs[1].read().get("tech.feature_offline"));
    println!("  tech.feature_realtime: {:?}", docs[1].read().get("tech.feature_realtime"));
    
    // Sync to others
    {
        let dev_state = docs[1].read().clone_state();
        for i in [0, 2] {
            docs[i].write().merge(&dev_state);
        }
    }
    println!("\n  [SYNC] → ProjectManager, Designer\n");

    // === Step 3: Designer adds UI config ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           Step 3: Designer adds UI configuration               ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    {
        let mut doc = docs[2].write();
        doc.set("ui.theme", JsonValue::String("dark".to_string()));
        doc.set("ui.primary_color", JsonValue::String("#3498db".to_string()));
        doc.set("ui.secondary_color", JsonValue::String("#2ecc71".to_string()));
        doc.set("ui.font_family", JsonValue::String("Inter".to_string()));
        doc.set("ui.font_size", JsonValue::Float(14.0));
        doc.set("ui.animations", JsonValue::Bool(true));
    }
    
    println!("Designer added:");
    println!("  ui.theme: {:?}", docs[2].read().get("ui.theme"));
    println!("  ui.primary_color: {:?}", docs[2].read().get("ui.primary_color"));
    println!("  ui.secondary_color: {:?}", docs[2].read().get("ui.secondary_color"));
    println!("  ui.font_family: {:?}", docs[2].read().get("ui.font_family"));
    println!("  ui.font_size: {:?}", docs[2].read().get("ui.font_size"));
    println!("  ui.animations: {:?}", docs[2].read().get("ui.animations"));
    
    // Sync to others
    {
        let design_state = docs[2].read().clone_state();
        for i in [0, 1] {
            docs[i].write().merge(&design_state);
        }
    }
    println!("\n  [SYNC] → ProjectManager, Developer\n");

    // === FINAL RESULT ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║              FINAL SYNCHRONIZED JSON DOCUMENT                  ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║                                                                ║");
    
    // Display JSON document structure
    let doc = docs[0].read();
    println!("║  {{                                                            ║");
    for key in doc.keys() {
        if let Some(value) = doc.get(&key) {
            println!("║    \"{}\": {:?},", key, value);
        }
    }
    println!("║  }}                                                            ║");
    
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    // === Verification ===
    println!("\n=== Verification: All Clients Synchronized ===\n");
    
    let reference_keys = docs[0].read().keys();
    let mut all_match = true;
    
    for (i, doc) in docs.iter().enumerate() {
        let user = clients[i].user_name();
        let keys = doc.read().keys();
        let matches = keys == reference_keys;
        let icon = if matches { "✓" } else { "✗" };
        println!("  {} {}: {} keys", icon, user, keys.len());
        if !matches {
            all_match = false;
        }
    }
    
    if all_match {
        println!("\n  ✓ All {} clients have identical JSON structure!", docs.len());
    }

    // === Live update demo ===
    println!("\n=== Live Update Demo ===\n");
    
    // Project Manager changes status
    println!("ProjectManager updates status: 'in-progress' → 'review'");
    {
        docs[0].write().set("status", JsonValue::String("review".to_string()));
    }
    
    // Sync the update
    {
        let pm_state = docs[0].read().clone_state();
        for i in 1..docs.len() {
            docs[i].write().merge(&pm_state);
        }
    }
    
    println!("\nAfter sync - all clients see:");
    for (i, doc) in docs.iter().enumerate() {
        let status = doc.read().get("status");
        println!("  {}: status = {:?}", clients[i].user_name(), status);
    }

    // === Concurrent edit scenario ===
    println!("\n=== Concurrent Edit Scenario ===\n");
    
    // Developer and Designer both edit different fields simultaneously
    println!("Developer changes: tech.min_rust_version = '1.80.0'");
    {
        docs[1].write().set("tech.min_rust_version", JsonValue::String("1.80.0".to_string()));
    }
    
    println!("Designer changes: ui.theme = 'light' (concurrent)");
    {
        docs[2].write().set("ui.theme", JsonValue::String("light".to_string()));
    }
    
    // Exchange states via CRDT merge
    {
        let dev_state = docs[1].read().clone_state();
        let design_state = docs[2].read().clone_state();
        
        docs[0].write().merge(&dev_state);
        docs[0].write().merge(&design_state);
        docs[1].write().merge(&design_state);
        docs[2].write().merge(&dev_state);
    }
    
    println!("\nAfter CRDT merge - both changes preserved:");
    println!("  tech.min_rust_version: {:?}", docs[0].read().get("tech.min_rust_version"));
    println!("  ui.theme: {:?}", docs[0].read().get("ui.theme"));
    
    println!("\n  ✓ No conflicts - JSON CRDT handles concurrent edits automatically!");

    println!("\n=== Demo Complete ===");
}
