use flatmarkdown::{BodyOptions, LinkResolution, body_to_html, body_to_html_with, page_to_ast};

// 太字、斜体、コードを含む段落は、<strong>、<em>、<code>になる。
#[test]
fn paragraph_with_bold_italic_code() {
    let input = "Hello **world** and _italic_ with `code`";
    let html = body_to_html(input);
    assert_eq!(
        html,
        "<p>Hello <strong>world</strong> and <em>italic</em> with <code>code</code></p>\n"
    );
}

// コードブロックの中の空行は残る。あとの段落は別のブロックになる。
#[test]
fn handle_empty_lines() {
    let input = "```rust\nlet x = 1;\n\nlet y = 2;\n```\n\nAfter the block.";
    let html = body_to_html(input);
    assert_eq!(
        html,
        "<pre><code class=\"language-rust\">let x = 1;\n\nlet y = 2;\n</code></pre>\n<p>After the block.</p>\n"
    );
}

// 接頭辞に合う.mdへのリンクだけがwikilinkになる。合わないリンクは、そのままである。
#[test]
fn resolve_links_with_prefix() {
    let input = "See [page](../docs/page.md) and [journal](../journals/2024-01-01.md) and [other](../other/page.md).";
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["../docs/".into(), "../journals/".into()],
        },
    };
    let html = body_to_html_with(input, &opts);
    assert_eq!(
        html,
        "<p>See <a href=\"page\" data-wikilink=\"true\">page</a> and <a href=\"2024-01-01\" data-wikilink=\"true\">journal</a> and <a href=\"../other/page.md\">other</a>.</p>\n"
    );
}

// メタデータ行は、次のブロックにレベル、ID、プロパティを与える。メタデータ行のないブロックはレベル1である。
#[test]
fn page_items_with_metadata() {
    let input = "<!-- @@ 2026-09-27-15-00-16-W3N8ASY plan=\"2026-09-28\" -->\n- [ ] Task1\n\nPlain paragraph\n";
    let v: serde_json::Value = serde_json::from_str(&page_to_ast(input).to_json()).unwrap();
    let items = v["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["level"], 2);
    assert_eq!(items[0]["id"], "2026-09-27-15-00-16-W3N8ASY");
    assert_eq!(items[0]["properties"]["plan"], "2026-09-28");
    assert_eq!(items[0]["body"], "- [ ] Task1");
    assert_eq!(items[1]["level"], 1);
    assert_eq!(items[1]["body"], "Plain paragraph");
}

// 脚注の参照と定義は、別々の本文にあっても、互いを指すリンクになる。
#[test]
fn footnote_reference_and_definition_in_separate_bodies() {
    assert_eq!(
        body_to_html("Text[^n]."),
        "<p>Text<sup id=\"fnref-n\" class=\"footnote-ref\"><a href=\"#fn-n\">[n]</a></sup>.</p>\n"
    );
    assert_eq!(
        body_to_html("[^n]: Note."),
        "<div id=\"fn-n\" class=\"footnote\">\n<p><a href=\"#fnref-n\" class=\"footnote-label\">[n]:</a> Note.</p>\n</div>\n"
    );
}

// リストで始まる脚注の定義では、ラベルはリストの前に出る。
#[test]
fn footnote_definition_starting_with_a_list_keeps_the_label_in_front() {
    assert_eq!(
        body_to_html("[^n]:\n    - item"),
        "<div id=\"fn-n\" class=\"footnote\">\n<a href=\"#fnref-n\" class=\"footnote-label\">[n]:</a>\n<ul>\n<li>item</li>\n</ul>\n</div>\n"
    );
}

// インライン脚注は、名前の__inline_1ではなく、番号の1を表示する。idとhrefは名前のままである。
#[test]
fn an_inline_footnote_shows_its_number_instead_of_its_name() {
    assert_eq!(
        body_to_html("Hi^[a note]."),
        concat!(
            "<p>Hi<sup id=\"fnref-__inline_1\" class=\"footnote-ref\">",
            "<a href=\"#fn-__inline_1\">[1]</a></sup>.</p>\n",
            "<div id=\"fn-__inline_1\" class=\"footnote\">\n",
            "<p><a href=\"#fnref-__inline_1\" class=\"footnote-label\">[1]:</a> a note</p>\n",
            "</div>\n"
        )
    );
}

// 2つ目のインライン脚注は、2を表示する。
#[test]
fn the_second_inline_footnote_shows_two() {
    let html = body_to_html("A^[x] B^[y]");
    assert!(
        html.contains("<a href=\"#fn-__inline_2\">[2]</a>"),
        "{html}"
    );
    assert!(
        html.contains("<a href=\"#fnref-__inline_2\" class=\"footnote-label\">[2]:</a>"),
        "{html}"
    );
}

// 同じ脚注を2回参照すると、2回目からのidにはref_numが付く。idが重ならないためである。
#[test]
fn the_second_reference_to_a_footnote_has_its_number_in_the_id() {
    assert_eq!(
        body_to_html("A[^x] B[^x] C[^x]"),
        concat!(
            "<p>A<sup id=\"fnref-x\" class=\"footnote-ref\"><a href=\"#fn-x\">[x]</a></sup>",
            " B<sup id=\"fnref-x-2\" class=\"footnote-ref\"><a href=\"#fn-x\">[x]</a></sup>",
            " C<sup id=\"fnref-x-3\" class=\"footnote-ref\"><a href=\"#fn-x\">[x]</a></sup></p>\n"
        )
    );
}
