//! Flat Markdown parser: an item body or a whole page to a JSON AST, and an item body to HTML.

mod ast;
mod body;
mod footnotes;
mod hashtags;
mod html;
mod links;
mod page;

pub use body::{BodyOptions, LinkResolution, body_to_ast, body_to_ast_with};
pub use html::{body_to_html, body_to_html_with};
pub use page::{page_to_ast, page_to_ast_with};

/// Helpers that the tests of more than one module use.
#[cfg(test)]
mod test_support {
    use crate::page_to_ast;
    use serde_json::Value;

    pub(crate) fn page(input: &str) -> Value {
        serde_json::from_str(&page_to_ast(input)).unwrap()
    }
}

// Run the code examples in the README as doctests, so that they keep working
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
