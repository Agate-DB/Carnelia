use mdcs_core::gset::GSet;
use mdcs_core::lattice::Lattice;
use mdcs_core::orset::ORSet;
use mdcs_core::pncounter::PNCounter;
use mdcs_core::lwwreg::LWWRegister;
use mdcs_core::mvreg::MVRegister;
use mdcs_core::map::{CRDTMap, MapValue};

/// Real-world example 1: Collaborative tags set (GSet)
/// A growing collection of tags that users add to posts/items
fn example_collaborative_tags() {
    println!("\n=== Example 1: Collaborative Tags (GSet) ===");

    let mut tags_replica1 = GSet::new();
    let mut tags_replica2 = GSet::new();

    // Replica 1 adds tags
    tags_replica1.insert("rust".to_string());
    tags_replica1.insert("crdt".to_string());
    tags_replica1.insert("distributed-systems".to_string());

    println!("Replica 1 tags: {:?}", tags_replica1.iter().collect::<Vec<_>>());

    // Replica 2 adds different tags
    tags_replica2.insert("database".to_string());
    tags_replica2.insert("crdt".to_string());
    tags_replica2.insert("offline-first".to_string());

    println!("Replica 2 tags: {:?}", tags_replica2.iter().collect::<Vec<_>>());

    // Merge replicas (converge)
    let merged = tags_replica1.join(&tags_replica2);
    println!("Merged tags: {:?}", merged.iter().collect::<Vec<_>>());
    println!("Total unique tags: {}", merged.len());
}

/// Real-world example 2: Shared shopping cart (ORSet)
/// Items can be added and removed, but concurrent add/remove favors add (last-write-wins semantics)
fn example_shopping_cart() {
    println!("\n=== Example 2: Shared Shopping Cart (ORSet) ===");

    let mut cart_user_a = ORSet::new();
    let mut cart_user_b = ORSet::new();

    // User A adds items
    cart_user_a.add("alice", "laptop".to_string());
    cart_user_a.add("alice", "mouse".to_string());
    cart_user_a.add("alice", "keyboard".to_string());

    println!("User A's cart: {:?}", cart_user_a.iter().collect::<Vec<_>>());

    // User B adds items (different device)
    cart_user_b.add("bob", "laptop".to_string());
    cart_user_b.add("bob", "monitor".to_string());

    println!("User B's cart: {:?}", cart_user_b.iter().collect::<Vec<_>>());

    // User A removes keyboard
    cart_user_a.remove(&"keyboard".to_string());

    // Merge both carts
    let merged_cart = cart_user_a.join(&cart_user_b);
    println!("Merged cart: {:?}", merged_cart.iter().collect::<Vec<_>>());
    println!("Items in cart: {}", merged_cart.len());
}

/// Real-world example 3: Multi-user set operations
/// Simulating a collaborative whiteboard where users draw/erase shapes
fn example_collaborative_whiteboard() {
    println!("\n=== Example 3: Collaborative Whiteboard (ORSet) ===");

    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
    struct Shape {
        id: String,
        shape_type: String,
        x: i32,
        y: i32,
    }

    let mut board_designer = ORSet::new();
    let mut board_developer = ORSet::new();

    // Designer draws shapes
    board_designer.add("designer", Shape {
        id: "circle1".to_string(),
        shape_type: "circle".to_string(),
        x: 100,
        y: 100,
    });
    board_designer.add("designer", Shape {
        id: "rect1".to_string(),
        shape_type: "rectangle".to_string(),
        x: 200,
        y: 150,
    });

    println!("Designer's board: {} shapes", board_designer.len());

    // Developer independently draws
    board_developer.add("developer", Shape {
        id: "line1".to_string(),
        shape_type: "line".to_string(),
        x: 50,
        y: 50,
    });

    // Both collaborate and merge
    let merged_board = board_designer.join(&board_developer);
    println!("Merged board: {} shapes", merged_board.len());
}

/// Real-world example 4: Presence tracking (ORSet)
/// Track which users are currently online in a collaborative session
fn example_presence_tracking() {
    println!("\n=== Example 4: Presence Tracking (ORSet) ===");

    let mut presence_node1 = ORSet::new();
    let mut presence_node2 = ORSet::new();

    // Node 1 sees some users online
    presence_node1.add("node1", "alice".to_string());
    presence_node1.add("node1", "bob".to_string());
    presence_node1.add("node1", "charlie".to_string());

    println!("Node 1 sees {} users online", presence_node1.len());

    // Node 2 sees different users
    presence_node2.add("node2", "bob".to_string());
    presence_node2.add("node2", "david".to_string());

    // Charlie leaves (observed on node1)
    presence_node1.remove(&"charlie".to_string());

    // Sync nodes
    let merged_presence = presence_node1.join(&presence_node2);
    println!("After merge, online users: {:?}",
             merged_presence.iter().collect::<Vec<_>>());
    println!("Total active users: {}", merged_presence.len());
}

/// Real-world example 5: Lattice properties verification
/// Demonstrates the mathematical properties that make CRDTs work
fn example_lattice_properties() {
    println!("\n=== Example 5: Lattice Properties ===");

    let mut set_a = GSet::new();
    set_a.insert(1);
    set_a.insert(2);

    let mut set_b = GSet::new();
    set_b.insert(2);
    set_b.insert(3);

    let mut set_c = GSet::new();
    set_c.insert(3);
    set_c.insert(4);

    // Commutativity: a ⊔ b = b ⊔ a
    let ab = set_a.join(&set_b);
    let ba = set_b.join(&set_a);
    println!("Commutativity: a⊔b == b⊔a: {}", ab == ba);

    // Associativity: (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
    let left = set_a.join(&set_b).join(&set_c);
    let right = set_a.join(&set_b.join(&set_c));
    println!("Associativity: (a⊔b)⊔c == a⊔(b⊔c): {}", left == right);

    // Idempotence: a ⊔ a = a
    let idempotent = set_a.join(&set_a);
    println!("Idempotence: a⊔a == a: {}", idempotent == set_a);

    // Monotonicity: if a ≤ b, then a ⊔ c ≤ b ⊔ c
    println!("Join results contain all elements: {}",
             set_a.iter().all(|x| ab.contains(x)) &&
                 set_b.iter().all(|x| ab.contains(x)));
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Carnelia - Merkle-Delta CRDT Store - Real-World Examples  ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    example_collaborative_tags();
    example_shopping_cart();
    example_collaborative_whiteboard();
    example_presence_tracking();
    example_lattice_properties();
    example_distributed_counter();
    example_user_profile_lww();
    example_conflict_aware_editing();
    example_document_store();

    println!("\n✓ All examples completed successfully!");
}

/// Real-world example 6: Distributed view counter (PNCounter)
/// Multiple servers tracking likes/unlikes on content
fn example_distributed_counter() {
    println!("\n=== Example 6: Distributed View Counter (PNCounter) ===");

    let mut counter_server1: PNCounter<String> = PNCounter::new();
    let mut counter_server2: PNCounter<String> = PNCounter::new();
    let mut counter_server3: PNCounter<String> = PNCounter::new();

    // Server 1 receives likes
    counter_server1.increment("server1".to_string(), 150);
    println!("Server 1 likes: +150, value = {}", counter_server1.value());

    // Server 2 receives likes and some unlikes
    counter_server2.increment("server2".to_string(), 80);
    counter_server2.decrement("server2".to_string(), 10);
    println!("Server 2 likes: +80, unlikes: -10, value = {}", counter_server2.value());

    // Server 3 receives mostly unlikes (spam removal)
    counter_server3.increment("server3".to_string(), 20);
    counter_server3.decrement("server3".to_string(), 5);
    println!("Server 3 likes: +20, unlikes: -5, value = {}", counter_server3.value());

    // Merge all servers
    let merged = counter_server1.join(&counter_server2).join(&counter_server3);
    println!("\nMerged total: {} net likes", merged.value());

    // Verify convergence regardless of merge order
    let merged_alt = counter_server3.join(&counter_server1).join(&counter_server2);
    println!("Convergence verified: {}", merged.value() == merged_alt.value());
}

/// Real-world example 7: User profile with LWW semantics
/// Last update wins for profile fields
fn example_user_profile_lww() {
    println!("\n=== Example 7: User Profile (LWW Register) ===");

    // User edits profile from phone
    let mut profile_phone: LWWRegister<String, String> = LWWRegister::new("phone".to_string());
    profile_phone.set("Alice Smith".to_string(), 1000, "phone".to_string());
    println!("Phone update (t=1000): '{}'", profile_phone.get().unwrap());

    // User edits profile from laptop (earlier timestamp)
    let mut profile_laptop: LWWRegister<String, String> = LWWRegister::new("laptop".to_string());
    profile_laptop.set("Alice S.".to_string(), 900, "laptop".to_string());
    println!("Laptop update (t=900): '{}'", profile_laptop.get().unwrap());

    // User edits from tablet (latest timestamp)
    let mut profile_tablet: LWWRegister<String, String> = LWWRegister::new("tablet".to_string());
    profile_tablet.set("Dr. Alice Smith".to_string(), 1100, "tablet".to_string());
    println!("Tablet update (t=1100): '{}'", profile_tablet.get().unwrap());

    // Merge all - latest timestamp wins
    let merged = profile_phone.join(&profile_laptop).join(&profile_tablet);
    println!("\nMerged profile: '{}' (timestamp {})",
             merged.get().unwrap(), merged.timestamp());

    // Demonstrate tie-breaking by replica ID
    let mut reg_a: LWWRegister<i32, String> = LWWRegister::new("a".to_string());
    let mut reg_b: LWWRegister<i32, String> = LWWRegister::new("b".to_string());
    reg_a.set(100, 500, "a".to_string());
    reg_b.set(200, 500, "b".to_string());
    let tie_break = reg_a.join(&reg_b);
    println!("Tie-break (same timestamp): {} wins (higher replica ID)",
             tie_break.get().unwrap());
}

/// Real-world example 8: Conflict-aware collaborative editing (MVRegister)
/// Preserve all concurrent edits for user resolution
fn example_conflict_aware_editing() {
    println!("\n=== Example 8: Conflict-Aware Editing (MVRegister) ===");

    let mut doc_alice: MVRegister<String> = MVRegister::new();
    let mut doc_bob: MVRegister<String> = MVRegister::new();

    // Alice writes her version
    doc_alice.write("alice", "The quick brown fox".to_string());
    println!("Alice wrote: {:?}", doc_alice.read());

    // Bob writes his version (concurrently)
    doc_bob.write("bob", "A lazy dog sleeps".to_string());
    println!("Bob wrote: {:?}", doc_bob.read());

    // Merge - both versions preserved!
    let merged = doc_alice.join(&doc_bob);
    let all_versions = merged.read();
    println!("\nMerged document has {} concurrent versions:", all_versions.len());
    for (i, version) in all_versions.iter().enumerate() {
        println!("  Version {}: '{}'", i + 1, version);
    }

    // User resolves conflict by choosing/combining
    let mut resolved = merged.clone();
    resolved.resolve("charlie", "The quick brown fox jumps over a lazy dog".to_string());
    println!("\nAfter resolution: {:?}", resolved.read());
}

/// Real-world example 9: Document store with nested CRDTs (Map)
/// JSON-like document with various field types
fn example_document_store() {
    println!("\n=== Example 9: Document Store (CRDT Map) ===");

    let mut doc_server1: CRDTMap<String> = CRDTMap::new();
    let mut doc_server2: CRDTMap<String> = CRDTMap::new();

    // Server 1 updates user document
    doc_server1.put("server1", "name".to_string(), MapValue::Text("Alice".to_string()));
    doc_server1.put("server1", "age".to_string(), MapValue::Int(30));
    doc_server1.put("server1", "active".to_string(), MapValue::Int(1));

    println!("Server 1 document:");
    println!("  name: {:?}", doc_server1.get(&"name".to_string()));
    println!("  age: {:?}", doc_server1.get(&"age".to_string()));

    // Server 2 updates same document (different fields)
    doc_server2.put("server2", "email".to_string(), MapValue::Text("alice@example.com".to_string()));
    doc_server2.put("server2", "age".to_string(), MapValue::Int(31)); // Concurrent age update

    println!("\nServer 2 document:");
    println!("  email: {:?}", doc_server2.get(&"email".to_string()));
    println!("  age: {:?}", doc_server2.get(&"age".to_string()));

    // Merge documents
    let merged = doc_server1.join(&doc_server2);
    println!("\nMerged document:");
    for key in merged.keys() {
        println!("  {}: {:?}", key, merged.get(key));
    }

    // Concurrent writes to same key results in multiple values
    let age_values = merged.get_all(&"age".to_string());
    println!("\nConcurrent 'age' values: {} version(s)", age_values.len());
}
