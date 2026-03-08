use mdcs_sdk::client::quick::create_collaborative_clients;

fn main() {
    // Create two connected collaborative clients
    let clients = create_collaborative_clients(&["Alice", "Bob"]);
    let client_a = &clients[0];
    let client_b = &clients[1];

    // Create sessions
    let session_a = client_a.create_session("room-1");
    let session_b = client_b.create_session("room-1");

    // Open text documents
    let doc_a = session_a.open_text_doc("shared-doc");
    let doc_b = session_b.open_text_doc("shared-doc");

    // User A types
    doc_a.write().insert(0, "Hello ");

    // User B types (concurrent edit)
    doc_b.write().insert(0, "World!");

    // Merge states to demonstrate convergence
    let state_a = doc_a.read().clone_state();
    let state_b = doc_b.read().clone_state();

    doc_a.write().merge(&state_b);
    doc_b.write().merge(&state_a);
}
