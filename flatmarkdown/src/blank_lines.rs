//! Blank lines of an item body, written into a page as lines of one backslash (ADR 070 of
//! PetaJournal). A blank line between two blocks would end the item in a page. A line of one
//! backslash is no blank line, so the item keeps it.

use crate::ast::options;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, parse_document};
use std::ops::RangeInclusive;

/// The line that stands for a blank line in a page.
const BLANK: &str = "\\";

/// `body` as it is written into a page: each blank line between its blocks, or before its
/// first block, is a line of one backslash. A line of backslashes gets one more, so that it is
/// not read as a blank line. The text of code and HTML blocks is shown as it is, so their
/// lines are kept.
pub fn body_to_file(body: &str) -> String {
    let kinds = line_kinds(body);
    body.split('\n')
        .zip(kinds)
        .map(|(line, kind)| match kind {
            LineKind::Boundary => BLANK.to_string(),
            LineKind::Text if is_backslashes(line) => format!("{line}{BLANK}"),
            LineKind::Text | LineKind::Literal => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `body` as it is read from a page: each line of backslashes outside code and HTML blocks
/// loses one, so that a line of one backslash is a blank line again.
pub fn body_from_file(body: &str) -> String {
    let kinds = line_kinds(body);
    body.split('\n')
        .zip(kinds)
        .map(|(line, kind)| {
            if kind == LineKind::Text && is_backslashes(line) {
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

/// What a line of a body is to the blocks of the body.
#[derive(Clone, Copy, PartialEq)]
enum LineKind {
    /// A blank line outside every block, with a block after it: in a page it ends the item.
    Boundary,
    /// A line of a code or HTML block, whose text is shown as it is.
    Literal,
    Text,
}

/// The kind of each line of `body`, read as `parse_page` reads the body in a page.
fn line_kinds(body: &str) -> Vec<LineKind> {
    let lines: Vec<&str> = body.split('\n').collect();
    let arena = Arena::new();
    let root = parse_document(&arena, &format!("{body}\n"), &options(false));
    let blocks: Vec<RangeInclusive<usize>> = root.children().map(line_range).collect();
    // A blank line ends these blocks, and a line of one backslash would not
    let ended_by_blank: Vec<RangeInclusive<usize>> = root
        .children()
        .filter(|node| {
            matches!(&node.data.borrow().value,
                NodeValue::HtmlBlock(html) if matches!(html.block_type, 6 | 7))
        })
        .map(line_range)
        .collect();
    let literals: Vec<RangeInclusive<usize>> = root
        .descendants()
        .filter(|node| {
            matches!(
                node.data.borrow().value,
                NodeValue::CodeBlock(_) | NodeValue::HtmlBlock(_)
            )
        })
        .map(line_range)
        .collect();
    (0..lines.len())
        .map(|index| {
            let blank = lines[index].trim().is_empty();
            let in_block = blocks.iter().any(|block| block.contains(&index));
            let block_after = blocks.iter().any(|block| *block.start() > index);
            let block_before = blocks.iter().rev().find(|block| *block.end() < index);
            let after_html_ended_by_blank =
                block_before.is_some_and(|block| ended_by_blank.contains(block));
            if literals.iter().any(|literal| literal.contains(&index)) {
                LineKind::Literal
            } else if blank && !in_block && block_after && !after_html_ended_by_blank {
                LineKind::Boundary
            } else {
                LineKind::Text
            }
        })
        .collect()
}

/// The lines of `node`, 0-based.
fn line_range<'a>(node: &'a AstNode<'a>) -> RangeInclusive<usize> {
    let pos = node.data.borrow().sourcepos;
    pos.start.line.saturating_sub(1)..=pos.end.line.saturating_sub(1)
}
