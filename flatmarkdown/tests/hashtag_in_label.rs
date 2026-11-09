//! A hashtag is not made inside the label of a link or a wikilink. A renderer draws a hashtag
//! as a link, so a hashtag in a label would be a link inside a link.

use flatmarkdown::{BodyOptions, LinkResolution, body_to_ast_with};
use flatmarkdown_ast::{Node, parse};

/// Parses with link resolution on, which also turns hashtags on.
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

fn text(value: &str) -> Node {
    Node::Text {
        value: value.to_string(),
    }
}

// リンクのラベルの中の#tagは、ハッシュタグにならない。文字のままである。
#[test]
fn a_hashtag_is_not_made_in_the_label_of_a_link() {
    let root = ast("[#tag x](https://example.com)");
    assert_eq!(
        inlines(&root),
        [Node::Link {
            url: "https://example.com".to_string(),
            title: String::new(),
            children: vec![text("#tag x")],
        }]
    );
}
