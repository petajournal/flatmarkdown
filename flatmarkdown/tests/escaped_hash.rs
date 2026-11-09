//! `\#` is written to keep a `#` from starting a hashtag. These tests check what `\#` becomes
//! in each place of a body: a plain `#` where Markdown reads backslash escapes, and `\#` as it
//! is where Markdown keeps the text raw (code, math, and raw HTML).

use flatmarkdown::{BodyOptions, LinkResolution, body_to_ast, body_to_ast_with, body_to_html};
use serde_json::Value;

fn ast(md: &str) -> Value {
    serde_json::from_str(&body_to_ast(md)).unwrap()
}

/// The first inline node of the first block.
fn first_inline(md: &str) -> Value {
    ast(md)["children"][0]["children"][0].clone()
}

// インラインコードの中の\#は、そのまま残る。
#[test]
fn an_escaped_hash_in_a_code_span_stays_as_written() {
    let code = first_inline(r"`a \# b`");
    assert_eq!(code["type"], "code");
    assert_eq!(code["literal"], r"a \# b");
}

// コードブロックの中の\#は、そのまま残る。
#[test]
fn an_escaped_hash_in_a_code_block_stays_as_written() {
    let block = ast("```sh\necho \\# not a comment\n```")["children"][0].clone();
    assert_eq!(block["type"], "code_block");
    assert_eq!(block["literal"], "echo \\# not a comment\n");
}

// コードブロックの言語名の\#は、#になる。
#[test]
fn an_escaped_hash_in_the_info_of_a_code_block_becomes_a_hash() {
    let block = ast("```c\\#\nx\n```")["children"][0].clone();
    assert_eq!(block["info"], "c#");
}

// 数式の中の\#は、そのまま残る。
#[test]
fn an_escaped_hash_in_math_stays_as_written() {
    let math = first_inline(r"$`a \# b`$");
    assert_eq!(math["type"], "math");
    assert_eq!(math["literal"], r"a \# b");
}

// 生のHTMLの中の\#は、インラインでもブロックでも、そのまま残る。
#[test]
fn an_escaped_hash_in_raw_html_stays_as_written() {
    let inline = first_inline(r#"<a href="x\#y">t</a>"#);
    assert_eq!(inline["type"], "html_inline");
    assert_eq!(inline["value"], r#"<a href="x\#y">"#);

    let block = ast("<div>\n\\#\n</div>")["children"][0].clone();
    assert_eq!(block["type"], "html_block");
    assert_eq!(block["literal"], "<div>\n\\#\n</div>\n");
}

// リンクのURLと題名の\#は、#になる。
#[test]
fn an_escaped_hash_in_the_url_and_title_of_a_link_becomes_a_hash() {
    let link = first_inline(r#"[x](u\#f "t \#1")"#);
    assert_eq!(link["type"], "link");
    assert_eq!(link["url"], "u#f");
    assert_eq!(link["title"], "t #1");
}

// 画像のURLの\#は、#になる。プロパティの区切りにはならない。
#[test]
fn an_escaped_hash_in_the_url_of_an_image_becomes_a_hash_and_starts_no_props() {
    let embed = first_inline(r"![a](p.png\#w=10)");
    assert_eq!(embed["type"], "embed");
    assert_eq!(embed["url"], "p.png#w=10");
    assert!(embed.get("props").is_none());
}

// アラートの題名の\#は、#になる。
#[test]
fn an_escaped_hash_in_the_title_of_an_alert_becomes_a_hash() {
    let alert = ast("> [!NOTE] Issue \\#12\n> body")["children"][0].clone();
    assert_eq!(alert["type"], "alert");
    assert_eq!(alert["title"], "Issue #12");
}

// リンクの解決が有効でも、コードの中の\#はそのまま残り、文字の\#はハッシュタグにならない。
#[test]
fn with_link_resolution_an_escaped_hash_is_no_hashtag_and_code_stays_as_written() {
    let opts = BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["./".into()],
        },
    };
    let v: Value = serde_json::from_str(&body_to_ast_with(r"\#no `\#code`", &opts)).unwrap();
    let inlines = v["children"][0]["children"].as_array().unwrap();
    assert_eq!(inlines[0]["type"], "text");
    assert_eq!(inlines[0]["value"], "#no ");
    assert_eq!(inlines[1]["type"], "code");
    assert_eq!(inlines[1]["literal"], r"\#code");
}

// どの場所の\#も、置き換え用の文字を出力に残さない。
#[test]
fn no_placeholder_character_is_left_in_the_output() {
    let md = "`\\#` [x](u\\#f \"\\#\") ![a](p\\#q) $`\\#`$ <b>\\#</b> \\#t\n\n```\\#\n\\#\n```\n\n<div>\n\\#\n</div>\n\n> [!TIP] \\#\n> \\#";
    assert!(!body_to_ast(md).contains('\u{FDD0}'));
    assert!(!body_to_html(md).contains('\u{FDD0}'));
}

// HTMLでも、コードの中の\#はそのまま出る。
#[test]
fn html_shows_an_escaped_hash_in_code_as_written() {
    assert_eq!(body_to_html(r"`a \# b`"), "<p><code>a \\# b</code></p>\n");
}
