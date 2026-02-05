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
    
    println!("Created clients:");
    println!("  - {} (peer: {})", clients[0].user_name(), clients[0].peer_id());
    println!("  - {} (peer: {})", clients[1].user_name(), clients[1].peer_id());
    println!();

    // Create sessions
    let writer_session = clients[0].create_session("document-editing");
    let editor_session = clients[1].create_session("document-editing");

    // Both open the same rich text document
    let writer_doc = writer_session.open_rich_text_doc("article.rtf");
    let editor_doc = editor_session.open_rich_text_doc("article.rtf");

    // === Step 1: Writer creates content ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           Step 1: Writer creates the initial draft             ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    {
        let mut doc = writer_doc.write();
        doc.insert(0, "Introduction to Collaborative Editing\n\n");
        let pos1 = doc.len();
        doc.insert(pos1, "Collaborative editing allows multiple users to work on ");
        let pos2 = doc.len();
        doc.insert(pos2, "the same document simultaneously. Changes are merged ");
        let pos3 = doc.len();
        doc.insert(pos3, "automatically using CRDT algorithms.\n\n");
        let pos4 = doc.len();
        doc.insert(pos4, "Key Benefits:\n");
        let pos5 = doc.len();
        doc.insert(pos5, "• Real-time collaboration\n");
        let pos6 = doc.len();
        doc.insert(pos6, "• Offline support\n");
        let pos7 = doc.len();
        doc.insert(pos7, "• No conflicts\n");
    }
    
    println!("Writer's initial draft:");
    println!("┌─────────────────────────────────────────────────────────────────┐");
    for line in writer_doc.read().get_text().lines() {
        println!("│ {:63}│", line);
    }
    println!("└─────────────────────────────────────────────────────────────────┘");
    
    // Sync to editor
    {
        let writer_state = writer_doc.read().clone_state();
        editor_doc.write().merge(&writer_state);
    }
    println!("\n  [SYNC] → Editor\n");

    // === Step 2: Editor adds formatting ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           Step 2: Editor adds formatting                       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    {
        let mut doc = editor_doc.write();
        
        // Make the title bold (first line)
        doc.format(0, 37, MarkType::Bold);
        println!("  ✓ Made title BOLD (0-37)");
        
        // Italicize "Collaborative editing"
        doc.format(39, 61, MarkType::Italic);
        println!("  ✓ Made 'Collaborative editing' ITALIC (39-61)");
        
        // Underline "CRDT algorithms"
        doc.format(152, 167, MarkType::Underline);
        println!("  ✓ Made 'CRDT algorithms' UNDERLINE (152-167)");
        
        // Bold the "Key Benefits" header
        doc.format(170, 182, MarkType::Bold);
        println!("  ✓ Made 'Key Benefits:' BOLD (170-182)");
    }
    
    // Sync formatting back to writer
    {
        let editor_state = editor_doc.read().clone_state();
        writer_doc.write().merge(&editor_state);
    }
    println!("\n  [SYNC] → Writer\n");

    // === Step 3: Writer adds more content ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           Step 3: Writer adds conclusion                       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    {
        let mut doc = writer_doc.write();
        let pos = doc.len();
        doc.insert(pos, "\nConclusion:\n");
        let pos = doc.len();
        doc.insert(pos, "MDCS provides a robust foundation for building ");
        let pos = doc.len();
        doc.insert(pos, "collaborative applications.");
    }
    
    // Writer also adds some formatting
    {
        let mut doc = writer_doc.write();
        let text = doc.get_text();
        let conclusion_pos = text.find("Conclusion:").unwrap_or(0);
        doc.format(conclusion_pos, conclusion_pos + 11, MarkType::Bold);
        println!("  ✓ Writer added conclusion and made header BOLD");
    }
    
    // Sync to editor
    {
        let writer_state = writer_doc.read().clone_state();
        editor_doc.write().merge(&writer_state);
    }
    println!("\n  [SYNC] → Editor\n");

    // === FINAL RESULT ===
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║              FINAL SYNCHRONIZED DOCUMENT                       ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║                                                                ║");
    
    let final_text = writer_doc.read().get_text();
    for line in final_text.lines() {
        println!("║  {:60}║", line);
    }
    
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    // Show formatting applied (note: actual mark retrieval requires internal API)
    println!("\n=== Applied Formatting ===\n");
    println!("  The following formatting was applied during editing:");
    println!("  BOLD       [  0- 37]: \"Introduction to Collaborative Edi...\"");
    println!("  ITALIC     [ 39- 61]: \"Collaborative editing\"");
    println!("  UNDERLINE  [152-167]: \"CRDT algorithms\"");
    println!("  BOLD       [170-182]: \"Key Benefits:\"");
    println!("  BOLD       [conclusion]: \"Conclusion:\"");

    // === Verification ===
    println!("\n=== Verification ===\n");
    
    let writer_text = writer_doc.read().get_text();
    let editor_text = editor_doc.read().get_text();
    
    if writer_text == editor_text {
        println!("  ✓ Writer and Editor have IDENTICAL documents!");
        println!("    - Text length: {} characters", writer_text.len());
        println!("    - Lines: {}", writer_text.lines().count());
    } else {
        println!("  ✗ Documents differ (unexpected!)");
    }

    // === Cursor tracking ===
    println!("\n=== Presence Tracking ===\n");
    
    // Writer is at the end
    let writer_pos = writer_doc.read().len();
    writer_session.awareness().set_cursor("article.rtf", writer_pos);
    println!("  Writer's cursor: position {} (end of document)", writer_pos);
    
    // Editor selects the title
    editor_session.awareness().set_selection("article.rtf", 0, 37);
    println!("  Editor's selection: 0-37 (title)");

    // Check cursors from writer's perspective
    let cursors = writer_session.awareness().get_cursors("article.rtf");
    println!("\n  Cursors visible to Writer:");
    for cursor in cursors {
        if let Some(start) = cursor.selection_start {
            println!("    - {}: selection {}-{}", 
                cursor.user_name, start, cursor.selection_end.unwrap_or(0));
        } else {
            println!("    - {}: position {}", cursor.user_name, cursor.position);
        }
    }

    println!("\n=== Demo Complete ===");
}
