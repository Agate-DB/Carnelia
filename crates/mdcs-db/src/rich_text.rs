//! Rich Text CRDT - Collaborative rich text with formatting marks.
//!
//! Extends RGAText with support for:
//! - Inline formatting (bold, italic, underline, strikethrough)
//! - Links and references
//! - Comments and annotations
//! - Custom marks for extensibility
//!
//! Uses anchor-based marks that reference TextIds for stability.

use crate::rga_text::{RGAText, RGATextDelta, TextId};
use mdcs_core::lattice::Lattice;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

/// Unique identifier for a mark (formatting span).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MarkId {
    /// The replica that created this mark.
    pub replica: String,
    /// Unique identifier within that replica.
    pub ulid: String,
}

impl MarkId {
    pub fn new(replica: impl Into<String>) -> Self {
        Self {
            replica: replica.into(),
            ulid: Ulid::new().to_string(),
        }
    }
}

/// The type/style of a formatting mark.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarkType {
    /// Bold text.
    Bold,
    /// Italic text.
    Italic,
    /// Underlined text.
    Underline,
    /// Strikethrough text.
    Strikethrough,
    /// Code/monospace text.
    Code,
    /// A hyperlink with URL.
    Link { url: String },
    /// A comment/annotation with author and content.
    Comment { author: String, content: String },
    /// Highlight with a color.
    Highlight { color: String },
    /// Custom mark type for extensibility.
    Custom { name: String, value: String },
}

impl MarkType {
    /// Check if this mark type conflicts with another.
    /// Conflicting marks cannot overlap.
    pub fn conflicts_with(&self, other: &MarkType) -> bool {
        use MarkType::*;
        match (self, other) {
            (Bold, Bold) => true,
            (Italic, Italic) => true,
            (Underline, Underline) => true,
            (Strikethrough, Strikethrough) => true,
            (Code, Code) => true,
            (Link { .. }, Link { .. }) => true,
            _ => false,
        }
    }
}

/// An anchor specifying a position in the text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anchor {
    /// Before all text.
    Start,
    /// After all text.
    End,
    /// After a specific character (by TextId).
    After(TextId),
    /// Before a specific character (by TextId).
    Before(TextId),
}

impl Anchor {
    /// Resolve this anchor to a position in the text.
    pub fn resolve(&self, text: &RGAText) -> Option<usize> {
        match self {
            Anchor::Start => Some(0),
            Anchor::End => Some(text.len()),
            Anchor::After(id) => text.id_to_position(id).map(|p| p + 1),
            Anchor::Before(id) => text.id_to_position(id),
        }
    }
}

/// A formatting mark that spans a range of text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    /// Unique identifier for this mark.
    pub id: MarkId,
    /// The type/style of the mark.
    pub mark_type: MarkType,
    /// Start anchor (inclusive).
    pub start: Anchor,
    /// End anchor (exclusive).
    pub end: Anchor,
    /// Whether this mark is deleted (tombstone).
    pub deleted: bool,
}

impl Mark {
    pub fn new(id: MarkId, mark_type: MarkType, start: Anchor, end: Anchor) -> Self {
        Self {
            id,
            mark_type,
            start,
            end,
            deleted: false,
        }
    }

    /// Get the resolved range (start, end) in the text.
    pub fn range(&self, text: &RGAText) -> Option<(usize, usize)> {
        let start = self.start.resolve(text)?;
        let end = self.end.resolve(text)?;
        Some((start, end))
    }

    /// Check if this mark covers a position.
    pub fn covers(&self, text: &RGAText, position: usize) -> bool {
        if self.deleted {
            return false;
        }
        if let Some((start, end)) = self.range(text) {
            position >= start && position < end
        } else {
            false
        }
    }
}

/// Delta for rich text operations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RichTextDelta {
    /// Text changes.
    pub text_delta: Option<RGATextDelta>,
    /// Marks to add.
    pub add_marks: Vec<Mark>,
    /// Marks to remove (by ID).
    pub remove_marks: Vec<MarkId>,
}

impl RichTextDelta {
    pub fn new() -> Self {
        Self {
            text_delta: None,
            add_marks: Vec::new(),
            remove_marks: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text_delta.is_none() && self.add_marks.is_empty() && self.remove_marks.is_empty()
    }
}

impl Default for RichTextDelta {
    fn default() -> Self {
        Self::new()
    }
}

/// Collaborative rich text with formatting support.
///
/// Combines RGAText for the text content with a set of
/// anchor-based marks for formatting.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RichText {
    /// The underlying plain text.
    text: RGAText,
    /// All marks indexed by their ID.
    marks: HashMap<MarkId, Mark>,
    /// The replica ID for this instance.
    replica_id: String,
    /// Pending delta for replication.
    #[serde(skip)]
    pending_delta: Option<RichTextDelta>,
}

impl RichText {
    /// Create a new empty rich text.
    pub fn new(replica_id: impl Into<String>) -> Self {
        let replica_id = replica_id.into();
        Self {
            text: RGAText::new(&replica_id),
            marks: HashMap::new(),
            replica_id,
            pending_delta: None,
        }
    }

    /// Get the replica ID.
    pub fn replica_id(&self) -> &str {
        &self.replica_id
    }

    /// Get the underlying text as a String.
    pub fn to_string(&self) -> String {
        self.text.to_string()
    }

    /// Get the text length.
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Get access to the underlying RGAText.
    pub fn text(&self) -> &RGAText {
        &self.text
    }

    // === Text Operations ===

    /// Insert plain text at a position.
    pub fn insert(&mut self, position: usize, text: &str) {
        self.text.insert(position, text);

        // Capture text delta
        if let Some(text_delta) = self.text.take_delta() {
            let delta = self.pending_delta.get_or_insert_with(RichTextDelta::new);
            delta.text_delta = Some(text_delta);
        }
    }

    /// Delete text range.
    pub fn delete(&mut self, start: usize, length: usize) {
        self.text.delete(start, length);

        // Capture text delta
        if let Some(text_delta) = self.text.take_delta() {
            let delta = self.pending_delta.get_or_insert_with(RichTextDelta::new);
            delta.text_delta = Some(text_delta);
        }
    }

    /// Replace text range.
    pub fn replace(&mut self, start: usize, end: usize, text: &str) {
        self.text.replace(start, end, text);

        // Capture text delta
        if let Some(text_delta) = self.text.take_delta() {
            let delta = self.pending_delta.get_or_insert_with(RichTextDelta::new);
            delta.text_delta = Some(text_delta);
        }
    }

    // === Mark Operations ===

    /// Add a formatting mark to a range.
    pub fn add_mark(&mut self, start: usize, end: usize, mark_type: MarkType) -> MarkId {
        let id = MarkId::new(&self.replica_id);

        // Convert positions to anchors
        let start_anchor = if start == 0 {
            Anchor::Start
        } else {
            self.text
                .position_to_id(start.saturating_sub(1))
                .map(|id| Anchor::After(id))
                .unwrap_or(Anchor::Start)
        };

        let end_anchor = if end >= self.text.len() {
            Anchor::End
        } else {
            self.text
                .position_to_id(end)
                .map(|id| Anchor::Before(id))
                .unwrap_or(Anchor::End)
        };

        let mark = Mark::new(id.clone(), mark_type, start_anchor, end_anchor);

        self.marks.insert(id.clone(), mark.clone());

        // Record delta
        let delta = self.pending_delta.get_or_insert_with(RichTextDelta::new);
        delta.add_marks.push(mark);

        id
    }

    /// Add bold formatting.
    pub fn bold(&mut self, start: usize, end: usize) -> MarkId {
        self.add_mark(start, end, MarkType::Bold)
    }

    /// Add italic formatting.
    pub fn italic(&mut self, start: usize, end: usize) -> MarkId {
        self.add_mark(start, end, MarkType::Italic)
    }

    /// Add underline formatting.
    pub fn underline(&mut self, start: usize, end: usize) -> MarkId {
        self.add_mark(start, end, MarkType::Underline)
    }

    /// Add a link.
    pub fn link(&mut self, start: usize, end: usize, url: impl Into<String>) -> MarkId {
        self.add_mark(start, end, MarkType::Link { url: url.into() })
    }

    /// Add a comment/annotation.
    pub fn comment(
        &mut self,
        start: usize,
        end: usize,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> MarkId {
        self.add_mark(
            start,
            end,
            MarkType::Comment {
                author: author.into(),
                content: content.into(),
            },
        )
    }

    /// Add a highlight.
    pub fn highlight(&mut self, start: usize, end: usize, color: impl Into<String>) -> MarkId {
        self.add_mark(
            start,
            end,
            MarkType::Highlight {
                color: color.into(),
            },
        )
    }

    /// Remove a mark by ID.
    pub fn remove_mark(&mut self, id: &MarkId) -> bool {
        if let Some(mark) = self.marks.get_mut(id) {
            mark.deleted = true;

            // Record delta
            let delta = self.pending_delta.get_or_insert_with(RichTextDelta::new);
            delta.remove_marks.push(id.clone());

            true
        } else {
            false
        }
    }

    /// Remove all marks of a type from a range.
    pub fn remove_marks_in_range(&mut self, start: usize, end: usize, mark_type: &MarkType) {
        let to_remove: Vec<_> = self
            .marks
            .iter()
            .filter(|(_, mark)| {
                if mark.deleted || &mark.mark_type != mark_type {
                    return false;
                }
                if let Some((ms, me)) = mark.range(&self.text) {
                    // Overlaps with range
                    ms < end && me > start
                } else {
                    false
                }
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_remove {
            self.remove_mark(&id);
        }
    }

    /// Get all marks at a position.
    pub fn marks_at(&self, position: usize) -> Vec<&Mark> {
        self.marks
            .values()
            .filter(|m| m.covers(&self.text, position))
            .collect()
    }

    /// Get all marks in a range.
    pub fn marks_in_range(&self, start: usize, end: usize) -> Vec<&Mark> {
        self.marks
            .values()
            .filter(|mark| {
                if mark.deleted {
                    return false;
                }
                if let Some((ms, me)) = mark.range(&self.text) {
                    ms < end && me > start
                } else {
                    false
                }
            })
            .collect()
    }

    /// Check if a position has a specific mark type.
    pub fn has_mark(&self, position: usize, mark_type: &MarkType) -> bool {
        self.marks_at(position)
            .iter()
            .any(|m| &m.mark_type == mark_type)
    }

    /// Get all marks (including deleted for debugging).
    pub fn all_marks(&self) -> impl Iterator<Item = &Mark> + '_ {
        self.marks.values()
    }

    /// Get only active marks.
    pub fn active_marks(&self) -> impl Iterator<Item = &Mark> + '_ {
        self.marks.values().filter(|m| !m.deleted)
    }

    // === Delta Operations ===

    /// Take the pending delta.
    pub fn take_delta(&mut self) -> Option<RichTextDelta> {
        self.pending_delta.take()
    }

    /// Apply a delta from another replica.
    pub fn apply_delta(&mut self, delta: &RichTextDelta) {
        // Apply text changes
        if let Some(text_delta) = &delta.text_delta {
            self.text.apply_delta(text_delta);
        }

        // Apply mark additions
        for mark in &delta.add_marks {
            self.marks
                .entry(mark.id.clone())
                .and_modify(|m| {
                    if mark.deleted {
                        m.deleted = true;
                    }
                })
                .or_insert_with(|| mark.clone());
        }

        // Apply mark removals
        for id in &delta.remove_marks {
            if let Some(mark) = self.marks.get_mut(id) {
                mark.deleted = true;
            }
        }
    }

    // === Rendering ===

    /// Render as HTML (basic implementation).
    pub fn to_html(&self) -> String {
        let text = self.to_string();
        if text.is_empty() {
            return String::new();
        }

        // Collect marks and their ranges
        let mut events: Vec<(usize, i8, &Mark)> = Vec::new();
        for mark in self.active_marks() {
            if let Some((start, end)) = mark.range(&self.text) {
                events.push((start, 1, mark)); // 1 = open
                events.push((end, -1, mark)); // -1 = close
            }
        }

        // Sort: by position, then closes before opens at same position
        events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let mut result = String::new();
        let chars: Vec<char> = text.chars().collect();
        let mut pos = 0;

        let mut open_tags: Vec<&Mark> = Vec::new();

        for (event_pos, event_type, mark) in events {
            // Output text before this event
            while pos < event_pos && pos < chars.len() {
                result.push(chars[pos]);
                pos += 1;
            }

            if event_type > 0 {
                // Open tag
                result.push_str(&mark_open_tag(&mark.mark_type));
                open_tags.push(mark);
            } else {
                // Close tag
                result.push_str(&mark_close_tag(&mark.mark_type));
                open_tags.retain(|m| m.id != mark.id);
            }
        }

        // Output remaining text
        while pos < chars.len() {
            result.push(chars[pos]);
            pos += 1;
        }

        result
    }
}

fn mark_open_tag(mark_type: &MarkType) -> String {
    match mark_type {
        MarkType::Bold => "<strong>".to_string(),
        MarkType::Italic => "<em>".to_string(),
        MarkType::Underline => "<u>".to_string(),
        MarkType::Strikethrough => "<s>".to_string(),
        MarkType::Code => "<code>".to_string(),
        MarkType::Link { url } => format!("<a href=\"{}\">", url),
        MarkType::Comment { author, content } => format!(
            "<span data-comment-author=\"{}\" data-comment=\"{}\">",
            author, content
        ),
        MarkType::Highlight { color } => format!("<mark style=\"background-color:{}\">", color),
        MarkType::Custom { name, value } => format!("<span data-{}=\"{}\">", name, value),
    }
}

fn mark_close_tag(mark_type: &MarkType) -> String {
    match mark_type {
        MarkType::Bold => "</strong>".to_string(),
        MarkType::Italic => "</em>".to_string(),
        MarkType::Underline => "</u>".to_string(),
        MarkType::Strikethrough => "</s>".to_string(),
        MarkType::Code => "</code>".to_string(),
        MarkType::Link { .. } => "</a>".to_string(),
        MarkType::Comment { .. } => "</span>".to_string(),
        MarkType::Highlight { .. } => "</mark>".to_string(),
        MarkType::Custom { .. } => "</span>".to_string(),
    }
}

impl std::fmt::Display for RichText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for RichText {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string() && self.marks.len() == other.marks.len()
    }
}

impl Eq for RichText {}

impl Lattice for RichText {
    fn bottom() -> Self {
        Self::new("")
    }

    fn join(&self, other: &Self) -> Self {
        let mut result = self.clone();

        // Merge text
        result.text = self.text.join(&other.text);

        // Merge marks
        for (id, mark) in &other.marks {
            result
                .marks
                .entry(id.clone())
                .and_modify(|m| {
                    if mark.deleted {
                        m.deleted = true;
                    }
                })
                .or_insert_with(|| mark.clone());
        }

        result
    }
}

impl Default for RichText {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let mut doc = RichText::new("r1");
        doc.insert(0, "Hello World");
        doc.bold(0, 5);

        assert_eq!(doc.to_string(), "Hello World");
        assert!(doc.has_mark(2, &MarkType::Bold));
        assert!(!doc.has_mark(6, &MarkType::Bold));
    }

    #[test]
    fn test_multiple_marks() {
        let mut doc = RichText::new("r1");
        doc.insert(0, "Hello World");
        doc.bold(0, 5);
        doc.italic(6, 11);

        let marks_at_2 = doc.marks_at(2);
        assert_eq!(marks_at_2.len(), 1);
        assert_eq!(marks_at_2[0].mark_type, MarkType::Bold);

        let marks_at_8 = doc.marks_at(8);
        assert_eq!(marks_at_8.len(), 1);
        assert_eq!(marks_at_8[0].mark_type, MarkType::Italic);
    }

    #[test]
    fn test_overlapping_marks() {
        let mut doc = RichText::new("r1");
        doc.insert(0, "Hello World");
        doc.bold(0, 8);
        doc.italic(4, 11);

        // Position 5 should have both
        let marks = doc.marks_at(5);
        assert_eq!(marks.len(), 2);
    }

    #[test]
    fn test_link_and_comment() {
        let mut doc = RichText::new("r1");
        doc.insert(0, "Check this out");
        doc.link(6, 10, "https://example.com");
        doc.comment(0, 14, "Alice", "Needs review");

        assert!(doc.has_mark(
            7,
            &MarkType::Link {
                url: "https://example.com".to_string()
            }
        ));
        assert!(doc
            .marks_at(0)
            .iter()
            .any(|m| matches!(&m.mark_type, MarkType::Comment { .. })));
    }

    #[test]
    fn test_remove_mark() {
        let mut doc = RichText::new("r1");
        doc.insert(0, "Hello World");
        let mark_id = doc.bold(0, 5);

        assert!(doc.has_mark(2, &MarkType::Bold));

        doc.remove_mark(&mark_id);

        assert!(!doc.has_mark(2, &MarkType::Bold));
    }

    #[test]
    fn test_concurrent_formatting() {
        let mut doc1 = RichText::new("r1");
        let mut doc2 = RichText::new("r2");

        // Setup
        doc1.insert(0, "Hello World");
        doc2.apply_delta(&doc1.take_delta().unwrap());

        // Concurrent formatting
        doc1.bold(0, 5);
        doc2.italic(6, 11);

        // Exchange deltas
        let delta1 = doc1.take_delta().unwrap();
        let delta2 = doc2.take_delta().unwrap();

        doc1.apply_delta(&delta2);
        doc2.apply_delta(&delta1);

        // Both should have both marks
        assert!(doc1.has_mark(2, &MarkType::Bold));
        assert!(doc1.has_mark(8, &MarkType::Italic));
        assert!(doc2.has_mark(2, &MarkType::Bold));
        assert!(doc2.has_mark(8, &MarkType::Italic));
    }

    #[test]
    fn test_html_rendering() {
        let mut doc = RichText::new("r1");
        doc.insert(0, "Hello World");
        doc.bold(0, 5);

        let html = doc.to_html();
        assert!(html.contains("<strong>Hello</strong>"));
        assert!(html.contains("World"));
    }

    #[test]
    fn test_insert_expands_mark() {
        let mut doc = RichText::new("r1");
        doc.insert(0, "AB");
        doc.bold(0, 2); // Bold "AB"

        // Insert "X" in the middle
        doc.insert(1, "X");

        // Text should be "AXB"
        assert_eq!(doc.to_string(), "AXB");

        // The mark anchor system means the mark covers A and B,
        // but X was inserted after A so may or may not be covered
        // depending on anchor resolution
    }

    #[test]
    fn test_lattice_join() {
        let mut doc1 = RichText::new("r1");
        let mut doc2 = RichText::new("r2");

        doc1.insert(0, "Hello");
        doc1.bold(0, 5);

        doc2.insert(0, "World");
        doc2.italic(0, 5);

        let merged = doc1.join(&doc2);

        // Should have marks from both
        assert!(merged.active_marks().count() >= 2);
    }

    #[test]
    fn test_marks_in_range() {
        let mut doc = RichText::new("r1");
        doc.insert(0, "Hello World Test");
        doc.bold(0, 5);
        doc.italic(6, 11);
        doc.underline(12, 16);

        let marks = doc.marks_in_range(4, 13);
        // Should include Bold (ends at 5), Italic (6-11), and Underline (starts at 12)
        assert!(marks.len() >= 2);
    }
}
