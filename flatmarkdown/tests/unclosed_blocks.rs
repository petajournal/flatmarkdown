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
