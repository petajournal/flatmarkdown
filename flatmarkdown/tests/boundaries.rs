//! Nodes and edge cases that no other test reached. Mutation testing found them: the code
//! could be changed at these places without any test failing.

use flatmarkdown::{BodyOptions, LinkResolution, body_to_ast, body_to_ast_with};
use flatmarkdown_ast::{Node, parse};

fn ast(md: &str) -> Node {
    parse(&body_to_ast(md)).unwrap()
}

/// Parses with link resolution on, which also turns hashtags on.
fn ast_with_hashtags(md: &str) -> Node {
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["./".to_string()],
        },
    };
    parse(&body_to_ast_with(md, &opts)).unwrap()
}

/// The inline nodes of the first block.
fn inlines(root: &Node) -> &[Node] {
    root.children()[0].children()
}

fn text(value: &str) -> Node {
    Node::Text {
        value: value.to_string(),
    }
}

fn hashtag(value: &str) -> Node {
    Node::Hashtag {
        value: value.to_string(),
    }
}

// ── Nodes ──

// `-# 文字`はsubtextになる。子はインラインのノードである。
#[test]
fn a_subtext_line_is_a_subtext_node_with_inline_children() {
    let root = ast("-# small **bold**");
    assert_eq!(
        root.children(),
        [Node::Subtext {
            children: vec![
                text("small "),
                Node::Strong {
                    children: vec![text("bold")]
                }
            ]
        }]
    );
}

// 表の列の配置は、指定がなければnone、あればleft、center、rightになる。
#[test]
fn the_alignment_of_a_table_column_is_none_unless_it_is_given() {
    let root = ast("| a | b | c | d |\n|---|:--|:-:|--:|\n| 1 | 2 | 3 | 4 |");
    match &root.children()[0] {
        Node::Table {
            alignments,
            num_columns,
            ..
        } => {
            assert_eq!(alignments, &["none", "left", "center", "right"]);
            assert_eq!(*num_columns, 4);
        }
        other => panic!("expected table, got {other:?}"),
    }
}

// ── Hashtags ──

// ハッシュタグだけの文字は、1つのhashtagになる。空の文字は残らない。
#[test]
fn a_text_that_is_only_a_hashtag_becomes_one_hashtag_and_no_empty_text() {
    let root = ast_with_hashtags("#tag");
    assert_eq!(inlines(&root), [hashtag("tag")]);
}

// ハッシュタグで終わる文字は、文字とhashtagになる。後ろに空の文字は残らない。
#[test]
fn a_text_that_ends_with_a_hashtag_leaves_no_empty_text_after_it() {
    let root = ast_with_hashtags("see #tag");
    assert_eq!(inlines(&root), [text("see "), hashtag("tag")]);
}

// 名前のない#は、ハッシュタグにならない。
#[test]
fn a_hash_without_a_name_is_not_a_hashtag() {
    let root = ast_with_hashtags("a # b");
    assert_eq!(inlines(&root), [text("a # b")]);
    let root = ast_with_hashtags("a #");
    assert_eq!(inlines(&root), [text("a #")]);
}

// /だけの名前は、ハッシュタグにならない。
#[test]
fn a_name_of_only_slashes_is_not_a_hashtag() {
    let root = ast_with_hashtags("a #/ b #// c");
    assert_eq!(inlines(&root), [text("a #/ b #// c")]);
}

// 名前の末尾の/は名前に入らず、後ろの文字に残る。
#[test]
fn a_slash_at_the_end_of_a_name_stays_in_the_text_after_the_hashtag() {
    let root = ast_with_hashtags("#a/b/ c");
    assert_eq!(inlines(&root), [hashtag("a/b"), text("/ c")]);
}

// ── Footnote labels ──

// 空の名前の[^]は、脚注にならない。
#[test]
fn an_empty_footnote_label_is_plain_text() {
    let root = ast("a[^] b");
    assert_eq!(root.children().len(), 1);
    assert_eq!(inlines(&root), [text("a[^] b")]);
}

// 空白を含む名前は、脚注にならない。本文は1つの段落のままである。
#[test]
fn a_footnote_label_with_a_space_is_plain_text() {
    let root = ast("a[^x y] b");
    assert_eq!(root.children().len(), 1);
    assert_eq!(inlines(&root), [text("a[^x y] b")]);
}

// 定義が自分自身を参照していても、定義は1つ、参照は1つである。
#[test]
fn a_definition_that_refers_to_itself_is_one_definition_and_one_reference() {
    let root = ast("[^a]: see [^a]");
    let mut definitions = 0;
    let mut references = 0;
    root.walk(&mut |node| match node {
        Node::FootnoteDefinition { .. } => definitions += 1,
        Node::FootnoteReference { .. } => references += 1,
        _ => {}
    });
    assert_eq!((definitions, references), (1, 1));
}

// 参照のない同じ名前の定義が2つあっても、足した参照は残らない。
#[test]
fn two_definitions_of_one_label_without_a_reference_leave_no_reference() {
    let root = ast("[^a]: one\n\n[^a]: two");
    let mut references = 0;
    root.walk(&mut |node| {
        if matches!(node, Node::FootnoteReference { .. }) {
            references += 1;
        }
    });
    assert_eq!(references, 0);
    assert!(
        root.children()
            .iter()
            .all(|block| matches!(block, Node::FootnoteDefinition { .. }))
    );
}
