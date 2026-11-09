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
