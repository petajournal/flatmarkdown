//! An inline footnote is written as `^[content]`. These tests check that it becomes a
//! reference and a definition in the same body, as SPEC_AST.md says.

use flatmarkdown::{parse_body, parse_page};
use serde_json::{Value, json};

fn ast(md: &str) -> Value {
    serde_json::from_str(&parse_body(md).to_json()).unwrap()
}

fn text(value: &str) -> Value {
    json!({"type": "text", "value": value})
}

// インライン脚注は、その場所の参照と、本文の末尾の定義になる。名前は__inline_1である。
#[test]
fn an_inline_footnote_is_a_reference_in_place_and_a_definition_at_the_end() {
    assert_eq!(
        ast("Hi^[a note]."),
        json!({"type": "document", "children": [
            {"type": "paragraph", "children": [
                text("Hi"),
                {"type": "footnote_reference", "name": "__inline_1", "ref_num": 1, "ix": 1},
                text(".")
            ]},
            {"type": "footnote_definition", "name": "__inline_1", "children": [
                {"type": "paragraph", "children": [text("a note")]}
            ]}
        ]})
    );
}

// インライン脚注の中身は、強調などのインラインの書き方を持てる。
#[test]
fn the_content_of_an_inline_footnote_can_hold_inline_markup() {
    let root = ast("a^[two **bold**]");
    assert_eq!(
        root["children"][1]["children"][0]["children"],
        json!([text("two "), {"type": "strong", "children": [text("bold")]}])
    );
}

// 2つ目のインライン脚注の名前は__inline_2である。番号は本文の中で1から数える。
#[test]
fn inline_footnotes_are_numbered_from_one_in_a_body() {
    let root = ast("a^[one] b^[two]");
    assert_eq!(root["children"][1]["name"], "__inline_1");
    assert_eq!(root["children"][2]["name"], "__inline_2");
}

// 名前は項目ごとに1から数え直す。別の項目のインライン脚注が、同じ名前になる。
#[test]
fn the_names_of_inline_footnotes_restart_in_every_item_of_a_page() {
    let page: Value =
        serde_json::from_str(&parse_page("first^[one]\n\nsecond^[two]\n").to_json()).unwrap();
    for item in page["items"].as_array().unwrap() {
        assert_eq!(
            item["ast"]["children"][0]["children"][1]["name"],
            "__inline_1"
        );
        assert_eq!(item["ast"]["children"][1]["name"], "__inline_1");
    }
}

// 中身が空の^[]と、コードの中の^[x]は、脚注にならない。
#[test]
fn an_empty_one_and_one_in_code_are_not_footnotes() {
    assert_eq!(
        ast("a^[]")["children"][0]["children"],
        json!([text("a^[]")])
    );
    assert_eq!(
        ast("`a^[x]`")["children"][0]["children"],
        json!([{"type": "code", "literal": "a^[x]"}])
    );
}
