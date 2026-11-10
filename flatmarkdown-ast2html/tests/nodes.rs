//! Renders nodes that are built by hand or read from JSON. No Markdown parser is used.

use flatmarkdown_ast::{Node, parse};
use flatmarkdown_ast2html::ast_to_html;

// JSONから読んだASTは、パーサーなしでHTMLになる。
#[test]
fn an_ast_read_from_json_is_rendered_without_a_parser() {
    let ast = parse(
        r#"{"type":"document","children":[{"type":"paragraph","children":[
            {"type":"text","value":"Hello "},
            {"type":"strong","children":[{"type":"text","value":"world"}]}]}]}"#,
    )
    .unwrap();
    assert_eq!(ast_to_html(&ast), "<p>Hello <strong>world</strong></p>\n");
}

// 文書の根でないノードも、そのノードだけのHTMLになる。
#[test]
fn a_node_that_is_not_a_document_is_rendered_alone() {
    let node = Node::Emph {
        children: vec![Node::Text { value: "a".into() }],
    };
    assert_eq!(ast_to_html(&node), "<em>a</em>");
}

// 知らない種類のノードは、タグを出さずに、子だけをHTMLにする。
#[test]
fn an_unknown_node_is_rendered_as_its_children_only() {
    let ast =
        parse(r#"{"type":"future_node","children":[{"type":"text","value":"a < b"}]}"#).unwrap();
    assert_eq!(ast_to_html(&ast), "a &lt; b");
}

// ハッシュタグの名前は、hrefでも表示する文字でもエスケープされる。
#[test]
fn html_hashtag_name_is_escaped_in_the_href_and_in_the_text() {
    let node = Node::Hashtag {
        value: "a<b>&\"c".to_string(),
    };
    assert_eq!(
        ast_to_html(&node),
        "<a href=\"a&lt;b&gt;&amp;&quot;c\" class=\"hashtag\">#a&lt;b&gt;&amp;&quot;c</a>"
    );
}
