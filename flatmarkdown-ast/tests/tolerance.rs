//! Checks that reading is tolerant: unknown nodes, missing fields, and what is still an error.
//! No Markdown parser is used.

use flatmarkdown_ast::{Node, extract_text, parse};

/// All type names in the tree, depth first.
fn type_names(root: &Node) -> Vec<&str> {
    let mut names = Vec::new();
    root.walk(&mut |n| names.push(n.type_name()));
    names
}

fn first_block(root: &Node) -> &Node {
    &root.children()[0]
}

// 未知の型のノードはエラーにならず Unknown として子を保持し、余分なフィールドは無視される。
#[test]
fn unknown_type_is_tolerated_and_keeps_children() {
    let json = r#"{"type":"document","children":[
        {"type":"paragraph","children":[
            {"type":"text","value":"a"},
            {"type":"insert","children":[{"type":"text","value":"b"}]},
            {"type":"text","value":"c","extra_field":1}
        ]}]}"#;
    let root = parse(json).unwrap();
    let para = first_block(&root);
    match &para.children()[1] {
        Node::Unknown {
            node_type,
            children,
        } => {
            assert_eq!(node_type, "insert");
            assert!(matches!(&children[0], Node::Text { value } if value == "b"));
        }
        other => panic!("expected unknown, got {other:?}"),
    }
    assert_eq!(extract_text(&root), "abc");
    assert_eq!(
        type_names(&root),
        ["document", "paragraph", "text", "insert", "text", "text"]
    );
}

// 既知の型でフィールドの型が合わないノード、type のないノード、JSON でない文字列の解析はエラーになる。
#[test]
fn malformed_known_node_is_an_error() {
    assert!(parse(r#"{"type":"heading","level":"two"}"#).is_err());
    assert!(parse(r#"{"children":[]}"#).is_err());
    assert!(parse("not json").is_err());
}

// 省略された任意フィールドは既定値になる。children は空、info と literal は空文字列、header は false である。
#[test]
fn missing_optional_fields_default() {
    let root = parse(r#"{"type":"document"}"#).unwrap();
    assert!(root.children().is_empty());
    let cb = parse(r#"{"type":"code_block"}"#).unwrap();
    assert!(
        matches!(cb, Node::CodeBlock { info, literal } if info.is_empty() && literal.is_empty())
    );
    let row = parse(r#"{"type":"table_row"}"#).unwrap();
    assert!(matches!(row, Node::TableRow { header: false, .. }));
}
