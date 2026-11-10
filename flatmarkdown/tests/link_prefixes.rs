//! A link is converted when its URL starts with one of the prefixes. When more than one prefix
//! matches, the longest one is used, so the order of the list does not change the result.

use flatmarkdown::{BodyOptions, LinkResolution, parse_body_with};
use flatmarkdown_ast::Node;

fn ast(md: &str, prefixes: &[&str]) -> Node {
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: prefixes.iter().map(|p| p.to_string()).collect(),
        },
    };
    parse_body_with(md, &opts)
}

/// The `url` of the wikilink that is the only inline node of the body.
fn wikilink_url(root: &Node) -> &str {
    match root.children()[0].children() {
        [Node::Wikilink { url, .. }] => url,
        other => panic!("expected one wikilink, got {other:?}"),
    }
}

// 接頭辞が2つ合うときは、長い方を使う。リストの順序は結果を変えない。
#[test]
fn the_longest_matching_prefix_is_used_whatever_the_order() {
    for prefixes in [["../", "../docs/"], ["../docs/", "../"]] {
        let root = ast("[x](../docs/a.md)", &prefixes);
        assert_eq!(wikilink_url(&root), "a", "{prefixes:?}");
    }
}

// 長い接頭辞が合わないリンクには、合う方の短い接頭辞を使う。
#[test]
fn a_shorter_prefix_is_used_when_the_longer_one_does_not_match() {
    for prefixes in [["../", "../docs/"], ["../docs/", "../"]] {
        let root = ast("[x](../other/a.md)", &prefixes);
        assert_eq!(wikilink_url(&root), "other/a", "{prefixes:?}");
    }
}

// どの接頭辞にも合わないリンクは、linkのままである。
#[test]
fn a_link_that_matches_no_prefix_stays_a_link() {
    let root = ast("[x](./a.md)", &["../", "../docs/"]);
    assert!(matches!(
        root.children()[0].children(),
        [Node::Link { url, .. }] if url == "./a.md"
    ));
}
