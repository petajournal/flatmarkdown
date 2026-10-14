use flatmarkdown::{body_to_html, body_to_html_with, page_to_ast, BodyOptions, LinkResolution};

#[test]
fn paragraph_with_bold_italic_code() {
    let input = "Hello **world** and _italic_ with `code`";
    let html = body_to_html(input);
    assert_eq!(
        html,
        "<p>Hello <strong>world</strong> and <em>italic</em> with <code>code</code></p>\n"
    );
}

#[test]
fn handle_empty_lines() {
    let input = "```rust\nlet x = 1;\n\nlet y = 2;\n```\n\nAfter the block.";
    let html = body_to_html(input);
    assert_eq!(
        html,
        "<pre><code class=\"language-rust\">let x = 1;\n\nlet y = 2;\n</code></pre>\n<p>After the block.</p>\n"
    );
}

#[test]
fn resolve_links_with_prefix() {
    let input = "See [page](../docs/page.md) and [journal](../journals/2024-01-01.md) and [other](../other/page.md).";
    let opts = BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../docs/".into(), "../journals/".into()] } };
    let html = body_to_html_with(input, &opts);
    assert_eq!(
        html,
        "<p>See <a href=\"page\" data-wikilink=\"true\">page</a> and <a href=\"2024-01-01\" data-wikilink=\"true\">journal</a> and <a href=\"../other/page.md\">other</a>.</p>\n"
    );
}

#[test]
fn page_items_with_metadata() {
    let input = "<!-- @@ 2026-09-27-15-00-16-W3N8ASY plan=\"2026-09-28\" -->\n- [ ] Task1\n\nPlain paragraph\n";
    let v: serde_json::Value = serde_json::from_str(&page_to_ast(input)).unwrap();
    let items = v["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["level"], 2);
    assert_eq!(items[0]["id"], "2026-09-27-15-00-16-W3N8ASY");
    assert_eq!(items[0]["properties"]["plan"], "2026-09-28");
    assert_eq!(items[0]["body"], "- [ ] Task1");
    assert_eq!(items[1]["level"], 1);
    assert_eq!(items[1]["body"], "Plain paragraph");
}

#[test]
fn footnote_reference_and_definition_in_separate_bodies() {
    assert_eq!(
        body_to_html("Text[^n]."),
        "<p>Text<sup id=\"fnref-n\" class=\"footnote-ref\"><a href=\"#fn-n\">[n]</a></sup>.</p>\n"
    );
    assert_eq!(
        body_to_html("[^n]: Note."),
        "<div id=\"fn-n\" class=\"footnote\">\n<a href=\"#fnref-n\" class=\"footnote-label\">[n]:</a>\n<p>Note.</p>\n</div>\n"
    );
}
