//! Blocks that open with a line and close with a line, such as a fenced code block. While such
//! a block is not closed, it runs to the end of the page and takes the items after it.

use crate::parse_page;

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
