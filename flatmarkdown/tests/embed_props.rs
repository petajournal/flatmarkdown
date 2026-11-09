//! The properties of an embed come from the fragment of its URL (`#key=value&flag`). These
//! tests check what the parser does with the unusual forms.

use flatmarkdown::body_to_ast;
use serde_json::{Value, json};

/// The `embed` node of a body that holds only one image.
fn embed(md: &str) -> Value {
    let root: Value = serde_json::from_str(&body_to_ast(md)).unwrap();
    let node = root["children"][0]["children"][0].clone();
    assert_eq!(node["type"], "embed", "{md:?}");
    node
}

// キーが空のプロパティ（=3）は捨てる。名前がないと、レンダラーが読めないからである。
#[test]
fn a_property_with_an_empty_key_is_dropped() {
    let node = embed("![a](u.png#w=1&=3)");
    assert_eq!(node["props"], json!({"w": "1"}));
}
