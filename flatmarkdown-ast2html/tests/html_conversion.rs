//! Parses Flat Markdown bodies and checks the HTML of their AST: the parser and the renderer
//! together.

use flatmarkdown::{BodyOptions, LinkResolution, parse_body, parse_body_with};
use flatmarkdown_ast2html::ast_to_html;

/// Parses `md` with the default options and renders its AST.
fn body_to_html(md: &str) -> String {
    ast_to_html(&parse_body(md))
}

/// Parses `md` with `opts` and renders its AST.
fn body_to_html_with(md: &str, opts: &BodyOptions) -> String {
    ast_to_html(&parse_body_with(md, opts))
}
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

// 段落は<p>になる。
#[test]
fn basic_paragraph() {
    let result = body_to_html("Hello, world!");
    assert_eq!(result, "<p>Hello, world!</p>\n");
}

// レベル1の見出しは<h1>になる。
#[test]
fn heading() {
    let result = body_to_html("# Title");
    assert_eq!(result, "<h1>Title</h1>\n");
}

// ~~text~~は<del>になる。
#[test]
fn strikethrough() {
    let result = body_to_html("~~deleted~~");
    assert_eq!(result, "<p><del>deleted</del></p>\n");
}

// タスクの項目は、無効にしたチェックボックスで始まる。済んだタスクにはcheckedがつく。
#[test]
fn tasklist() {
    let result = body_to_html("- [x] done\n- [ ] todo");
    assert_eq!(
        result,
        "<ul>\n<li><input type=\"checkbox\" checked disabled>done</li>\n\
         <li><input type=\"checkbox\" disabled>todo</li>\n</ul>\n"
    );
}

// 段落の中のただの改行は、HTMLでも改行のままである。<br />にはならない。
#[test]
fn softbreak_html() {
    let result = body_to_html("line1\nline2");
    assert_eq!(result, "<p>line1\nline2</p>\n");
}

// 行末の\は<br />になる。
#[test]
fn hardbreak_html() {
    let result = body_to_html("line1\\\nline2");
    assert_eq!(result, "<p>line1<br />\nline2</p>\n");
}

// >>>で囲んだ複数行の引用は<blockquote>になる。
#[test]
fn multiline_block_quote_html() {
    let result = body_to_html(">>>\nq1\n\nq2\n>>>");
    assert_eq!(
        result,
        "<blockquote>\n<p>q1</p>\n<p>q2</p>\n</blockquote>\n"
    );
}

// wikilinkは、data-wikilink="true"のついた<a>になる。
#[test]
fn wikilink_html_basic() {
    let result = body_to_html("[[page]]");
    assert_eq!(
        result,
        "<p><a href=\"page\" data-wikilink=\"true\">page</a></p>\n"
    );
}

// ラベルのあるwikilinkは、hrefがurlで、文字がラベルの<a>になる。
#[test]
fn wikilink_html_with_label() {
    let result = body_to_html("[[url|link label]]");
    assert_eq!(
        result,
        "<p><a href=\"url\" data-wikilink=\"true\">link label</a></p>\n"
    );
}

// コードフェンスの言語名は、class属性の中でエスケープされる。
#[test]
fn html_code_fence_info_is_escaped_in_the_class_attribute() {
    let result = body_to_html("```a\"><b>\ncode\n```");
    assert_eq!(
        result,
        "<pre><code class=\"language-a&quot;&gt;&lt;b&gt;\">code\n</code></pre>\n"
    );
}

// アラートの題名は、文字としてエスケープされる。
#[test]
fn html_alert_title_is_escaped_as_text() {
    let result = body_to_html("> [!NOTE] a < b & c\n> body");
    assert_eq!(
        result,
        "<blockquote class=\"alert alert-note\">\n<p class=\"alert-title\">a &lt; b &amp; c</p>\n<p>body</p>\n</blockquote>\n"
    );
}

// 画像のaltには、子の文字だけが入る。子のタグは入らない。
#[test]
fn html_image_alt_holds_only_the_text_of_its_children() {
    let result = body_to_html("![a **b** [c](d) <e>](u.png)");
    assert_eq!(
        result,
        "<p><img src=\"u.png\" alt=\"a b c &lt;e&gt;\" /></p>\n"
    );
}

// 画像のaltには、コードの文字とハッシュタグも入る。改行は空白になる。
#[test]
fn html_image_alt_holds_code_and_hashtags_and_turns_a_line_break_into_a_space() {
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["./".into()],
        },
    };
    let result = body_to_html_with("![a `b`\nc #t](u.png)", &opts);
    assert_eq!(result, "<p><img src=\"u.png\" alt=\"a b c #t\" /></p>\n");
}

// 本文の生のHTMLは、そのまま出る。自分のノートを描くためのものなので、取り除かない。
#[test]
fn html_raw_html_in_the_body_is_written_as_it_is() {
    assert_eq!(
        body_to_html("<script>alert(1)</script>"),
        "<script>alert(1)</script>"
    );
    assert_eq!(
        body_to_html("a <img src=x onerror=alert(1)> b"),
        "<p>a <img src=x onerror=alert(1)> b</p>\n"
    );
}

// リンクのURLは調べない。javascript:のリンクも、リンクのまま出る。
#[test]
fn html_link_url_is_not_checked() {
    assert_eq!(
        body_to_html("[a](javascript:alert(1))"),
        "<p><a href=\"javascript:alert(1)\">a</a></p>\n"
    );
}

// 1本の|で囲んだ文字は、HTMLでも|が残る。
#[test]
fn text_between_single_pipes_keeps_its_pipes_in_html() {
    assert_eq!(body_to_html("a |b| c"), "<p>a |b| c</p>\n");
}

// どの場所の\#も、置き換え用の文字をHTMLに残さない。
#[test]
fn no_placeholder_character_is_left_in_the_html() {
    let md = "`\\#` [x](u\\#f \"\\#\") ![a](p\\#q) $`\\#`$ <b>\\#</b> \\#t\n\n```\\#\n\\#\n```\n\n<div>\n\\#\n</div>\n\n> [!TIP] \\#\n> \\#";
    assert!(!body_to_html(md).contains('\u{FDD0}'));
}

// HTMLでも、コードの中の\#はそのまま出る。
#[test]
fn html_shows_an_escaped_hash_in_code_as_written() {
    assert_eq!(body_to_html(r"`a \# b`"), "<p><code>a \\# b</code></p>\n");
}
