//! A tag link (`[#name](<prefix>name.md)`) becomes a hashtag only when the name follows the
//! rules of a tag name. The name of a `.md` file can come from outside, so a link must not
//! make a hashtag that could not be written as `#name` in text.

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

fn text(value: &str) -> Node {
    Node::Text {
        value: value.to_string(),
    }
}

fn wikilink(url: &str, label: &str) -> Node {
    Node::Wikilink {
        url: url.to_string(),
        children: vec![text(label)],
    }
}

// 名前に空白のあるタグのリンクは、hashtagにならない。ふつうのwikilinkになり、ラベルは文字のままである。
#[test]
fn a_tag_link_with_a_space_in_its_name_is_a_wikilink() {
    let root = ast("[#a b](<../a b.md>)");
    assert_eq!(inlines(&root), [wikilink("a b", "#a b")]);
}
