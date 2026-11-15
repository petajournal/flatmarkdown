//! The HTML that `ast_to_html_with` writes when the options differ from the default.

use flatmarkdown::{BodyOptions, parse_body_with};
use flatmarkdown_ast::Node;
use flatmarkdown_ast2html::{HtmlOptions, ast_to_html, ast_to_html_with};

fn hashtag(name: &str) -> Node {
    Node::Hashtag {
        value: name.to_string(),
    }
}

fn hashtag_href(template: &str) -> HtmlOptions {
    HtmlOptions {
        hashtag_href: template.to_string(),
    }
}

// 既定の設定では、ast_to_htmlと同じHTMLになる。
#[test]
fn the_default_options_give_the_same_html_as_ast_to_html() {
    let node = hashtag("Rust");
    assert_eq!(
        ast_to_html_with(&node, &HtmlOptions::default()),
        ast_to_html(&node)
    );
}

// ハッシュタグのhrefは、テンプレートの{}をタグ名に置き換えたものである。表示する文字は変わらない。
#[test]
fn the_href_of_a_hashtag_is_the_template_with_the_tag_name_in_place_of_the_braces() {
    assert_eq!(
        ast_to_html_with(&hashtag("Rust"), &hashtag_href("/tags/{}.html")),
        "<a href=\"/tags/Rust.html\" class=\"hashtag\">#Rust</a>"
    );
}

// テンプレートの文字も、hrefの中でエスケープされる。
#[test]
fn the_characters_of_the_template_are_escaped_in_the_href() {
    assert_eq!(
        ast_to_html_with(&hashtag("Rust"), &hashtag_href("/search?tag={}&lang=ja")),
        "<a href=\"/search?tag=Rust&amp;lang=ja\" class=\"hashtag\">#Rust</a>"
    );
}

// 設定は、リスト、表、脚注、アラートの中のハッシュタグにも届く。
#[test]
fn the_options_reach_hashtags_inside_lists_tables_footnotes_and_alerts() {
    let md = "- #a\n\n| h |\n|---|\n| #b |\n\n[^n]: #c\n\n> [!NOTE]\n> #d";
    let ast = parse_body_with(md, &BodyOptions::relative_page_links());
    let html = ast_to_html_with(&ast, &hashtag_href("/tags/{}"));
    for name in ["a", "b", "c", "d"] {
        assert!(
            html.contains(&format!("<a href=\"/tags/{name}\" class=\"hashtag\">")),
            "#{name} does not link to its tag page in:\n{html}"
        );
    }
}
