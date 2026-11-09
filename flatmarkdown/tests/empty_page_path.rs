//! A link to a `.md` file without a name is not converted. The name of a `.md` file can come
//! from outside, so `.md` alone must not make a wikilink or a hashtag with an empty name.

use flatmarkdown::{BodyOptions, LinkResolution, body_to_ast_with};
use flatmarkdown_ast::{Node, parse};

/// Parses with link resolution on.
fn ast(md: &str) -> Node {
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["../".to_string(), "./".to_string()],
        },
    };
    parse(&body_to_ast_with(md, &opts)).unwrap()
}

/// The inline nodes of the first block.
fn inlines(root: &Node) -> &[Node] {
    root.children()[0].children()
}

fn link(url: &str, label: &str) -> Node {
    Node::Link {
        url: url.to_string(),
        title: String::new(),
        children: vec![Node::Text {
            value: label.to_string(),
        }],
    }
}

// ページ名が空の.mdへのリンクは、wikilinkにならない。linkのままである。
#[test]
fn a_link_to_a_md_file_without_a_name_stays_a_link() {
    let root = ast("[x](../.md)");
    assert_eq!(inlines(&root), [link("../.md", "x")]);
}

// 文字が#だけで、ページ名が空のリンクは、hashtagにならない。linkのままである。
#[test]
fn a_tag_link_to_a_md_file_without_a_name_stays_a_link() {
    let root = ast("[#](../.md)");
    assert_eq!(inlines(&root), [link("../.md", "#")]);
}

// ページ名が空で#idだけがあるリンクも、linkのままである。
#[test]
fn a_link_with_only_a_fragment_after_the_prefix_stays_a_link() {
    let root = ast("[x](./.md#id)");
    assert_eq!(inlines(&root), [link("./.md#id", "x")]);
}

// フォルダの中の、名前のある.mdへのリンクは、今までどおりwikilinkになる。
#[test]
fn a_link_to_a_named_md_file_in_a_folder_is_still_a_wikilink() {
    let root = ast("[x](../a/b.md)");
    assert_eq!(
        inlines(&root),
        [Node::Wikilink {
            url: "a/b".to_string(),
            children: vec![Node::Text {
                value: "x".to_string()
            }],
        }]
    );
}

// フォルダの中の、名前のない.mdへのリンクも、linkのままである。
#[test]
fn a_link_to_a_md_file_without_a_name_in_a_folder_stays_a_link() {
    let root = ast("[x](../a/.md)");
    assert_eq!(inlines(&root), [link("../a/.md", "x")]);
}

// 空白だけの名前は、名前ではない。空白だけの名前の.mdへのリンクも、linkのままである。
#[test]
fn a_link_to_a_md_file_with_only_whitespace_as_its_name_stays_a_link() {
    for (md, url) in [
        ("[x](<../ .md>)", "../ .md"),
        ("[x](<../a/  .md>)", "../a/  .md"),
        ("[# ](<../ .md>)", "../ .md"),
    ] {
        let root = ast(md);
        match inlines(&root) {
            [Node::Link { url: actual, .. }] => assert_eq!(actual, url, "{md:?}"),
            other => panic!("expected one link for {md:?}, got {other:?}"),
        }
    }
}
