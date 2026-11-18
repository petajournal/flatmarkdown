//! The hrefs of wikilinks and hashtags, from the table in `hrefs.json`.
//!
//! The same table is read by the tests of the npm package, and a copy of it by the tests of
//! `flatmarkdown-ast2react`. So the three renderers give the same hrefs.

use flatmarkdown_ast::Node;
use flatmarkdown_ast2html::{HtmlOptions, ast_to_html_with};
use serde::Deserialize;

#[derive(Deserialize)]
struct Table {
    cases: Vec<Case>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Case {
    kind: String,
    template: String,
    fragment_pattern: String,
    name: String,
    href: String,
}

fn table() -> Vec<Case> {
    let json = include_str!("hrefs.json");
    serde_json::from_str::<Table>(json).unwrap().cases
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;")
}

/// The href that `ast_to_html_with` writes for the case.
fn rendered_href(case: &Case) -> String {
    let text = vec![Node::Text {
        value: "x".to_string(),
    }];
    let (node, options) = match case.kind.as_str() {
        "wikilink" => (
            Node::Wikilink {
                url: case.name.clone(),
                children: text,
            },
            HtmlOptions {
                wikilink_href: case.template.clone(),
                wikilink_fragment_pattern: case.fragment_pattern.clone(),
                ..HtmlOptions::default()
            },
        ),
        "hashtag" => (
            Node::Hashtag {
                value: case.name.clone(),
            },
            HtmlOptions {
                hashtag_href: case.template.clone(),
                ..HtmlOptions::default()
            },
        ),
        kind => panic!("unknown kind {kind}"),
    };
    let html = ast_to_html_with(&node, &options);
    let start = html.find("href=\"").unwrap() + "href=\"".len();
    let end = start + html[start..].find('"').unwrap();
    html[start..end].to_string()
}

// 表のどの行でも、描いたhrefは、表のhrefをHTMLのエスケープにかけたものである。
#[test]
fn every_href_of_the_table_is_rendered_escaped_for_html() {
    let wrong: Vec<String> = table()
        .iter()
        .filter(|case| rendered_href(case) != escape_html(&case.href))
        .map(|case| format!("{case:?} gave {}", rendered_href(case)))
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
