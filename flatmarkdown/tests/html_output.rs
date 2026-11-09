//! The HTML that `body_to_html` writes for each kind of node. Every test compares the whole
//! output, so a change of one character is noticed.

use flatmarkdown::{BodyOptions, LinkResolution, body_to_html, body_to_html_with};

// ── Lists ──

// 項目の間に空行のない箇条書きは、項目をpで包まない。
#[test]
fn a_tight_bullet_list_has_no_paragraph_elements() {
    assert_eq!(
        body_to_html("- a\n- b"),
        "<ul>\n<li>a</li>\n<li>b</li>\n</ul>\n"
    );
}

// 項目の間に空行のある箇条書きは、項目をpで包む。
#[test]
fn a_loose_bullet_list_wraps_each_item_in_a_paragraph_element() {
    assert_eq!(
        body_to_html("- a\n\n- b"),
        "<ul>\n<li><p>a</p>\n</li>\n<li><p>b</p>\n</li>\n</ul>\n"
    );
}

// 1から始まる番号付きリストは、start属性のないolになる。
#[test]
fn an_ordered_list_that_starts_at_one_has_no_start_attribute() {
    assert_eq!(
        body_to_html("1. a\n2. b"),
        "<ol>\n<li>a</li>\n<li>b</li>\n</ol>\n"
    );
}

// 1以外から始まる番号付きリストは、start属性を持つ。
#[test]
fn an_ordered_list_that_starts_elsewhere_has_a_start_attribute() {
    assert_eq!(
        body_to_html("3. a\n4. b"),
        "<ol start=\"3\">\n<li>a</li>\n<li>b</li>\n</ol>\n"
    );
}

// タイトなリストでpが外れるのは、項目の直下の段落だけである。項目の中の引用の段落はpで包まれる。
#[test]
fn a_paragraph_in_a_block_quote_of_a_tight_item_keeps_its_paragraph_element() {
    assert_eq!(
        body_to_html("- a\n  > q"),
        "<ul>\n<li>a<blockquote>\n<p>q</p>\n</blockquote>\n</li>\n</ul>\n"
    );
}

// ── Tables ──

// 表は、見出しの行がtheadに、ほかの行がtbodyに入る。セルの中の書式も描かれる。
#[test]
fn a_table_has_its_header_row_in_thead_and_the_other_rows_in_tbody() {
    assert_eq!(
        body_to_html("| a | **b** |\n|---|---|\n| 1 | `2` |"),
        "<table>\n<thead>\n<tr><th>a</th><th><strong>b</strong></th></tr>\n</thead>\n\
         <tbody>\n<tr><td>1</td><td><code>2</code></td></tr>\n</tbody>\n</table>\n"
    );
}

// 見出しの行だけの表は、tbodyを持たない。
#[test]
fn a_table_with_only_a_header_row_has_no_tbody() {
    assert_eq!(
        body_to_html("| a | b |\n|---|---|"),
        "<table>\n<thead>\n<tr><th>a</th><th>b</th></tr>\n</thead>\n</table>\n"
    );
}

// ── Code blocks ──

// 言語名のないフェンスのコードブロックは、class属性を持たない。
#[test]
fn a_fenced_code_block_without_a_language_has_no_class() {
    assert_eq!(body_to_html("```\nx\n```"), "<pre><code>x\n</code></pre>\n");
}

// インデントのコードブロックは、class属性を持たない。
#[test]
fn an_indented_code_block_has_no_class() {
    assert_eq!(body_to_html("    x"), "<pre><code>x\n</code></pre>\n");
}

// 言語名がtextのコードブロックは、class属性を持たない。言語名のないフェンスと同じ扱いである。
#[test]
fn a_code_block_with_the_language_text_has_no_class() {
    assert_eq!(
        body_to_html("```text\nx\n```"),
        "<pre><code>x\n</code></pre>\n"
    );
}

// ── Alerts ──

// 題名のないアラートは、種類ごとの既定の題名を持つ。
#[test]
fn an_alert_without_a_title_gets_the_default_title_of_its_type() {
    for (marker, class, title) in [
        ("NOTE", "note", "Note"),
        ("TIP", "tip", "Tip"),
        ("IMPORTANT", "important", "Important"),
        ("WARNING", "warning", "Warning"),
        ("CAUTION", "caution", "Caution"),
    ] {
        assert_eq!(
            body_to_html(&format!("> [!{marker}]\n> x")),
            format!(
                "<blockquote class=\"alert alert-{class}\">\n\
                 <p class=\"alert-title\">{title}</p>\n<p>x</p>\n</blockquote>\n"
            )
        );
    }
}

// 題名のあるアラートは、その題名を持つ。
#[test]
fn an_alert_with_a_title_shows_that_title() {
    assert_eq!(
        body_to_html("> [!TIP] Custom\n> x"),
        "<blockquote class=\"alert alert-tip\">\n\
         <p class=\"alert-title\">Custom</p>\n<p>x</p>\n</blockquote>\n"
    );
}

// ── Other blocks ──

// 水平線はhrになる。
#[test]
fn a_thematic_break_is_an_hr() {
    assert_eq!(body_to_html("---"), "<hr />\n");
}

// subtextはsubになる。
#[test]
fn a_subtext_is_a_sub_element() {
    assert_eq!(body_to_html("-# small"), "<sub>small</sub>");
}

// 引用はblockquoteになる。
#[test]
fn a_block_quote_is_a_blockquote() {
    assert_eq!(
        body_to_html("> q"),
        "<blockquote>\n<p>q</p>\n</blockquote>\n"
    );
}

// ── Inline markup ──

// 下線、ハイライト、上付き、下付き、スポイラーは、それぞれの要素になる。
#[test]
fn the_extended_inline_markup_becomes_its_own_element() {
    assert_eq!(body_to_html("__u__"), "<p><u>u</u></p>\n");
    assert_eq!(body_to_html("==h=="), "<p><mark>h</mark></p>\n");
    assert_eq!(body_to_html("x^2^"), "<p>x<sup>2</sup></p>\n");
    assert_eq!(body_to_html("H~2~O"), "<p>H<sub>2</sub>O</p>\n");
    assert_eq!(
        body_to_html("||s||"),
        "<p><span class=\"spoiler\">s</span></p>\n"
    );
}

// 絵文字のショートコードは、絵文字の文字になる。
#[test]
fn a_shortcode_becomes_its_emoji() {
    assert_eq!(body_to_html(":rabbit:"), "<p>🐰</p>\n");
}

// 数式は、classがmathのspanになる。中身はエスケープされる。
#[test]
fn math_is_a_span_with_the_class_math_and_escaped_content() {
    assert_eq!(
        body_to_html("$`a<b`$"),
        "<p><span class=\"math\">a&lt;b</span></p>\n"
    );
}

// リンクの解決が有効なとき、ハッシュタグは、classがhashtagのリンクになる。
#[test]
fn with_link_resolution_a_hashtag_is_a_link_with_the_class_hashtag() {
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["./".to_string()],
        },
    };
    assert_eq!(
        body_to_html_with("see #diary", &opts),
        "<p>see <a href=\"diary\" class=\"hashtag\">#diary</a></p>\n"
    );
}

// リンクの解決が無効なとき、#で始まる語はただの文字である。
#[test]
fn without_link_resolution_a_hash_word_is_plain_text() {
    assert_eq!(body_to_html("see #diary"), "<p>see #diary</p>\n");
}

// 文字の中の<、>、&は、エスケープされる。
#[test]
fn special_characters_in_text_are_escaped() {
    assert_eq!(
        body_to_html("a < b & c > d"),
        "<p>a &lt; b &amp; c &gt; d</p>\n"
    );
}
