//! Checks the functions that walk and change a tree of nodes. No Markdown parser is used.

use flatmarkdown_ast::{Node, extract_text, parse};

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
