//! A hashtag is not made inside the label of a link or a wikilink. A renderer draws a hashtag
//! as a link, so a hashtag in a label would be a link inside a link.

use flatmarkdown::{BodyOptions, LinkResolution, parse_body_with};
use flatmarkdown_ast::Node;

/// Parses with link resolution on, which also turns hashtags on.
fn ast(md: &str) -> Node {
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["../".to_string(), "./".to_string()],
        },
    };
    parse_body_with(md, &opts)
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

// wikilinkのラベルの中の#tagは、ハッシュタグにならない。
#[test]
fn a_hashtag_is_not_made_in_the_label_of_a_wikilink() {
    let root = ast("[[page|#tag]]");
    assert_eq!(
        inlines(&root),
        [Node::Wikilink {
            url: "page".to_string(),
            children: vec![text("#tag")],
        }]
    );
}

// ラベルの中の書式の、さらに中にある#tagも、ハッシュタグにならない。
#[test]
fn a_hashtag_is_not_made_inside_markup_in_a_label() {
    let root = ast("[**#tag**](https://example.com)");
    assert_eq!(
        inlines(&root),
        [Node::Link {
            url: "https://example.com".to_string(),
            title: String::new(),
            children: vec![Node::Strong {
                children: vec![text("#tag")]
            }],
        }]
    );
}

// .mdへのリンクから作ったwikilinkでも、ラベルの中の#tagはハッシュタグにならない。
#[test]
fn a_hashtag_is_not_made_in_the_label_of_a_wikilink_made_from_a_link() {
    let root = ast("[#a](../b.md)");
    assert_eq!(
        inlines(&root),
        [Node::Wikilink {
            url: "b".to_string(),
            children: vec![text("#a")],
        }]
    );
}

// #idのついたタグのリンクは、タグのリンクではない。wikilinkになり、ラベルは文字のままである。
#[test]
fn a_tag_link_with_a_fragment_is_a_wikilink_with_a_text_label() {
    let root = ast("[#tag](./tag.md#id)");
    assert_eq!(
        inlines(&root),
        [Node::Wikilink {
            url: "tag#id".to_string(),
            children: vec![text("#tag")],
        }]
    );
}

// リンクの前後の#tagは、今までどおりハッシュタグになる。
#[test]
fn hashtags_before_and_after_a_link_are_still_hashtags() {
    let root = ast("#a [x](https://example.com) #b");
    let types: Vec<&str> = inlines(&root).iter().map(Node::type_name).collect();
    assert_eq!(types, ["hashtag", "text", "link", "text", "hashtag"]);
}
