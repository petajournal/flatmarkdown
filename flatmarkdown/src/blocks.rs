//! Blocks that open with a line and close with a line, such as a fenced code block. While such
//! a block is not closed, it runs to the end of the page and takes the items after it.

use crate::ast::options;
use crate::parse_page;
use comrak::nodes::NodeValue;
use comrak::{Arena, parse_document};

/// The metadata line of the item that is put after a body to see whether the body takes it.
const AFTER: &str = "<!-- @ unclosed=\"after\" -->\nafter";

/// Whether `body` ends inside a block that is still open, such as a fenced code block without
/// its closing fence. Such a body takes the items after it when it is written into a page.
pub fn has_unclosed_block(body: &str) -> bool {
    // The line before the body keeps a body that starts with `---` from being front matter
    let page = parse_page(&format!("<!-- @ -->\n{body}\n\n{AFTER}\n"));
    !page.items.last().is_some_and(|item| {
        item.body == "after"
            && item
                .properties
                .get("unclosed")
                .is_some_and(|v| v == "after")
    })
}

/// `body` with the closing lines of the blocks it ends inside, so that it takes no item after
/// it. A body that ends inside no block is given back as it is.
pub fn close_blocks(body: &str) -> String {
    let closings = closing_lines(body);
    if closings.is_empty() {
        return body.to_string();
    }
    format!("{body}\n{}", closings.join("\n"))
}

/// The lines that close the blocks `body` ends inside, the innermost first.
fn closing_lines(body: &str) -> Vec<String> {
    if !has_unclosed_block(body) {
        return Vec::new();
    }
    let arena = Arena::new();
    let root = parse_document(&arena, body, &options(false));
    // An open block takes everything after it, so it is the last block of the body
    let Some(last) = root.last_child() else {
        return Vec::new();
    };
    match &last.data.borrow().value {
        NodeValue::CodeBlock(code) => vec![fence(code.fence_char, code.fence_length)],
        _ => Vec::new(),
    }
}

/// A closing fence of `length` times `char`.
fn fence(char: u8, length: usize) -> String {
    char::from(char).to_string().repeat(length)
}
