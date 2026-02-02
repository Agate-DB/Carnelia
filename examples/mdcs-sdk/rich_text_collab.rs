//! Rich Text Collaboration Example
//!
//! This example demonstrates collaborative rich text editing
//! with formatting support (bold, italic, etc.).
//!
//! Run with: cargo run --example rich_text_collab

use mdcs_sdk::client::quick::create_collaborative_clients;
use mdcs_sdk::MarkType;

fn main() {
    println!("=== Rich Text Collaboration Example ===\n");

    // Create 2 connected clients
    let clients = create_collaborative_clients(&["Writer", "Editor"]);
    
    println!("Created clients: Writer and Editor\n");

    // Create sessions
    let writer_session = clients[0].create_session("document-editing");
    let editor_session = clients[1].create_session("document-editing");

    // Both open the same rich text document
    let writer_doc = writer_session.open_rich_text_doc("article.rtf");
    let editor_doc = editor_session.open_rich_text_doc("article.rtf");

    // Writer creates the initial content
    println!("Writer creates the initial draft...");
    {
        let mut doc = writer_doc.write();
        doc.insert(0, "Introduction to Collaborative Editing\n\n");
        let pos1 = doc.len();
        doc.insert(pos1, "Collaborative editing allows multiple users to work on ");
        let pos2 = doc.len();
        doc.insert(pos2, "the same document simultaneously. Changes are merged ");
        let pos3 = doc.len();
        doc.insert(pos3, "automatically using CRDT algorithms.");
    }
    
    println!("Writer's draft:");
    println!("{}\n", writer_doc.read().get_text());

    // Editor adds formatting
    println!("Editor adds formatting...");
    {
        let mut doc = editor_doc.write();
        
        // Make the title bold (positions 0-37)
        doc.format(0, 37, MarkType::Bold);
        println!("  - Made title bold (0-37)");
        
        // Italicize "Collaborative editing"
        doc.format(38, 60, MarkType::Italic);
        println!("  - Italicized 'Collaborative editing' (38-60)");
        
        // Underline "CRDT algorithms"
        doc.format(152, 167, MarkType::Underline);
        println!("  - Underlined 'CRDT algorithms' (152-167)");
    }

    println!("\n=== Document Stats ===");
    println!("Total length: {} characters", writer_doc.read().len());
    println!("Content: {}", writer_doc.read().get_text());

    // Demonstrate cursor tracking
    println!("\n=== Cursor Positions ===");
    
    // Writer is at the end
    writer_session.awareness().set_cursor("article.rtf", writer_doc.read().len());
    println!("Writer's cursor: position {}", writer_doc.read().len());
    
    // Editor selects the title
    editor_session.awareness().set_selection("article.rtf", 0, 37);
    println!("Editor's selection: 0-37 (title)");

    // Check cursors from writer's perspective
    let cursors = writer_session.awareness().get_cursors("article.rtf");
    println!("\nCursors visible to Writer:");
    for cursor in cursors {
        println!("  {} at position {}", cursor.user_name, cursor.position);
        if let Some(start) = cursor.selection_start {
            println!("    Selection: {}-{}", start, cursor.selection_end.unwrap_or(0));
        }
    }

    println!("\n=== Demo Complete ===");
}
