//! A block that opens with a line and closes with a line runs to the end of the page while it
//! is not closed. These tests check that such a block in an item body is found, and closed.

use flatmarkdown::{close_blocks, has_unclosed_block, split_body};

const FENCE: &str = "```";

// 開始記号だけのコードブロックは、閉じていない。閉じたコードブロックは、閉じている。
#[test]
fn a_code_block_without_its_closing_fence_is_unclosed() {
    assert!(has_unclosed_block(&format!("{FENCE}\nabc")));
    assert!(!has_unclosed_block(&format!("{FENCE}\nabc\n{FENCE}")));
}

/// A body for each kind of block that closes with a line, each without its closing line.
fn unclosed_bodies() -> Vec<String> {
    vec![
        "~~~\nabc".to_string(),
        format!("{FENCE}math\nx^2"),
        format!("````\nabc\n{FENCE}"),
        ">>>\nabc".to_string(),
        "<!--\nabc".to_string(),
        "<pre>\nabc".to_string(),
        "<script>\nabc".to_string(),
        "<?\nabc".to_string(),
        "<![CDATA[\nabc".to_string(),
        "<!DOCTYPE\nabc".to_string(),
        format!(">>>\n{FENCE}\nabc"),
        ">>>\n<!--\nabc".to_string(),
    ]
}

// 行で開いて行で閉じるブロックは、どれも閉じていなければ見つかる。コードブロック、数式のブロック、複数行の引用、HTMLのブロックである。
#[test]
fn every_block_that_closes_with_a_line_is_found_unclosed() {
    for body in unclosed_bodies() {
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

// 閉じていないコードブロックは、開始記号と同じ記号と長さの行で閉じる。
#[test]
fn a_code_block_is_closed_with_a_fence_of_its_own_kind() {
    assert_eq!(
        close_blocks(&format!("{FENCE}\nabc")),
        format!("{FENCE}\nabc\n{FENCE}")
    );
    assert_eq!(close_blocks("~~~~\nabc"), "~~~~\nabc\n~~~~");
}

// どの種類のブロックも、補った本文はもう閉じている。補うのは本文の後ろだけである。
#[test]
fn every_unclosed_block_is_closed_by_adding_lines_after_it() {
    for body in unclosed_bodies() {
        let closed = close_blocks(&body);
        assert!(closed.starts_with(&body), "{closed:?}");
        assert!(!has_unclosed_block(&closed), "{closed:?}");
    }
}

// 複数行の引用の中のブロックが閉じていれば、補うのは引用の終了記号だけである。
#[test]
fn a_quote_with_a_closed_block_inside_gets_only_its_own_closing() {
    for inner in [
        format!("{FENCE}\nabc\n{FENCE}"),
        "<!--\nabc\n-->".to_string(),
    ] {
        let body = format!(">>>\n{inner}");
        assert_eq!(close_blocks(&body), format!("{body}\n>>>"));
    }
}

// ブロックの外で分けると、前と後ろはそのまま分かれる。位置は文字で数える。
#[test]
fn a_body_split_outside_a_block_is_split_as_it_is() {
    assert_eq!(
        split_body("日本語", 1),
        ("日".to_string(), "本語".to_string())
    );
}

// コードブロックの中で分けると、前の部分は閉じ、後ろの部分には情報文字列ごと開始行が写される。
#[test]
fn a_code_block_split_in_two_is_closed_and_opened_again() {
    let (before, after) = split_body(&format!("{FENCE}rust\ncode\n{FENCE}"), 10);
    assert_eq!(before, format!("{FENCE}rust\nco\n{FENCE}"));
    assert_eq!(after, format!("{FENCE}rust\nde\n{FENCE}"));
}

// HTMLのブロックの中で分けると、後ろの部分には、開始行のうち開始の記号の部分が写される。属性も写される。
#[test]
fn an_html_block_split_in_two_is_opened_again_with_its_opening_text() {
    assert_eq!(
        split_body("<!DOCTYPE html\nab\n>", 16),
        (
            "<!DOCTYPE html\na\n>".to_string(),
            "<!DOCTYPE\nb\n>".to_string()
        )
    );
    assert_eq!(
        split_body("<pre class=\"x\">\nab\n</pre>", 17),
        (
            "<pre class=\"x\">\na\n</pre>".to_string(),
            "<pre class=\"x\">\nb\n</pre>".to_string()
        )
    );
}

// 複数行の引用の中のコードブロックで分けると、前は内側から閉じ、後ろは外側から開く。
#[test]
fn a_code_block_in_a_quote_split_in_two_closes_and_opens_both() {
    let body = format!(">>>\n{FENCE}\nab\n{FENCE}\n>>>");
    let (before, after) = split_body(&body, 9);
    assert_eq!(before, format!(">>>\n{FENCE}\na\n{FENCE}\n>>>"));
    assert_eq!(after, format!(">>>\n{FENCE}\nb\n{FENCE}\n>>>"));
}

// どこで分けても、前の部分と後ろの部分は、どちらも閉じている。どの種類のブロックでも同じである。
#[test]
fn both_parts_of_a_split_are_closed_wherever_it_is() {
    for body in unclosed_bodies().iter().map(|b| close_blocks(b)) {
        for at in 0..=body.chars().count() {
            let (before, after) = split_body(&body, at);
            assert!(!has_unclosed_block(&before), "{body:?} at {at}: {before:?}");
            assert!(!has_unclosed_block(&after), "{body:?} at {at}: {after:?}");
        }
    }
}
