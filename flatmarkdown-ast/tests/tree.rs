//! Checks the functions that walk and change a tree of nodes. No Markdown parser is used.

use flatmarkdown_ast::{Node, extract_text, heading_level, parse};

// children_mutで、子の並びを書き換えられる。
#[test]
fn children_mut_lets_the_children_be_changed() {
    let mut paragraph = parse(
        r#"{"type":"paragraph","children":[{"type":"text","value":"a"},{"type":"softbreak"}]}"#,
    )
    .unwrap();
    let children = paragraph.children_mut().unwrap();
    children.retain(|child| matches!(child, Node::Text { .. }));
    children.push(Node::Text { value: "b".into() });
    assert_eq!(extract_text(&paragraph), "ab");
    assert_eq!(paragraph.children().len(), 2);
}

// 子を持てないノードのchildren_mutは、Noneである。
#[test]
fn children_mut_of_a_leaf_is_none() {
    let mut text = Node::Text { value: "a".into() };
    assert!(text.children_mut().is_none());
    let mut code_block = parse(r#"{"type":"code_block","info":"","literal":"x"}"#).unwrap();
    assert!(code_block.children_mut().is_none());
}

// 子を持てるノードのchildren_mutは、子がなくてもSomeである。
#[test]
fn children_mut_of_an_empty_container_is_some() {
    let mut document = parse(r#"{"type":"document"}"#).unwrap();
    assert!(
        document
            .children_mut()
            .is_some_and(|children| children.is_empty())
    );
}

// extract_textは、textノードの文字だけを文書の順につなぐ。コードの中身は含まない。
#[test]
fn extract_text_joins_only_the_text_nodes_in_document_order() {
    let root = parse(
        r#"{"type":"document","children":[{"type":"paragraph","children":[
            {"type":"text","value":"a "},
            {"type":"strong","children":[{"type":"text","value":"b"}]},
            {"type":"code","literal":"c"},
            {"type":"text","value":" d"}]}]}"#,
    )
    .unwrap();
    assert_eq!(extract_text(&root), "a b d");
}

// heading_levelは、最上位の子のうち最初の見出しのレベルを返す。見出しがなければNoneである。
#[test]
fn heading_level_is_the_level_of_the_first_top_level_heading() {
    let with_heading = parse(
        r#"{"type":"document","children":[{"type":"paragraph"},{"type":"heading","level":3},{"type":"heading","level":1}]}"#,
    )
    .unwrap();
    assert_eq!(heading_level(&with_heading), Some(3));

    let nested_only = parse(
        r#"{"type":"document","children":[{"type":"block_quote","children":[{"type":"heading","level":2}]}]}"#,
    )
    .unwrap();
    assert_eq!(heading_level(&nested_only), None);
}

// walkは、ノード自身とすべての子孫を、深さ優先で訪れる。type_nameは、JSONのtypeの名前を返す。
#[test]
fn walk_visits_the_node_and_its_descendants_depth_first() {
    let root = parse(
        r#"{"type":"document","children":[{"type":"paragraph","children":[{"type":"text","value":"a"}]},{"type":"thematic_break"}]}"#,
    )
    .unwrap();
    let mut names = Vec::new();
    root.walk(&mut |node| names.push(node.type_name()));
    assert_eq!(names, ["document", "paragraph", "text", "thematic_break"]);
}
