//! Tight or loose is a property of a whole list in CommonMark. These tests check that the AST
//! says it on the list only, and not on each item.

use flatmarkdown::body_to_ast;
use serde_json::Value;

fn first_block(md: &str) -> Value {
    let ast: Value = serde_json::from_str(&body_to_ast(md).to_json()).unwrap();
    ast["children"][0].clone()
}

// タイトなリストは、listのtightがtrueである。itemはtightを持たない。
#[test]
fn a_tight_list_says_tight_on_the_list_and_not_on_its_items() {
    let list = first_block("- a\n- b");
    assert_eq!(list["type"], "list");
    assert_eq!(list["tight"], true);
    let item = &list["children"][0];
    assert_eq!(item["type"], "item");
    assert!(item.get("tight").is_none());
}

// ルーズなリストは、listのtightがfalseである。itemはtightを持たない。
#[test]
fn a_loose_list_says_loose_on_the_list_and_not_on_its_items() {
    let list = first_block("1. a\n\n2. b");
    assert_eq!(list["tight"], false);
    assert!(list["children"][0].get("tight").is_none());
}
