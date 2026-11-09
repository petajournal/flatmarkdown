//! Checks that the types read and write the JSON that SPEC_AST.md defines, and nothing more.

use flatmarkdown_ast::{Node, parse, parse_page};
use serde_json::{Value, json};

/// Reads `json` into the types and writes it back.
fn rewritten(json: &Value) -> Value {
    let node = parse(&json.to_string()).unwrap();
    serde_json::to_value(&node).unwrap()
}

// 子のないノードは、childrenを書き出さない。
#[test]
fn a_node_without_children_is_written_without_children() {
    let empty_document = json!({"type": "document"});
    assert_eq!(rewritten(&empty_document), empty_document);

    let empty_item = json!({"type": "list", "list_type": "bullet", "start": 1, "tight": true, "delimiter": "period",
        "children": [{"type": "item", "list_type": "bullet", "start": 1, "tight": false}]});
    assert_eq!(rewritten(&empty_item), empty_item);
}

// 子のあるノードは、childrenを書き出す。
#[test]
fn a_node_with_children_is_written_with_children() {
    let paragraph = json!({"type": "paragraph", "children": [{"type": "text", "value": "a"}]});
    assert_eq!(rewritten(&paragraph), paragraph);
}

// embedは、propsが空のときpropsを書き出さない。propsがあるときは書き出す。
#[test]
fn an_embed_is_written_without_empty_props() {
    let plain = json!({"type": "embed", "url": "p.png", "title": "",
        "children": [{"type": "text", "value": "alt"}]});
    assert_eq!(rewritten(&plain), plain);

    let with_props =
        json!({"type": "embed", "url": "p.png", "title": "", "props": {"w": "10", "flag": true}});
    assert_eq!(rewritten(&with_props), with_props);
}

// heading、code_block、mathは、仕様にある属性だけを書き出す。
#[test]
fn heading_code_block_and_math_are_written_with_only_the_attributes_of_the_spec() {
    let heading =
        json!({"type": "heading", "level": 2, "children": [{"type": "text", "value": "T"}]});
    assert_eq!(rewritten(&heading), heading);

    let code_block = json!({"type": "code_block", "info": "rust", "literal": "x\n"});
    assert_eq!(rewritten(&code_block), code_block);

    let math = json!({"type": "math", "literal": "x^2"});
    assert_eq!(rewritten(&math), math);
}

// listのstartは、整数として書き出す。
#[test]
fn the_start_of_a_list_is_written_as_an_integer() {
    let list = json!({"type": "list", "list_type": "ordered", "start": 3, "tight": true, "delimiter": "paren"});
    assert_eq!(rewritten(&list), list);
}

// 値がnullになりうる属性は、nullのまま書き出す。
#[test]
fn nullable_attributes_are_written_as_null() {
    let task_item = json!({"type": "task_item", "symbol": null});
    assert_eq!(rewritten(&task_item), task_item);

    let alert = json!({"type": "alert", "alert_type": "note", "title": null});
    assert_eq!(rewritten(&alert), alert);
}

// 仕様にないノードは、Unknownとして読む。子は保たれる。
#[test]
fn a_node_that_the_spec_does_not_define_is_read_as_unknown() {
    for node_type in [
        "image",
        "description_list",
        "description_item",
        "description_term",
        "description_details",
        "heex_block",
        "heex_inline",
        "insert",
    ] {
        let json = json!({"type": node_type, "children": [{"type": "text", "value": "x"}]});
        let node = parse(&json.to_string()).unwrap();
        match &node {
            Node::Unknown {
                node_type: read,
                children,
            } => {
                assert_eq!(read, node_type);
                assert_eq!(children.len(), 1);
            }
            other => panic!("expected unknown for {node_type}, got {other:?}"),
        }
    }
}

// ページは、front_matterと項目の並びとして読める。項目は、level、id、properties、body、astを持つ。
#[test]
fn a_page_is_read_as_front_matter_and_items() {
    let json = json!({
        "type": "page",
        "front_matter": "---\nkey: value\n---\n\n",
        "items": [
            {
                "level": 2,
                "id": "2026-09-26-18-14-47-32W15WT",
                "properties": {"plan": "2026-09-28"},
                "body": "- [ ] Task1",
                "ast": {"type": "document", "children": [{"type": "paragraph"}]}
            },
            {"level": 1, "id": null, "properties": {}, "body": "", "ast": {"type": "document"}}
        ]
    });
    let page = parse_page(&json.to_string()).unwrap();
    assert_eq!(
        page.front_matter.as_deref(),
        Some("---\nkey: value\n---\n\n")
    );
    assert_eq!(page.items.len(), 2);

    let first = &page.items[0];
    assert_eq!(first.level, 2);
    assert_eq!(first.id.as_deref(), Some("2026-09-26-18-14-47-32W15WT"));
    assert_eq!(
        first.properties.get("plan").map(String::as_str),
        Some("2026-09-28")
    );
    assert_eq!(first.body, "- [ ] Task1");
    assert!(matches!(first.ast, Node::Document { .. }));

    let second = &page.items[1];
    assert_eq!(second.id, None);
    assert!(second.properties.is_empty());
}

// ページは、読んだときと同じJSONとして書き出す。typeはpageで、front_matterがないときはnullである。
#[test]
fn a_page_is_written_back_as_the_same_json() {
    let json = json!({
        "type": "page",
        "front_matter": null,
        "items": [
            {"level": 1, "id": null, "properties": {"a": "b"}, "body": "x",
             "ast": {"type": "document", "children": [{"type": "paragraph", "children": [{"type": "text", "value": "x"}]}]}}
        ]
    });
    let page = parse_page(&json.to_string()).unwrap();
    assert_eq!(serde_json::to_value(&page).unwrap(), json);
}

// ページでないJSONと、必須の項目を欠く項目は、エラーになる。
#[test]
fn a_malformed_page_is_an_error() {
    assert!(parse_page("not json").is_err());
    assert!(parse_page(r#"{"type":"page","front_matter":null}"#).is_err());
    assert!(parse_page(r#"{"type":"page","front_matter":null,"items":[{"level":1}]}"#).is_err());
}
