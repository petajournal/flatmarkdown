//! A wikilink without a page name is not a wikilink. `[[]]` stays text, so that a `wikilink`
//! node never has an empty `url`.

use flatmarkdown::{BodyOptions, LinkResolution, parse_body, parse_body_with};
use flatmarkdown_ast::Node;

fn ast(md: &str) -> Node {
    parse_body(md)
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
    let root = parse_body_with("[[]] #tag", &opts);
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

// ページ名もIDもない[[#]]は、wikilinkにならない。文字のままである。ラベルがあっても同じである。
#[test]
fn a_wikilink_with_only_a_hash_sign_stays_text() {
    assert_eq!(inlines(&ast("[[#]]")), [text("[[#]]")]);
    assert_eq!(inlines(&ast("[[#|label]]")), [text("[[#|label]]")]);
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

// \#は#と同じである。[[\#]]も、wikilinkにならない。文字のままである。
#[test]
fn a_wikilink_with_only_an_escaped_hash_sign_stays_text() {
    assert_eq!(inlines(&ast("[[\\#]]")), [text("[[#]]")]);
    assert_eq!(inlines(&ast("[[\\#|label]]")), [text("[[#|label]]")]);
}

// ページ名の前後の空白は、全角空白やノーブレークスペースも落とす。
#[test]
fn any_whitespace_around_a_page_name_is_dropped() {
    for md in [
        "[[\u{3000}a\u{3000}]]",
        "[[\u{A0}a\u{A0}]]",
        "[[ \u{3000} a]]",
    ] {
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

// ラベルがあっても、ページ名の前後の空白を落とす。ラベルはそのまま残る。
#[test]
fn the_whitespace_is_dropped_when_a_label_follows() {
    assert_eq!(
        inlines(&ast("[[\u{3000}a\u{3000}|l]]")),
        [Node::Wikilink {
            url: "a".to_string(),
            children: vec![text("l")],
        }]
    );
}

// 全角空白やノーブレークスペースだけの名前も、名前ではない。文字のままである。
#[test]
fn a_wikilink_with_only_wide_whitespace_as_its_name_stays_text() {
    for md in ["[[\u{3000}]]", "[[\u{A0}]]", "[[ \u{3000} ]]"] {
        assert_eq!(inlines(&ast(md)), [text("[[]]")], "{md:?}");
    }
    assert_eq!(inlines(&ast("[[\u{3000}|label]]")), [text("[[|label]]")]);
}

// #のあとにIDがなければ、#は落とす。[[a#]]は[[a]]と同じである。
#[test]
fn a_hash_sign_without_an_id_is_dropped_from_a_page_name() {
    for md in ["[[a#]]", "[[a# ]]", "[[a#|a]]"] {
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

// ##は、IDのない#ではない。[[##]]のurlは#になりそうだが、#だけの名前は名前ではないので、文字のままである。
#[test]
fn a_wikilink_with_two_hash_signs_stays_text() {
    assert_eq!(inlines(&ast("[[##]]")), [text("[[##]]")]);
}
