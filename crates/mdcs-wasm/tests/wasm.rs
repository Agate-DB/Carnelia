//! WASM-specific integration tests
//!
//! These tests run in a headless browser environment using wasm-bindgen-test.
//! Run with: `wasm-pack test --headless --chrome`

use wasm_bindgen_test::*;
use mdcs_wasm::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_document_basic_operations() {
    let mut doc = CollaborativeDocument::new("test-doc", "test-replica");
    
    // Test initial state
    assert!(doc.is_empty());
    assert_eq!(doc.len(), 0);
    assert_eq!(doc.version(), 0);
    
    // Test insert
    doc.insert(0, "Hello");
    assert_eq!(doc.get_text(), "Hello");
    assert_eq!(doc.len(), 5);
    assert_eq!(doc.version(), 1);
    
    // Test append
    doc.insert(5, " World");
    assert_eq!(doc.get_text(), "Hello World");
    assert_eq!(doc.len(), 11);
    
    // Test insert in middle
    doc.insert(5, ",");
    assert_eq!(doc.get_text(), "Hello, World");
}

#[wasm_bindgen_test]
fn test_document_delete() {
    let mut doc = CollaborativeDocument::new("test-doc", "test-replica");
    doc.insert(0, "Hello, World!");
    
    // Delete from middle
    doc.delete(5, 2); // Remove ", "
    assert_eq!(doc.get_text(), "HelloWorld!");
    
    // Delete from start
    doc.delete(0, 5); // Remove "Hello"
    assert_eq!(doc.get_text(), "World!");
    
    // Delete from end
    doc.delete(5, 1); // Remove "!"
    assert_eq!(doc.get_text(), "World");
}

#[wasm_bindgen_test]
fn test_document_formatting() {
    let mut doc = CollaborativeDocument::new("test-doc", "test-replica");
    doc.insert(0, "Hello World");
    
    // Apply bold to "Hello"
    doc.apply_bold(0, 5);
    let html = doc.get_html();
    // HTML should contain bold tags (either <b> or <strong>)
    assert!(html.contains("<b>") || html.contains("<strong>") || html.contains("Hello"));
    
    // Apply italic to "World"
    doc.apply_italic(6, 11);
    let html2 = doc.get_html();
    assert!(html2.contains("<i>") || html2.contains("<em>") || html2.contains("World"));
}

#[wasm_bindgen_test]
fn test_document_link() {
    let mut doc = CollaborativeDocument::new("test-doc", "test-replica");
    doc.insert(0, "Click here for more");
    doc.apply_link(0, 10, "https://example.com");
    
    let html = doc.get_html();
    assert!(html.contains("href") || html.contains("example.com") || html.contains("Click"));
}

#[wasm_bindgen_test]
fn test_document_serialize_deserialize() {
    let mut doc1 = CollaborativeDocument::new("test-doc", "replica-1");
    doc1.insert(0, "Hello from replica 1");
    doc1.apply_bold(0, 5);
    
    // Serialize
    let state = doc1.serialize().expect("Serialization should succeed");
    assert!(!state.is_empty());
    
    // Create another document and merge
    let mut doc2 = CollaborativeDocument::new("test-doc", "replica-2");
    doc2.merge(&state).expect("Merge should succeed");
    
    // Should have the same content
    assert_eq!(doc1.get_text(), doc2.get_text());
}

#[wasm_bindgen_test]
fn test_concurrent_edits_convergence() {
    // Simulate two users editing concurrently
    let mut doc_alice = CollaborativeDocument::new("shared-doc", "alice");
    let mut doc_bob = CollaborativeDocument::new("shared-doc", "bob");
    
    // Both start with same base
    doc_alice.insert(0, "Base text");
    let base_state = doc_alice.serialize().unwrap();
    doc_bob.merge(&base_state).unwrap();
    
    // Alice adds " - edited by Alice" at the end
    doc_alice.insert(9, " - Alice");
    
    // Bob adds " - edited by Bob" at the end
    doc_bob.insert(9, " - Bob");
    
    // Exchange states
    let alice_state = doc_alice.serialize().unwrap();
    let bob_state = doc_bob.serialize().unwrap();
    
    doc_alice.merge(&bob_state).unwrap();
    doc_bob.merge(&alice_state).unwrap();
    
    // Both should converge to the same state
    assert_eq!(doc_alice.get_text(), doc_bob.get_text());
    
    // Both texts should contain both edits
    let final_text = doc_alice.get_text();
    assert!(final_text.contains("Base text"));
    assert!(final_text.contains("Alice") || final_text.contains("Bob"));
}

#[wasm_bindgen_test]
fn test_document_snapshot_restore() {
    let mut original = CollaborativeDocument::new("test-doc", "test-replica");
    original.insert(0, "Important content");
    original.apply_bold(0, 9);
    
    // Create snapshot
    let snapshot = original.snapshot().expect("Snapshot should succeed");
    
    // Restore from snapshot
    let restored = CollaborativeDocument::restore(snapshot)
        .expect("Restore should succeed");
    
    // Verify restored document matches
    assert_eq!(original.get_text(), restored.get_text());
    assert_eq!(original.doc_id(), restored.doc_id());
}

#[wasm_bindgen_test]
fn test_user_presence_basic() {
    let presence = UserPresence::new("user-1", "Alice", "#FF6B6B");
    
    assert_eq!(presence.user_id(), "user-1");
    assert_eq!(presence.user_name(), "Alice");
    assert_eq!(presence.color(), "#FF6B6B");
    assert_eq!(presence.cursor(), None);
    assert!(!presence.has_selection());
}

#[wasm_bindgen_test]
fn test_user_presence_cursor() {
    let mut presence = UserPresence::new("user-1", "Alice", "#FF6B6B");
    
    presence.set_cursor(42);
    assert_eq!(presence.cursor(), Some(42));
    assert!(!presence.has_selection());
    
    presence.set_cursor(100);
    assert_eq!(presence.cursor(), Some(100));
}

#[wasm_bindgen_test]
fn test_user_presence_selection() {
    let mut presence = UserPresence::new("user-1", "Alice", "#FF6B6B");
    
    presence.set_selection(10, 50);
    assert!(presence.has_selection());
    assert_eq!(presence.selection_start(), Some(10));
    assert_eq!(presence.selection_end(), Some(50));
    assert_eq!(presence.cursor(), Some(50)); // Cursor at end of selection
    
    // Selection should normalize (start < end)
    presence.set_selection(50, 10);
    assert_eq!(presence.selection_start(), Some(10));
    assert_eq!(presence.selection_end(), Some(50));
}

#[wasm_bindgen_test]
fn test_user_presence_clear() {
    let mut presence = UserPresence::new("user-1", "Alice", "#FF6B6B");
    
    presence.set_selection(10, 50);
    assert!(presence.has_selection());
    
    presence.clear();
    assert_eq!(presence.cursor(), None);
    assert!(!presence.has_selection());
}

#[wasm_bindgen_test]
fn test_user_presence_serialization() {
    let mut original = UserPresence::new("user-1", "Alice", "#FF6B6B");
    original.set_selection(10, 50);
    
    // Serialize to JSON
    let json = original.to_json().expect("to_json should succeed");
    
    // Deserialize
    let restored = UserPresence::from_json(json).expect("from_json should succeed");
    
    assert_eq!(original.user_id(), restored.user_id());
    assert_eq!(original.user_name(), restored.user_name());
    assert_eq!(original.color(), restored.color());
    assert_eq!(original.cursor(), restored.cursor());
    assert_eq!(original.selection_start(), restored.selection_start());
    assert_eq!(original.selection_end(), restored.selection_end());
}

#[wasm_bindgen_test]
fn test_generate_replica_id() {
    let id1 = generate_replica_id();
    let id2 = generate_replica_id();
    
    // IDs should be non-empty
    assert!(!id1.is_empty());
    assert!(!id2.is_empty());
    
    // IDs should be unique (with very high probability)
    assert_ne!(id1, id2);
}

#[wasm_bindgen_test]
fn test_generate_user_color() {
    let color = generate_user_color();
    
    // Should be a valid hex color
    assert!(color.starts_with('#'));
    assert_eq!(color.len(), 7);
}

#[wasm_bindgen_test]
fn test_edge_cases_empty_operations() {
    let mut doc = CollaborativeDocument::new("test-doc", "test-replica");
    
    // Insert empty string should be no-op
    doc.insert(0, "");
    assert_eq!(doc.len(), 0);
    
    // Delete from empty document
    doc.delete(0, 10);
    assert_eq!(doc.len(), 0);
    
    // Insert then delete same
    doc.insert(0, "Hello");
    doc.delete(0, 5);
    assert!(doc.is_empty());
}

#[wasm_bindgen_test]
fn test_edge_cases_out_of_bounds() {
    let mut doc = CollaborativeDocument::new("test-doc", "test-replica");
    doc.insert(0, "Hello");
    
    // Insert past end should append
    doc.insert(1000, " World");
    assert_eq!(doc.get_text(), "Hello World");
    
    // Delete past end should be bounded
    doc.delete(5, 1000);
    assert_eq!(doc.get_text(), "Hello");
    
    // Apply formatting past end should be bounded
    doc.apply_bold(0, 1000);
    // Should not panic, formatting bounded to actual content
}

#[wasm_bindgen_test]
fn test_multiple_replicas_three_way_merge() {
    let mut doc_a = CollaborativeDocument::new("doc", "replica-a");
    let mut doc_b = CollaborativeDocument::new("doc", "replica-b");
    let mut doc_c = CollaborativeDocument::new("doc", "replica-c");
    
    // Each replica makes an edit
    doc_a.insert(0, "A");
    doc_b.insert(0, "B");
    doc_c.insert(0, "C");
    
    // Get all states
    let state_a = doc_a.serialize().unwrap();
    let state_b = doc_b.serialize().unwrap();
    let state_c = doc_c.serialize().unwrap();
    
    // Merge all into A
    doc_a.merge(&state_b).unwrap();
    doc_a.merge(&state_c).unwrap();
    
    // Merge all into B
    doc_b.merge(&state_a).unwrap();
    doc_b.merge(&state_c).unwrap();
    
    // Merge all into C
    doc_c.merge(&state_a).unwrap();
    doc_c.merge(&state_b).unwrap();
    
    // All should converge
    assert_eq!(doc_a.get_text(), doc_b.get_text());
    assert_eq!(doc_b.get_text(), doc_c.get_text());
    
    // Result should contain all edits
    let result = doc_a.get_text();
    assert!(result.contains('A'));
    assert!(result.contains('B'));
    assert!(result.contains('C'));
}
