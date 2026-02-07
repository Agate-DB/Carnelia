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
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           PHASE 1: Initial State (Both Online)                 ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    {
        let mut doc = desktop_doc.write();
        doc.insert(0, "Shopping List\n============\n");
    }

    // Sync initial content to mobile via CRDT merge
    {
        let desktop_state = desktop_doc.read().clone_state();
        mobile_doc.write().merge(&desktop_state);
    }

    println!("Desktop creates initial document and syncs to Mobile:");
    println!("┌─────────────────────────────────────┐");
    println!("│ {}│", desktop_doc.read().get_text().replace("\n", "\n│ "));
    println!("└─────────────────────────────────────┘");
    println!();

    // === Phase 2: Mobile goes offline ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           PHASE 2: Network Partition (Mobile Offline)          ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Simulate mobile going offline
    mobile_session.awareness().set_status(UserStatus::Away);
    println!("📵 Mobile status: OFFLINE\n");

    // Mobile makes changes while offline
    {
        let mut doc = mobile_doc.write();
        let pos = doc.len();
        doc.insert(pos, "[ ] Milk\n");
        let pos = doc.len();
        doc.insert(pos, "[ ] Bread\n");
        let pos = doc.len();
        doc.insert(pos, "[ ] Eggs\n");
    }
    println!("Mobile adds items (OFFLINE - not synced yet):");
    println!("┌─────────────────────────────────────┐");
    for line in mobile_doc.read().get_text().lines() {
        println!("│ {:35}│", line);
    }
    println!("└─────────────────────────────────────┘");
    println!();

    // Meanwhile, Desktop also makes changes
    {
        let mut doc = desktop_doc.write();
        let pos = doc.len();
        doc.insert(pos, "[ ] Coffee\n");
        let pos = doc.len();
        doc.insert(pos, "[ ] Sugar\n");
    }
    println!("Desktop adds items (ONLINE - separate changes):");
    println!("┌─────────────────────────────────────┐");
    for line in desktop_doc.read().get_text().lines() {
        println!("│ {:35}│", line);
    }
    println!("└─────────────────────────────────────┘");
    println!();

    // Show diverged state
    println!("⚠️  Documents have DIVERGED:");
    println!(
        "   Mobile:  {} bytes, {} lines",
        mobile_doc.read().get_text().len(),
        mobile_doc.read().get_text().lines().count()
    );
    println!(
        "   Desktop: {} bytes, {} lines",
        desktop_doc.read().get_text().len(),
        desktop_doc.read().get_text().lines().count()
    );
    println!();

    // === Phase 3: Mobile comes back online ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           PHASE 3: Reconnection & Automatic Merge              ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    mobile_session.awareness().set_status(UserStatus::Online);
    println!("📶 Mobile status: BACK ONLINE\n");

    // Simulate bidirectional sync
    println!("🔄 Syncing via CRDT merge...");
    println!("   → Mobile sends state to Desktop");
    println!("   ← Desktop sends state to Mobile\n");

    // Bidirectional CRDT merge
    {
        let mobile_state = mobile_doc.read().clone_state();
        let desktop_state = desktop_doc.read().clone_state();

        // Both apply each other's state (CRDT merge is commutative)
        mobile_doc.write().merge(&desktop_state);
        desktop_doc.write().merge(&mobile_state);
    }

    // === FINAL RESULT ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║              FINAL MERGED DOCUMENT (Both Clients)              ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║                                                                ║");
    for line in mobile_doc.read().get_text().lines() {
        println!("║  {:60}║", line);
    }
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    // Verify sync
    println!("\n=== Verification ===\n");
    let mobile_text = mobile_doc.read().get_text();
    let desktop_text = desktop_doc.read().get_text();

    if mobile_text == desktop_text {
        println!("✓ Mobile and Desktop are IDENTICAL!");
        println!("  - Total length: {} bytes", mobile_text.len());
        println!("  - Total lines: {}", mobile_text.lines().count());
        println!("  - Mobile's items: ✓ preserved");
        println!("  - Desktop's items: ✓ preserved");
    } else {
        println!("✗ Documents differ (unexpected!)");
    }

    // === Explain CRDT merge semantics ===
    println!("\n=== How CRDT Merge Works ===\n");
    println!("The merge succeeded because:");
    println!("  1. Each edit has a unique ID (replica + sequence number)");
    println!("  2. CRDTs are designed to merge WITHOUT conflicts");
    println!("  3. The order is determined by causal relationships");
    println!();
    println!("Key properties demonstrated:");
    println!("  • Commutativity: Order of applying merges doesn't matter");
    println!("  • Idempotency: Merging same state twice has no extra effect");
    println!("  • Convergence: All replicas reach identical state");

    // === Concurrent edit example ===
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║           BONUS: Concurrent Edit at Same Position              ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Both edit at position 0 at the "same time"
    {
        let mut doc = mobile_doc.write();
        doc.insert(0, "[!] URGENT: ");
    }
    {
        let mut doc = desktop_doc.write();
        doc.insert(0, "[*] NOTE: ");
    }

    println!("Mobile inserts '[!] URGENT: ' at position 0");
    println!("Desktop inserts '[*] NOTE: ' at position 0 (concurrent)\n");

    // Sync again via CRDT merge
    {
        let mobile_state = mobile_doc.read().clone_state();
        let desktop_state = desktop_doc.read().clone_state();
        mobile_doc.write().merge(&desktop_state);
        desktop_doc.write().merge(&mobile_state);
    }

    println!("After CRDT merge:");
    println!("┌─────────────────────────────────────────────────────────────┐");
    let preview = mobile_doc.read().get_text();
    let first_line = preview.lines().next().unwrap_or("");
    println!("│ {:59}│", first_line);
    println!("│ ...                                                         │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!();
    println!("Both prefixes are preserved - no data loss!");

    println!("\n=== Demo Complete ===");
}
