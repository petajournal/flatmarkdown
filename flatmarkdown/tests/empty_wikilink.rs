//! A wikilink without a page name is not a wikilink. `[[]]` stays text, so that a `wikilink`
//! node never has an empty `url`.

use flatmarkdown::{BodyOptions, LinkResolution, body_to_ast, body_to_ast_with};
use flatmarkdown_ast::{Node, parse};

fn ast(md: &str) -> Node {
    parse(&body_to_ast(md)).unwrap()
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

// ページ名のない[[]]は、wikilinkにならない。文字のままである。
#[test]
fn a_wikilink_without_a_page_name_stays_text() {
    let root = ast("[[]]");
    assert_eq!(inlines(&root), [text("[[]]")]);
}

// ページ名がなくラベルだけの[[|label]]も、wikilinkにならない。|も文字として残る。
#[test]
fn a_wikilink_with_only_a_label_stays_text() {
    let root = ast("[[|label]]");
    assert_eq!(inlines(&root), [text("[[|label]]")]);
}

// 空白だけの名前は、名前ではない。[[ ]]は、[[]]という文字になる。
#[test]
fn a_wikilink_with_only_whitespace_as_its_name_stays_text() {
    for md in ["[[ ]]", "[[   ]]", "[[|]]"] {
        assert_eq!(inlines(&ast(md)), [text("[[]]")], "{md:?}");
    }
}

// 文字に戻った[[]]は、前後の文字と1つのtextノードになる。
#[test]
fn an_empty_wikilink_joins_the_text_around_it() {
    let root = ast("a [[]] b");
    assert_eq!(inlines(&root), [text("a [[]] b")]);
}

// 文字に戻った[[]]のあとの#tagは、今までどおりハッシュタグになる。
#[test]
fn a_hashtag_after_an_empty_wikilink_is_still_a_hashtag() {
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["./".to_string()],
        },
    };
    let root = parse(&body_to_ast_with("[[]] #tag", &opts)).unwrap();
    assert_eq!(
        inlines(&root),
        [
            text("[[]] "),
            Node::Hashtag {
                value: "tag".to_string()
            }
        ]
    );
}

// ページ名のあるwikilinkは、今までどおりwikilinkである。#idだけの[[#id]]も、名前が空ではない。
#[test]
fn a_wikilink_with_a_name_is_still_a_wikilink() {
    for (md, url) in [("[[page]]", "page"), ("[[#id]]", "#id")] {
        assert_eq!(
            inlines(&ast(md)),
            [Node::Wikilink {
                url: url.to_string(),
                children: vec![text(url)],
            }],
            "{md:?}"
        );
    }
}

// ラベルが空の[[a|]]は、ラベルのない[[a]]と同じである。子はページ名の文字になる。
#[test]
fn a_wikilink_with_an_empty_label_shows_its_page_name() {
    for md in ["[[a|]]", "[[a| ]]"] {
        assert_eq!(
            inlines(&ast(md)),
            [Node::Wikilink {
                url: "a".to_string(),
                children: vec![text("a")],
            }],
            "{md:?}"
        );
    }
}
