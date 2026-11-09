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
