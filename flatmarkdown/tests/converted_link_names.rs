//! A link that becomes a wikilink loses the whitespace around its page path, as `[[ a ]]` does.

use flatmarkdown::{BodyOptions, LinkResolution, body_to_ast_with};
use flatmarkdown_ast::{Node, parse};

/// Parses with link resolution on.
fn ast(md: &str) -> Node {
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["../".to_string()],
        },
    };
    parse(&body_to_ast_with(md, &opts)).unwrap()
}

/// The inline nodes of the first block.
fn inlines(root: &Node) -> &[Node] {
    root.children()[0].children()
}

fn wikilink(url: &str, label: &str) -> Node {
    Node::Wikilink {
        url: url.to_string(),
        children: vec![Node::Text {
            value: label.to_string(),
        }],
    }
}

// wikilinkになるリンクのページパスは、前後の空白を落とす。[[ a ]]と同じである。
#[test]
fn the_whitespace_around_a_converted_page_path_is_dropped() {
    for md in ["[x](<../ a.md>)", "[x](<../a .md>)", "[x](<../  a  .md>)"] {
        assert_eq!(inlines(&ast(md)), [wikilink("a", "x")], "{md:?}");
    }
}

// #idがあっても、ページパスの前後の空白を落とす。
#[test]
fn the_whitespace_is_dropped_before_a_fragment_too() {
    assert_eq!(
        inlines(&ast("[x](<../ a .md#id>)")),
        [wikilink("a#id", "x")]
    );
}

// ページパスの中の空白は、そのまま残る。
#[test]
fn the_whitespace_inside_a_converted_page_path_is_kept() {
    assert_eq!(
        inlines(&ast("[x](<../ a b/c d .md>)")),
        [wikilink("a b/c d", "x")]
    );
}

// 空白を落としたあとの名前でタグのリンクかどうかを決める。
#[test]
fn a_tag_link_is_matched_after_the_whitespace_is_dropped() {
    assert_eq!(
        inlines(&ast("[#a](<../ a .md>)")),
        [Node::Hashtag {
            value: "a".to_string()
        }]
    );
}

// ページパスに\#があるリンクは、wikilinkにならない。a#bは「aの項目b」と読めてしまうからである。
#[test]
fn a_link_with_an_escaped_hash_in_its_page_path_stays_a_link() {
    assert_eq!(
        inlines(&ast("[x](../a\\#b.md)")),
        [Node::Link {
            url: "../a#b.md".to_string(),
            title: String::new(),
            children: vec![Node::Text {
                value: "x".to_string()
            }],
        }]
    );
}

// パーサーが目印に使う文字がページパスにあっても、\#とは取り違えない。wikilinkになる。
#[test]
fn a_noncharacter_in_a_page_path_is_not_read_as_an_escaped_hash() {
    assert_eq!(
        inlines(&ast("[x](../\u{FDD0}\u{FDD1}.md)")),
        [wikilink("\u{FDD0}\u{FDD1}", "x")]
    );
}

// #のあとにIDがなければ、#は落とす。
#[test]
fn a_hash_sign_without_an_id_is_dropped_from_a_converted_page_path() {
    assert_eq!(inlines(&ast("[x](../a.md#)")), [wikilink("a", "x")]);
}

// IDのない#のあるタグのリンクも、hashtagになる。
#[test]
fn a_tag_link_with_a_hash_sign_without_an_id_is_a_hashtag() {
    assert_eq!(
        inlines(&ast("[#a](../a.md#)")),
        [Node::Hashtag {
            value: "a".to_string()
        }]
    );
}
