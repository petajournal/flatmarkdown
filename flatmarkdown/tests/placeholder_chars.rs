//! The parser marks places in a body with noncharacters (U+FDD0 to U+FDD2) while it works.
//! A body can come from outside and can hold these characters itself. They must come out as
//! they went in.

use flatmarkdown::{BodyOptions, LinkResolution, parse_body, parse_body_with};
use flatmarkdown_ast::Node;

/// Parses with link resolution off, and with it on.
fn asts(md: &str) -> [Node; 2] {
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["../".to_string(), "./".to_string()],
        },
    };
    [parse_body(md), parse_body_with(md, &opts)]
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

// 本文にあるU+FDD0からU+FDD2の文字は、そのまま残る。#には変わらない。
#[test]
fn a_noncharacter_in_a_body_is_kept() {
    for md in [
        "a\u{FDD0}b",
        "a\u{FDD1}b",
        "a\u{FDD2}b",
        "\u{FDD0}",
        "\u{FDD0}\u{FDD0}",
        "\u{FDD0}\u{FDD1}",
        "\u{FDD0}\u{FDD2}",
        "\u{FDD0}\u{FDD0}\u{FDD1}",
    ] {
        for root in asts(md) {
            assert_eq!(inlines(&root), [text(md)], "{md:?}");
        }
    }
}

// これらの文字のとなりにある\#は、今までどおり#になる。
#[test]
fn an_escaped_hash_next_to_a_noncharacter_is_still_a_hash() {
    for (md, value) in [
        ("\u{FDD0}\\#", "\u{FDD0}#"),
        ("\\#\u{FDD0}", "#\u{FDD0}"),
        ("\\#\u{FDD1}", "#\u{FDD1}"),
        ("\u{FDD0}\\#\u{FDD0}\\#", "\u{FDD0}#\u{FDD0}#"),
    ] {
        for root in asts(md) {
            assert_eq!(inlines(&root), [text(value)], "{md:?}");
        }
    }
}

// コードの中でも、これらの文字はそのまま残る。\#は書いたとおりに残る。
#[test]
fn a_noncharacter_in_code_is_kept() {
    for literal in ["a\u{FDD0}b", "\u{FDD0}\\#", "\u{FDD0}\u{FDD1}"] {
        let md = format!("`{literal}`");
        for root in asts(&md) {
            assert_eq!(
                inlines(&root),
                [Node::Code {
                    literal: literal.to_string()
                }],
                "{md:?}"
            );
        }
    }
}

// 相手のない脚注がある本文でも、これらの文字で始まる段落は消えない。
#[test]
fn a_paragraph_that_starts_with_a_noncharacter_is_kept_beside_a_lone_footnote() {
    for first in ["\u{FDD1}first", "\u{FDD0}\u{FDD2}first", "\u{FDD0}first"] {
        let md = format!("{first}\n\nText[^1].");
        for root in asts(&md) {
            assert_eq!(root.children().len(), 2, "{md:?}");
            assert_eq!(inlines(&root), [text(first)], "{md:?}");
        }
    }
}
