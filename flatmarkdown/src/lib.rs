//! Flat Markdown parser: an item body or a whole page to the AST of the `flatmarkdown-ast` crate.

mod ast;
mod body;
mod footnotes;
mod hashtags;
mod links;
mod page;

pub use body::{BodyOptions, LinkResolution, parse_body, parse_body_with};
pub use page::{parse_page, parse_page_with};

/// Helpers that the tests of more than one module use.
#[cfg(test)]
mod test_support {
    use crate::parse_page;
    use serde_json::Value;

    pub(crate) fn page(input: &str) -> Value {
        serde_json::from_str(&parse_page(input).to_json()).unwrap()
    }
}

// Run the code examples in the README as doctests, so that they keep working
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
