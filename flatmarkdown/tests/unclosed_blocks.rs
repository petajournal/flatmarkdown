//! A block that opens with a line and closes with a line runs to the end of the page while it
//! is not closed. These tests check that such a block in an item body is found, and closed.

use flatmarkdown::has_unclosed_block;

const FENCE: &str = "```";

// 開始記号だけのコードブロックは、閉じていない。閉じたコードブロックは、閉じている。
#[test]
fn a_code_block_without_its_closing_fence_is_unclosed() {
    assert!(has_unclosed_block(&format!("{FENCE}\nabc")));
    assert!(!has_unclosed_block(&format!("{FENCE}\nabc\n{FENCE}")));
}

// 行で開いて行で閉じるブロックは、どれも閉じていなければ見つかる。コードブロック、数式のブロック、複数行の引用、HTMLのブロックである。
#[test]
fn every_block_that_closes_with_a_line_is_found_unclosed() {
    for body in [
        "~~~\nabc".to_string(),
        format!("{FENCE}math\nx^2"),
        format!("````\nabc\n{FENCE}"),
        ">>>\nabc".to_string(),
        "<!--\nabc".to_string(),
        "<pre>\nabc".to_string(),
        "<script>\nabc".to_string(),
        "<?\nabc".to_string(),
        "<![CDATA[\nabc".to_string(),
        format!(">>>\n{FENCE}\nabc"),
    ] {
        assert!(has_unclosed_block(&body), "{body:?}");
    }
}

// 開始記号は、本文のどの行にあってもよい。最後の行の```だけや、リストの後ろの0桁目の```も、閉じていない。
#[test]
fn an_opening_line_anywhere_in_the_body_is_found() {
    assert!(has_unclosed_block(&format!("abc\n{FENCE}")));
    assert!(has_unclosed_block(&format!(
        "- a\n  {FENCE}\n  abc\n{FENCE}"
    )));
}

// 閉じたブロック、終了記号だけの本文、終了記号のない入れ物の中のブロック、空行で終わるHTMLのブロックは、閉じていないとしない。
#[test]
fn a_closed_block_and_a_lone_closing_line_are_not_unclosed() {
    for body in [
        "".to_string(),
        "abc".to_string(),
        format!("{FENCE}rust\nfn main() {{}}\n{FENCE}"),
        ">>>\nabc\n>>>".to_string(),
        format!(">>>\n{FENCE}\nabc\n{FENCE}\n>>>"),
        "<pre>abc</pre>".to_string(),
        "abc\n-->".to_string(),
        "abc\n</pre>".to_string(),
        format!("- a\n  {FENCE}\n  abc"),
        format!("> {FENCE}\n> abc"),
        "<div>\nabc".to_string(),
        "---\nabc".to_string(),
    ] {
        assert!(!has_unclosed_block(&body), "{body:?}");
    }
}
