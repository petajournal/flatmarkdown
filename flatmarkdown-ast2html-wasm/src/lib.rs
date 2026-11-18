//! The wasm build of `flatmarkdown-ast2html`.
//!
//! This crate only passes strings in and a string out. The rendering is done by
//! `flatmarkdown-ast2html`, so that the npm package and the Rust crate give the same HTML.

use flatmarkdown_ast2html::HtmlOptions;
use wasm_bindgen::prelude::*;

/// Renders the JSON of an AST to HTML, with the settings of [`HtmlOptions`]. A setting that is
/// `undefined` in JavaScript keeps its default value.
///
/// JSON that is not an AST becomes an exception in JavaScript.
#[wasm_bindgen(js_name = renderToHtml)]
pub fn render_to_html(
    json: &str,
    wikilink_href: Option<String>,
    hashtag_href: Option<String>,
    wikilink_fragment_pattern: Option<String>,
) -> Result<String, JsError> {
    let ast = flatmarkdown_ast::Node::from_json(json)?;
    let default = HtmlOptions::default();
    let options = HtmlOptions {
        wikilink_href: wikilink_href.unwrap_or(default.wikilink_href),
        hashtag_href: hashtag_href.unwrap_or(default.hashtag_href),
        wikilink_fragment_pattern: wikilink_fragment_pattern
            .unwrap_or(default.wikilink_fragment_pattern),
    };
    Ok(flatmarkdown_ast2html::ast_to_html_with(&ast, &options))
}
