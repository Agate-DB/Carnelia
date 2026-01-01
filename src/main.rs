use mdcs_core::gset::GSet;
use mdcs_core::lattice::Lattice;
use mdcs_core::orset::ORSet;

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

    println!("\n✓ All examples completed successfully!");
}
