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

// 省略された任意フィールドは既定値になる。children は空、info と literal は空文字列、fenced は true、header は false である。
#[test]
fn missing_optional_fields_default() {
    let root = parse(r#"{"type":"document"}"#).unwrap();
    assert!(root.children().is_empty());
    let cb = parse(r#"{"type":"code_block"}"#).unwrap();
    assert!(
        matches!(cb, Node::CodeBlock { fenced: true, info, literal } if info.is_empty() && literal.is_empty())
    );
    let row = parse(r#"{"type":"table_row"}"#).unwrap();
    assert!(matches!(row, Node::TableRow { header: false, .. }));
}

// 古いASTの item にある tight は、無視して読める。
#[test]
fn tight_on_an_item_of_an_old_ast_is_ignored() {
    let item = parse(r#"{"type":"item","list_type":"bullet","start":1,"tight":false}"#).unwrap();
    assert!(matches!(item, Node::Item { start: 1, .. }));
}

// 古いASTの front_matter ノードは、未知のノードとして読める。front matter はページの文字列の項目であり、ノードではない。
#[test]
fn a_front_matter_node_of_an_old_ast_is_read_as_unknown() {
    let node = parse(r#"{"type":"front_matter","value":"---\nkey: value\n---\n"}"#).unwrap();
    assert!(matches!(node, Node::Unknown { node_type, .. } if node_type == "front_matter"));
}

// 古いASTの raw、escaped、escaped_tag は、未知のノードとして読める。パーサーが出力しないので、仕様にない。
#[test]
fn raw_escaped_and_escaped_tag_of_an_old_ast_are_read_as_unknown() {
    for (json, name) in [
        (r#"{"type":"raw","value":"x"}"#, "raw"),
        (r#"{"type":"escaped"}"#, "escaped"),
        (r#"{"type":"escaped_tag","value":"script"}"#, "escaped_tag"),
    ] {
        let node = parse(json).unwrap();
        assert!(
            matches!(&node, Node::Unknown { node_type, .. } if node_type == name),
            "{name}: {node:?}"
        );
    }
}

// 未知のノードは、type が unknown のノードとして書き出される。元の type の名前は書き出されない。
#[test]
fn an_unknown_node_is_written_with_the_type_unknown() {
    let node = parse(r#"{"type":"insert","children":[{"type":"text","value":"x"}]}"#).unwrap();
    assert_eq!(
        serde_json::to_value(&node).unwrap(),
        serde_json::json!({"type": "unknown", "children": [{"type": "text", "value": "x"}]})
    );
}
