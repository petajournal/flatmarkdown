//! A code block is written with fences or with an indent of four spaces. These tests check that
//! the AST says which of the two the author wrote.

use flatmarkdown::body_to_ast;
use serde_json::Value;

fn first_block(md: &str) -> Value {
    let ast: Value = serde_json::from_str(&body_to_ast(md).to_json()).unwrap();
    ast["children"][0].clone()
}

// フェンスで書いたコードブロックは、fencedがtrueである。
#[test]
fn a_code_block_written_with_fences_is_fenced() {
    let block = first_block("```rust\nfn main() {}\n```");
    assert_eq!(block["type"], "code_block");
    assert_eq!(block["fenced"], true);
}

// 言語名のないフェンスのコードブロックも、fencedがtrueである。
#[test]
fn a_code_block_written_with_fences_and_no_info_is_fenced() {
    let block = first_block("```\nplain\n```");
    assert_eq!(block["info"], "text");
    assert_eq!(block["fenced"], true);
}

// インデントで書いたコードブロックは、fencedがfalseで、infoは空文字列である。
#[test]
fn a_code_block_written_with_an_indent_is_not_fenced() {
    let block = first_block("    let x = 1;\n    let y = 2;");
    assert_eq!(block["type"], "code_block");
    assert_eq!(block["fenced"], false);
    assert_eq!(block["info"], "");
    assert_eq!(block["literal"], "let x = 1;\nlet y = 2;\n");
}
