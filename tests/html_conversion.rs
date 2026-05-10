use flatmarkdown::{body_to_html, body_to_html_with, BodyOptions, LinkResolution};

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
        "<pre><code class=\"language-rust\">let x = 1;\n\nlet y = 2;\n</code></pre>\n<br />\nAfter the block."
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
