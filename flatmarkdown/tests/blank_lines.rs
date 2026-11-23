//! A blank line in an item body would end the item in a page. In the file, such a line is
//! written as a line of one backslash, and read back as a blank line (ADR 070 of PetaJournal).

use flatmarkdown::body_to_file;

// 段落の途中の空行は、ファイルでは`\`だけの行になる。
#[test]
fn a_blank_line_inside_a_paragraph_is_written_as_a_backslash_line() {
    assert_eq!(body_to_file("a\n\nb"), "a\n\\\nb");
}
