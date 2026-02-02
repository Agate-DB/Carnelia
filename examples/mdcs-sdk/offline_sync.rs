//! Offline Sync Example
//!
//! This example demonstrates how MDCS handles offline editing
//! and synchronization when clients reconnect.
//!
//! Run with: cargo run --example offline_sync

use mdcs_sdk::client::quick::create_collaborative_clients;
use mdcs_sdk::UserStatus;

fn main() {
    println!("=== Offline Sync Example ===\n");

    // Create two clients that will simulate network partition
    let clients = create_collaborative_clients(&["Mobile", "Desktop"]);
    
    let mobile = &clients[0];
    let desktop = &clients[1];

    // Both clients start with connected sessions
    let mobile_session = mobile.create_session("notes");
    let desktop_session = desktop.create_session("notes");
    
    // Open the same document on both
    let mobile_doc = mobile_session.open_text_doc("shopping-list.txt");
    let desktop_doc = desktop_session.open_text_doc("shopping-list.txt");
    
    // === Phase 1: Initial sync (both online) ===
    println!("=== Phase 1: Both Online ===\n");
    
    {
        let mut doc = desktop_doc.write();
        doc.insert(0, "Shopping List\n============\n");
    }
    println!("Desktop creates initial document:");
    println!("{}", desktop_doc.read().get_text());

    // === Phase 2: Mobile goes offline ===
    println!("=== Phase 2: Mobile Goes Offline ===\n");
    
    // Simulate mobile going offline
    mobile_session.awareness().set_status(UserStatus::Away);
    println!("Mobile status: Offline (Away)");
    println!();
    
    // Mobile makes changes while offline
    {
        let mut doc = mobile_doc.write();
        doc.insert(0, "[ ] Milk\n");
        doc.insert(0, "[ ] Bread\n");
        doc.insert(0, "[ ] Eggs\n");
    }
    println!("Mobile adds items (offline):");
    println!("{}", mobile_doc.read().get_text());
    
    // Meanwhile, Desktop also makes changes
    {
        let mut doc = desktop_doc.write();
        let pos = doc.len();
        doc.insert(pos, "\n[ ] Coffee\n");
        let pos = doc.len();
        doc.insert(pos, "[ ] Sugar\n");
    }
    println!("Desktop adds items (online):");
    println!("{}", desktop_doc.read().get_text());

    // === Phase 3: Mobile comes back online ===
    println!("\n=== Phase 3: Mobile Reconnects ===\n");
    
    mobile_session.awareness().set_status(UserStatus::Online);
    println!("Mobile status: Back Online (Active)");
    println!();
    
    // In a real scenario, the sync layer would automatically 
    // exchange deltas. Here we show what each client sees:
    
    println!("Mobile's document (contains mobile's edits):");
    println!("{}", mobile_doc.read().get_text());
    
    println!("\nDesktop's document (contains desktop's edits):");
    println!("{}", desktop_doc.read().get_text());
    
    // === Explain CRDT merge semantics ===
    println!("\n=== How CRDT Merge Works ===\n");
    println!("In a fully integrated system:");
    println!("1. Mobile sends its deltas (Eggs, Bread, Milk inserts)");
    println!("2. Desktop sends its deltas (Coffee, Sugar inserts)");
    println!("3. Both apply each other's deltas");
    println!("4. CRDT merge ensures identical final state on both");
    println!();
    println!("The merged result preserves ALL edits:");
    println!("  - Mobile's items appear at the top (inserted first)");
    println!("  - Desktop's items appear at the bottom");
    println!("  - No data is lost, no conflicts to resolve!");
    
    // === Demonstrate delta accumulation ===
    println!("\n=== Delta Accumulation ===\n");
    
    // Show how many operations each document has accumulated
    println!("Documents track their edit history as deltas.");
    println!("When sync occurs, only the missing deltas are sent,");
    println!("not the entire document state.\n");
    
    println!("Mobile document length: {} bytes", mobile_doc.read().get_text().len());
    println!("Desktop document length: {} bytes", desktop_doc.read().get_text().len());
    
    // === Conflict-free resolution example ===
    println!("\n=== Concurrent Edit Example ===\n");
    
    // Both edit at position 0 at the "same time"
    {
        let mut doc = mobile_doc.write();
        doc.insert(0, "[!] Urgent: ");
    }
    {
        let mut doc = desktop_doc.write();
        doc.insert(0, "[*] Note: ");
    }
    
    println!("Mobile inserts '[!] Urgent: ' at position 0");
    println!("Desktop inserts '[*] Note: ' at position 0 (concurrent)");
    println!();
    println!("CRDT handles this automatically:");
    println!("  - Uses causal ordering to determine position");
    println!("  - Both edits are preserved");
    println!("  - Final document contains both prefixes");
    
    println!("\n=== Demo Complete ===");
}
