//! A blank line in an item body would end the item in a page. In the file, such a line is
//! written as a line of one backslash, and read back as a blank line (ADR 070 of PetaJournal).

use flatmarkdown::{body_from_file, body_to_file};

// 段落の途中の空行は、ファイルでは`\`だけの行になる。
#[test]
fn a_blank_line_inside_a_paragraph_is_written_as_a_backslash_line() {
    assert_eq!(body_to_file("a\n\nb"), "a\n\\\nb");
}

// ファイルの`\`だけの行は、読むと空行に戻る。
#[test]
fn a_backslash_line_is_read_as_a_blank_line() {
    assert_eq!(body_from_file("a\n\\\nb"), "a\n\nb");
}
