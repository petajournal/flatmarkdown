//! `||text||` is a spoiler. A single `|` on each side of some text is not: it is plain text.
//! These tests check that the single pipes stay in the output, and that a spoiler is still a
//! spoiler.

use flatmarkdown::{BodyOptions, LinkResolution, body_to_ast, body_to_ast_with, body_to_html};
use serde_json::{Value, json};

/// The inline nodes of the first block.
fn inlines(md: &str) -> Value {
    let ast: Value = serde_json::from_str(&body_to_ast(md).to_json()).unwrap();
    ast["children"][0]["children"].clone()
}

fn text(value: &str) -> Value {
    json!({"type": "text", "value": value})
}

// 1本の|で囲んだ文字は、|を含む1つの文字になる。
#[test]
fn text_between_single_pipes_keeps_its_pipes() {
    assert_eq!(inlines("a |b| c"), json!([text("a |b| c")]));
    assert_eq!(inlines("|x| + |y|"), json!([text("|x| + |y|")]));
}

// 1本の|で囲んだ文字は、HTMLでも|が残る。
#[test]
fn text_between_single_pipes_keeps_its_pipes_in_html() {
    assert_eq!(body_to_html("a |b| c"), "<p>a |b| c</p>\n");
}

// 1本の|の中の強調は強調のまま残り、その前後に|の文字が置かれる。
#[test]
fn markup_between_single_pipes_stays_and_gets_a_pipe_on_each_side() {
    assert_eq!(
        inlines("|**b**|"),
        json!([
            text("|"),
            {"type": "strong", "children": [text("b")]},
            text("|")
        ])
    );
}

// 1本の|の中の改行は改行のまま残る。
#[test]
fn a_line_break_between_single_pipes_stays() {
    assert_eq!(
        inlines("x |a\nb| y"),
        json!([text("x |a"), {"type": "softbreak"}, text("b| y")])
    );
}

// ||で囲んだ文字はスポイラーのままである。その隣の1本の|は文字になる。
#[test]
fn a_spoiler_next_to_single_pipes_is_still_a_spoiler() {
    assert_eq!(
        inlines("|a| ||b||"),
        json!([
            text("|a| "),
            {"type": "spoilered_text", "children": [text("b")]}
        ])
    );
}

// スポイラーの中の1本の|は、スポイラーの中の文字になる。
#[test]
fn single_pipes_inside_a_spoiler_are_text_of_the_spoiler() {
    assert_eq!(
        inlines("||a |b| c||"),
        json!([{"type": "spoilered_text", "children": [text("a |b| c")]}])
    );
}

// 1本の|の中のスポイラーはスポイラーのままで、外側の|は文字になる。
#[test]
fn a_spoiler_inside_single_pipes_is_still_a_spoiler() {
    assert_eq!(
        inlines("|a ||b|| c|"),
        json!([
            text("|a "),
            {"type": "spoilered_text", "children": [text("b")]},
            text(" c|")
        ])
    );
}

// |の直後の#は、ハッシュタグにならない。#の前が空白ではないためである。
#[test]
fn a_hash_right_after_a_single_pipe_is_not_a_hashtag() {
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["./".to_string()],
        },
    };
    let ast: Value = serde_json::from_str(&body_to_ast_with("|#tag|", &opts).to_json()).unwrap();
    assert_eq!(ast["children"][0]["children"], json!([text("|#tag|")]));
}
