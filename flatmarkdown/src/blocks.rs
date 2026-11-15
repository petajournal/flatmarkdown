//! Blocks that open with a line and close with a line, such as a fenced code block. While such
//! a block is not closed, it runs to the end of the page and takes the items after it.

use crate::ast::options;
use crate::parse_page;
use comrak::nodes::{AstNode, NodeValue};
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
    closed(body, &open_blocks(body))
}

/// Splits `body` before its character `at`, as Enter splits an item. When the place is inside
/// blocks, the part before it gets their closing lines, and the part after it gets their
/// opening lines: each part is a whole body that takes no item after it.
///
/// A place inside an opening line, such as between the backticks of a fence, leaves the
/// closing line of the block alone in the part after it, where it opens a block. That part is
/// closed too, so the promise holds for any place.
pub fn split_body(body: &str, at: usize) -> (String, String) {
    let place = body.char_indices().nth(at).map_or(body.len(), |(i, _)| i);
    let (before, after) = body.split_at(place);
    let open = open_blocks(before);
    let mut after_lines: Vec<&str> = open.iter().map(|block| block.opening.as_str()).collect();
    after_lines.push(after);
    (closed(before, &open), close_blocks(&after_lines.join("\n")))
}

/// A block that a body ends inside: the line that opened it, and a line that closes it.
struct OpenBlock {
    opening: String,
    closing: String,
}

/// `body` with the closing lines of `open`, the innermost first.
fn closed(body: &str, open: &[OpenBlock]) -> String {
    open.iter().rev().fold(body.to_string(), |text, block| {
        format!("{text}\n{}", block.closing)
    })
}

/// The blocks that `body` ends inside, the outermost first.
fn open_blocks(body: &str) -> Vec<OpenBlock> {
    if !has_unclosed_block(body) {
        return Vec::new();
    }
    let lines: Vec<&str> = body.lines().collect();
    let arena = Arena::new();
    // A page has a line break after the body. Without it, a line such as `<pre` at the end is
    // read as a paragraph, and not as the HTML block it is in the page
    let root = parse_document(&arena, &format!("{body}\n"), &options(false));
    // An open block takes everything after it, so it is the last block of the body
    let Some(last) = root.last_child() else {
        return Vec::new();
    };
    let value = last.data.borrow().value.clone();
    match value {
        NodeValue::MultilineBlockQuote(quote) => {
            let marks = ">".repeat(quote.fence_length);
            let mut open = vec![OpenBlock {
                opening: marks.clone(),
                closing: marks,
            }];
            // A block inside it can be open too. A `>>>` inside it closes it, so no quote is
            // inside another: two levels at most
            open.extend(
                last.last_child()
                    .and_then(|inner| open_block(inner, &lines)),
            );
            open
        }
        _ => open_block(last, &lines).into_iter().collect(),
    }
}

/// `node` as an open block, or `None` when it is closed or is no block that closes with a line.
fn open_block<'a>(node: &'a AstNode<'a>, lines: &[&str]) -> Option<OpenBlock> {
    let data = node.data.borrow();
    let first_line = lines
        .get(data.sourcepos.start.line.saturating_sub(1))
        .copied()
        .unwrap_or("");
    match &data.value {
        NodeValue::CodeBlock(code) if code.fenced && !code.closed => Some(OpenBlock {
            opening: first_line.trim_start().to_string(),
            closing: char::from(code.fence_char)
                .to_string()
                .repeat(code.fence_length),
        }),
        NodeValue::HtmlBlock(html) => {
            let (opening, closing) = html_marks(html.block_type, first_line)?;
            // The block ends at the first line with its closing text, in any case, as `</PRE>`
            let literal = html.literal.to_ascii_lowercase();
            (!literal.contains(closing.as_str())).then_some(OpenBlock { opening, closing })
        }
        _ => None,
    }
}

/// The text that opens an HTML block of `block_type` on its `first_line`, and the text that
/// ends it, for the kinds that end with a text and not with a blank line.
fn html_marks(block_type: u8, first_line: &str) -> Option<(String, String)> {
    let first = first_line.trim_start();
    let marks = |opening: &str, closing: &str| Some((opening.to_string(), closing.to_string()));
    match block_type {
        1 => {
            let lower = first.to_ascii_lowercase();
            let tag = ["pre", "script", "style", "textarea"]
                .into_iter()
                .find(|tag| lower.starts_with(&format!("<{tag}")))?;
            // The tag with its attributes, without the text after it on the line
            let opening = first.find('>').map_or(first, |end| &first[..=end]);
            Some((opening.to_string(), format!("</{tag}>")))
        }
        2 => marks("<!--", "-->"),
        3 => marks("<?", "?>"),
        4 => {
            let name_end = first
                .char_indices()
                .skip(2)
                .find(|(_, c)| !c.is_ascii_alphabetic())
                .map_or(first.len(), |(i, _)| i);
            Some((first[..name_end].to_string(), ">".to_string()))
        }
        5 => marks("<![CDATA[", "]]>"),
        _ => None,
    }
}
