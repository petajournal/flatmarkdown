//! Blank lines of an item body, written into a page as lines of one backslash (ADR 070 of
//! PetaJournal). A blank line between two blocks would end the item in a page. A line of one
//! backslash is no blank line, so the item keeps it.

use crate::ast::options;
use comrak::{Arena, parse_document};

/// The line that stands for a blank line in a page.
const BLANK: &str = "\\";

/// `body` as it is written into a page: each blank line between its blocks, or before its
/// first block, is a line of one backslash.
pub fn body_to_file(body: &str) -> String {
    let kinds = line_kinds(body);
    body.split('\n')
        .zip(kinds)
        .map(|(line, kind)| match kind {
            LineKind::Boundary => BLANK,
            LineKind::Other => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `body` as it is read from a page: each line of one backslash is a blank line again.
pub fn body_from_file(body: &str) -> String {
    body.split('\n')
        .map(|line| if line == BLANK { "" } else { line })
        .collect::<Vec<_>>()
        .join("\n")
}

/// What a line of a body is to the blocks of the body.
#[derive(Clone, Copy, PartialEq)]
enum LineKind {
    /// A blank line outside every block, with a block after it: in a page it ends the item.
    Boundary,
    Other,
}

/// The kind of each line of `body`, read as `parse_page` reads the body in a page.
fn line_kinds(body: &str) -> Vec<LineKind> {
    let lines: Vec<&str> = body.split('\n').collect();
    let arena = Arena::new();
    let root = parse_document(&arena, &format!("{body}\n"), &options(false));
    // The lines of each block, 0-based and inclusive
    let blocks: Vec<(usize, usize)> = root
        .children()
        .map(|node| {
            let pos = node.data.borrow().sourcepos;
            (
                pos.start.line.saturating_sub(1),
                pos.end.line.saturating_sub(1),
            )
        })
        .collect();
    (0..lines.len())
        .map(|index| {
            let blank = lines[index].trim().is_empty();
            let in_block = blocks
                .iter()
                .any(|&(start, end)| (start..=end).contains(&index));
            let block_after = blocks.iter().any(|&(start, _)| start > index);
            if blank && !in_block && block_after {
                LineKind::Boundary
            } else {
                LineKind::Other
            }
        })
        .collect()
}
