//! A blank line in an item body would end the item in a page. In the file, such a line is
//! written as a line of one backslash, and read back as a blank line (ADR 070 of PetaJournal).

use flatmarkdown::{body_from_file, body_to_file, parse_page};
use proptest::prelude::*;

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

// コードブロックの中の`\`だけの行も、書くと`\`が1つ増える。読む向きが本文の構造を見ずに済み、どの本文も元に戻るためである。
#[test]
fn a_line_of_backslashes_inside_a_code_block_gets_one_more_too() {
    let body = format!("{FENCE}\n\\\n{FENCE}");

    assert_eq!(body_to_file(&body), format!("{FENCE}\n\\\\\n{FENCE}"));
}

/// Bodies whose blocks change when a blank line in them is written as a backslash line.
fn tricky_bodies() -> Vec<String> {
    vec![
        "a\n\n    code".to_string(),
        "a\n\n---".to_string(),
        "a\n\n===".to_string(),
        "- a\n\nb".to_string(),
        "> a\n\nb".to_string(),
        "|a|\n|-|\n|1|\n\nb".to_string(),
        "[x]: /url\n\nb".to_string(),
        "\n<div>\na".to_string(),
        format!("a\n\n{FENCE}\n\\\n\n{FENCE}\n\n\\\\"),
        "    code\n\n\\".to_string(),
        "\\\n\n\\\\\n \n".to_string(),
        "<div>\n\n\\\n\nb".to_string(),
        "\n<custom>\n\\".to_string(),
        "a\n\n</pre>\n\\\\".to_string(),
        "<div>\na\n\n\nb".to_string(),
        "\n    x\n<custom>\n\\".to_string(),
        "\n|-|\n</pre>\n\\".to_string(),
    ]
}

// どの本文も、ファイルに書いて読むと元に戻る。
#[test]
fn a_body_written_and_read_again_is_the_same() {
    for body in tricky_bodies() {
        assert_eq!(body_from_file(&body_to_file(&body)), body, "{body:?}");
    }
}

// 段落の中に空行がある本文は、ページに書いて読み戻しても、1つの項目のままである。
#[test]
fn a_paragraph_with_blank_lines_stays_one_item_in_a_page() {
    let body = "\na\n\nb\n\n\nc";
    let page = parse_page(&format!("<!-- @ -->\n{}\n", body_to_file(body)));

    assert_eq!(page.items.len(), 1);
    assert_eq!(body_from_file(&page.items[0].body), body);
}

// 後ろにブロックのない空行は変わらない。参照の定義はブロックを作らず、前のブロックに付くためである。
#[test]
fn a_blank_line_with_no_block_after_it_is_kept() {
    let body = "[a]\n\n[a]: /url";
    assert_eq!(body_to_file(body), body);
}

/// Lines that start or end blocks, and lines of any of the marks of Markdown.
fn any_line() -> impl Strategy<Value = String> {
    let marked = prop::sample::select(vec![
        "", "\\", "\\\\", FENCE, "~~~", "    x", "<div>", "</div>", "<pre>", "</pre>", "<custom>",
        "<!--", "-->", "- a", "  - b", "1. a", "> a", ">>>", "---", "===", "# h", "|a|", "|-|",
        "[x]: /u", "$$", "- [ ] t",
    ])
    .prop_map(str::to_string);
    prop_oneof![marked, "[a-z \\\\`<>|#*=:/\\[\\]-]{1,6}"]
        // A line of spaces comes back as a blank line, as ADR 070 says
        .prop_map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                line
            }
        })
}

proptest! {
    // どの本文も、ファイルに書いて読むと元に戻る。行は、ブロックを始めたり終えたりする行と、Markdownの記号を混ぜた行である。
    #[test]
    fn any_body_written_and_read_again_is_the_same(lines in prop::collection::vec(any_line(), 0..12)) {
        let body = lines.join("\n");
        prop_assert_eq!(body_from_file(&body_to_file(&body)), body);
    }

    // 段落の行と空行でできた本文は、ページに書いて読み戻しても、1つの項目のままである。
    #[test]
    fn a_paragraph_with_any_blank_lines_stays_one_item_in_a_page(
        lines in prop::collection::vec(prop_oneof!["", "[a-z]{1,5}"], 0..12),
        last in "[a-z]{1,5}",
    ) {
        // The last line has text: a page drops the blank lines at the end of a body
        let body = format!("{}{last}", lines.iter().map(|line| format!("{line}\n")).collect::<String>());
        let page = parse_page(&format!("<!-- @ -->\n{}\n", body_to_file(&body)));

        prop_assert_eq!(page.items.len(), 1);
        prop_assert_eq!(body_from_file(&page.items[0].body), body);
    }
}
