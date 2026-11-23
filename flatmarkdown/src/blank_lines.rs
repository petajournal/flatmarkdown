//! Blank lines of an item body, written into a page as lines of one backslash (ADR 070 of
//! PetaJournal). A blank line between two blocks would end the item in a page. A line of one
//! backslash is no blank line, so the item keeps it.

use crate::ast::options;
use comrak::nodes::AstNode;
use comrak::{Arena, parse_document};
use std::ops::RangeInclusive;

/// The line that stands for a blank line in a page.
const BLANK: &str = "\\";

/// `body` as it is written into a page: each blank line between its blocks, or before its
/// first block, is a line of one backslash. Every line of backslashes gets one more, so that it
/// is not read as a blank line.
///
/// A line of one backslash may change the blocks after it, as it does not end a paragraph.
/// [`body_from_file`] does not look at the blocks, so any body comes back as it was.
pub fn body_to_file(body: &str) -> String {
    let boundaries = boundaries(body);
    body.split('\n')
        .zip(boundaries)
        .map(|(line, boundary)| {
            if boundary {
                BLANK.to_string()
            } else if is_backslashes(line) {
                format!("{line}{BLANK}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `body` as it is read from a page: each line of backslashes loses one, so that a line of one
/// backslash is a blank line again.
pub fn body_from_file(body: &str) -> String {
    body.split('\n')
        .map(|line| {
            if is_backslashes(line) {
                &line[BLANK.len()..]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether `line` is one backslash or more, and nothing else.
fn is_backslashes(line: &str) -> bool {
    !line.is_empty() && line.chars().all(|c| c == '\\')
}

/// Whether each line of `body` is a blank line outside every block, with a block after it.
/// In a page, such a line ends the item. The blocks are read as `parse_page` reads them.
fn boundaries(body: &str) -> Vec<bool> {
    let arena = Arena::new();
    let root = parse_document(&arena, &format!("{body}\n"), &options(false));
    let blocks: Vec<RangeInclusive<usize>> = root.children().map(line_range).collect();
    body.split('\n')
        .enumerate()
        .map(|(index, line)| {
            line.trim().is_empty()
                && !blocks.iter().any(|block| block.contains(&index))
                && blocks.iter().any(|block| *block.start() > index)
        })
        .collect()
}

/// The lines of `node`, 0-based.
fn line_range<'a>(node: &'a AstNode<'a>) -> RangeInclusive<usize> {
    let pos = node.data.borrow().sourcepos;
    pos.start.line.saturating_sub(1)..=pos.end.line.saturating_sub(1)
}
