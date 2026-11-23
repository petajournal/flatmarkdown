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

const FENCE: &str = "```";

// 1文字以上の`\`だけの行は、書くと`\`が1つ増え、読むと1つ減る。空行の`\`と区別するためである。
#[test]
fn a_line_of_backslashes_gets_one_more_when_written_and_one_less_when_read() {
    let body = "\\\na\n\\\\";
    let file = "\\\\\na\n\\\\\\";

    assert_eq!(body_to_file(body), file);
    assert_eq!(body_from_file(file), body);
}

// 空白やタブだけの行も空行として`\`だけの行になり、中の空白は残らない。
#[test]
fn a_line_of_spaces_is_written_as_a_backslash_line() {
    assert_eq!(body_to_file("a\n \t \nb"), "a\n\\\nb");
}

// 本文の先頭の空行は`\`だけの行になり、末尾の空行は変わらない。
#[test]
fn blank_lines_at_the_start_are_written_and_those_at_the_end_are_kept() {
    assert_eq!(body_to_file("\n\na\n\n"), "\\\n\\\na\n\n");
}

// 見出しと段落のように、最上位のブロックの間の空行も`\`だけの行になる。
#[test]
fn a_blank_line_between_a_heading_and_a_paragraph_is_written_as_a_backslash_line() {
    assert_eq!(body_to_file("# H1\n\nPara"), "# H1\n\\\nPara");
}

// リスト、コードブロック、`>>>`の引用の中の空行は、項目を分けないので変わらない。
#[test]
fn blank_lines_inside_a_list_a_code_block_or_a_quote_are_kept() {
    for body in [
        "- a\n\n- b".to_string(),
        format!("{FENCE}\na\n\nb\n{FENCE}"),
        "    a\n\n    b".to_string(),
        ">>>\na\n\nb\n>>>".to_string(),
    ] {
        assert_eq!(body_to_file(&body), body, "{body:?}");
    }
}

// コードブロックとHTMLのブロックの中の`\`だけの行は、書いても読んでも変わらない。中の文字は、そのまま表示されるためである。
#[test]
fn a_line_of_backslashes_inside_a_code_block_or_an_html_block_is_kept() {
    for body in [
        format!("{FENCE}\n\\\n{FENCE}"),
        "    a\n    \\\\".to_string(),
        "<pre>\n\\\n</pre>".to_string(),
        "<div>\n\\\n</div>".to_string(),
    ] {
        assert_eq!(body_to_file(&body), body, "{body:?}");
        assert_eq!(body_from_file(&body), body, "{body:?}");
    }
}

// 後ろにブロックのない空行は変わらない。参照の定義はブロックを作らず、前のブロックに付くためである。
#[test]
fn a_blank_line_with_no_block_after_it_is_kept() {
    let body = "[a]\n\n[a]: /url";
    assert_eq!(body_to_file(body), body);
}
