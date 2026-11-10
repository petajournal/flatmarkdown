//! The wasm build of `flatmarkdown-ast2html`.
//!
//! This crate only passes a string in and a string out. The rendering is done by
//! `flatmarkdown-ast2html`, so that the npm package and the Rust crate give the same HTML.

use wasm_bindgen::prelude::*;

/// Renders the JSON of an AST to HTML.
///
/// JSON that is not an AST becomes an exception in JavaScript.
#[wasm_bindgen(js_name = renderToHtml)]
pub fn render_to_html(json: &str) -> Result<String, JsError> {
    let ast = flatmarkdown_ast::Node::from_json(json)?;
    Ok(flatmarkdown_ast2html::ast_to_html(&ast))
}
