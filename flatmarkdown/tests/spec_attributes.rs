//! Attributes and rules of SPEC_AST.md that no other test of the parser checked. A review of
//! the specification against the tests found them on 2026-11-10.

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

// ── Lists ──

// 番号のあとが)の番号付きリストは、delimiterがparenである。.ならperiodである。
#[test]
fn the_delimiter_of_an_ordered_list_is_paren_or_period() {
    for (md, expected) in [("1) a", "paren"), ("1. a", "period")] {
        match &ast(md).children()[0] {
            Node::List { delimiter, .. } => assert_eq!(delimiter, expected, "{md:?}"),
            other => panic!("expected list, got {other:?}"),
        }
    }
}

// 番号付きリストのitemのstartは、その項目の番号である。listのstartは、最初の番号である。
#[test]
fn the_start_of_an_ordered_item_is_its_own_number() {
    let root = ast("3. a\n4. b");
    let Node::List {
        start, children, ..
    } = &root.children()[0]
    else {
        panic!("expected list, got {root:?}");
    };
    assert_eq!(*start, 3);
    let starts: Vec<u32> = children
        .iter()
        .map(|item| match item {
            Node::Item { start, .. } => *start,
            other => panic!("expected item, got {other:?}"),
        })
        .collect();
    assert_eq!(starts, [3, 4]);
}

// ── Raw HTML ──

// html_blockのblock_typeは、CommonMarkのHTMLブロックの種類の番号である。コメントは2、<div>は6である。
#[test]
fn the_block_type_of_an_html_block_is_the_kind_of_the_block() {
    assert_eq!(
        ast("<!-- c -->").children(),
        [Node::HtmlBlock {
            block_type: 2,
            literal: "<!-- c -->".to_string()
        }]
    );
    assert_eq!(
        ast("<div>\nx\n</div>").children(),
        [Node::HtmlBlock {
            block_type: 6,
            literal: "<div>\nx\n</div>".to_string()
        }]
    );
}

// ── Footnotes ──

// 同じ脚注を2回参照すると、2回目のref_numは2になる。ixは脚注ごとの番号で、同じ脚注なら同じである。
#[test]
fn the_second_reference_to_a_footnote_has_the_ref_num_two() {
    let root = ast("[^a] [^a] [^b]\n\n[^a]: x\n\n[^b]: y");
    let reference = |name: &str, ref_num, ix| Node::FootnoteReference {
        name: name.to_string(),
        ref_num,
        ix,
    };
    assert_eq!(
        root.children()[0].children(),
        [
            reference("a", 1, 1),
            text(" "),
            reference("a", 2, 1),
            text(" "),
            reference("b", 1, 2),
        ]
    );
}

// ── Front matter ──

// 本文の先頭の---は水平線である。front matterは、ページの先頭でだけ読まれる。
#[test]
fn a_body_that_starts_like_front_matter_starts_with_a_thematic_break() {
    let root = ast("---\na: b\n---\n\nx");
    assert_eq!(
        root.children(),
        [
            Node::ThematicBreak,
            Node::Heading {
                level: 2,
                children: vec![text("a: b")]
            },
            Node::Paragraph {
                children: vec![text("x")]
            },
        ]
    );
}

// ── Hashtags ──

// 改行の直後の#tagは、ハッシュタグになる。改行のあとは、新しいtextノードの先頭だからである。
#[test]
fn a_hashtag_can_start_right_after_a_line_break() {
    let root = ast_with_hashtags("a\n#tag");
    assert_eq!(
        root.children()[0].children(),
        [text("a"), Node::Softbreak, hashtag("tag")]
    );
}

// ハッシュタグの位置は、textノードの中で判定する。強調の直後の#tagは、空白がなくてもハッシュタグになる。
#[test]
fn a_hashtag_can_start_right_after_inline_markup() {
    let root = ast_with_hashtags("**b**#tag");
    assert_eq!(
        root.children()[0].children(),
        [
            Node::Strong {
                children: vec![text("b")]
            },
            hashtag("tag")
        ]
    );
}

// 文字参照の&#35;は#になる。textノードの先頭なら、ハッシュタグになる。タグにしない書き方は\#である。
#[test]
fn a_character_reference_of_the_hash_sign_can_start_a_hashtag() {
    let root = ast_with_hashtags("&#35;tag");
    assert_eq!(root.children()[0].children(), [hashtag("tag")]);
    let root = ast_with_hashtags("a&#35;tag");
    assert_eq!(root.children()[0].children(), [text("a#tag")]);
}
