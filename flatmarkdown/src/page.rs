//! Parses a whole page: the front matter, the metadata lines, and the items.

use crate::ast::options;
use crate::body::{BodyOptions, body_to_node};
use comrak::nodes::NodeValue;
use comrak::{Arena, parse_document};
use flatmarkdown_ast::{Item, Page};

/// Item metadata parsed from a `<!-- @@ ID key="value" -->` line.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ItemMetadata {
    level: usize,
    id: Option<String>,
    properties: Vec<(String, String)>,
}

/// The characters of the random tail of an HMTID: Crockford's Base32. It has no lower case
/// and no `I`, `L`, `O`, or `U`. See https://github.com/sosuisen/hmtid.
const HMTID_TAIL: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Returns `true` if `s` has the HMTID shape: `YYYY-MM-DD-hh-mm-ss-XXXXXXX`
/// (27 characters, ASCII digits in the date part and Crockford's Base32 in the tail).
fn is_hmtid(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 27 {
        return false;
    }
    b.iter().enumerate().all(|(i, &c)| match i {
        4 | 7 | 10 | 13 | 16 | 19 => c == b'-',
        0..=18 => c.is_ascii_digit(),
        _ => HMTID_TAIL.contains(&c),
    })
}

/// Parses `key="value"` starting at `chars[pos]`.
/// Returns the pair and the index just after the closing quote, or `None` if malformed.
fn parse_property(chars: &[char], pos: usize) -> Option<((String, String), usize)> {
    let mut i = pos;
    let mut key = String::new();
    while i < chars.len() && chars[i] != '=' {
        if chars[i].is_whitespace() || chars[i] == '"' {
            return None;
        }
        key.push(chars[i]);
        i += 1;
    }
    if key.is_empty() || i + 1 >= chars.len() || chars[i + 1] != '"' {
        return None;
    }
    i += 2;
    let mut value = String::new();
    loop {
        let c = *chars.get(i)?;
        match c {
            '"' => break,
            '\\' => {
                i += 1;
                match *chars.get(i)? {
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    _ => return None,
                }
            }
            _ => value.push(c),
        }
        i += 1;
    }
    if value.contains("-->") {
        return None;
    }
    Some(((key, value), i + 1))
}

/// Parses a single-line metadata comment `<!-- @@ ID key="value" -->`.
/// Returns `None` for anything that does not follow the metadata syntax exactly,
/// because such comments must stay in the body.
fn parse_metadata_line(line: &str) -> Option<ItemMetadata> {
    // Trailing whitespace is invisible in an editor, so it does not invalidate the line.
    let inner = line
        .trim_end()
        .strip_prefix("<!-- @")?
        .strip_suffix("-->")?;
    // One or more spaces stand before `-->`. How many does not matter, as between the elements.
    let inner = inner.strip_suffix(' ')?.trim_end_matches(' ');
    let chars: Vec<char> = inner.chars().collect();
    let extra_ats = chars.iter().take_while(|&&c| c == '@').count();
    let level = 1 + extra_ats;
    let mut i = extra_ats;
    let mut id = None;
    let mut properties = Vec::new();
    let mut first = true;
    while i < chars.len() {
        if chars[i] != ' ' {
            return None;
        }
        // The spaces at the end are already gone, so an element always follows these.
        while chars[i] == ' ' {
            i += 1;
        }
        let token_end = chars[i..]
            .iter()
            .position(|&c| c == ' ')
            .map_or(chars.len(), |p| i + p);
        let token: String = chars[i..token_end].iter().collect();
        if first && !token.contains('=') {
            if !is_hmtid(&token) {
                return None;
            }
            id = Some(token);
            i = token_end;
        } else {
            let (prop, next) = parse_property(&chars, i)?;
            properties.push(prop);
            i = next;
        }
        first = false;
    }
    Some(ItemMetadata {
        level,
        id,
        properties,
    })
}

/// Returns `true` if the line is empty or contains only whitespace.
fn is_blank(line: &str) -> bool {
    line.chars().all(char::is_whitespace)
}

/// Joins `lines[start..=end]` after trimming blank lines from both ends.
fn body_text(lines: &[&str], start: usize, end: usize) -> String {
    let mut s = start;
    let mut e = end.min(lines.len().saturating_sub(1));
    while s <= e && is_blank(lines[s]) {
        s += 1;
    }
    while e > s && is_blank(lines[e]) {
        e -= 1;
    }
    if s > e {
        return String::new();
    }
    lines[s..=e].join("\n")
}

/// Builds the item of one metadata line (or of none) and its body.
fn item(meta: Option<ItemMetadata>, body: String, opts: &BodyOptions) -> Item {
    let meta = meta.unwrap_or(ItemMetadata {
        level: 1,
        id: None,
        properties: Vec::new(),
    });
    let ast = body_to_node(&body, opts);
    Item {
        level: meta.level as u32,
        id: meta.id,
        properties: meta.properties.into_iter().collect(),
        body,
        ast,
    }
}

/// An item whose body has not been closed yet while scanning a page.
struct OpenItem {
    meta: Option<ItemMetadata>,
    /// First source line (0-based) of the body.
    start: usize,
    /// Whether the item already owns a block. A second block starts a new item.
    has_block: bool,
}

/// Parses a whole Flat Markdown page into a JSON string:
/// `{"type":"page","front_matter":string|null,"items":[{level,id,properties,body,ast}]}`.
///
/// Each block at the shallowest level becomes one item. A single-line
/// `<!-- @@ ID key="value" -->` comment directly above a block is that block's metadata.
///
/// An item's body spans from its start (the line after its metadata line, or the
/// block itself) up to the line before the next item starts. Lines that comrak
/// consumes without creating a node (link reference definitions) and footnote
/// definitions therefore stay with the item they follow, which is where the
/// paragraph that uses them normally is.
pub fn page_to_ast_with(input: &str, opts: &BodyOptions) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let arena = Arena::new();
    let root = parse_document(&arena, input, &options(true));

    let mut front_matter: Option<String> = None;
    let mut items: Vec<Item> = Vec::new();
    // The implicit leading item collects lines that precede the first block, so that
    // nothing before the first block or metadata line is lost.
    let mut open = OpenItem {
        meta: None,
        start: 0,
        has_block: false,
    };

    let close = |open: OpenItem, end_exclusive: usize, items: &mut Vec<Item>| {
        let body = if end_exclusive > open.start {
            body_text(&lines, open.start, end_exclusive - 1)
        } else {
            String::new()
        };
        // A metadata line alone is an item with an empty body. Anything else
        // without text is just blank space between items.
        if open.meta.is_some() || !body.is_empty() {
            items.push(item(open.meta, body, opts));
        }
    };

    for node in root.children() {
        let data = node.data.borrow();
        let start = data.sourcepos.start.line.saturating_sub(1);
        let end = data.sourcepos.end.line.saturating_sub(1);

        if let NodeValue::FrontMatter(s) = &data.value {
            front_matter = Some(s.clone());
            open.start = end + 1;
            continue;
        }

        // A referenced footnote definition is a node, but it is treated like an
        // unreferenced one so that a definition always joins the previous item. comrak
        // also moves these nodes to the end of the document, which would otherwise put
        // the same lines into two items.
        if matches!(data.value, NodeValue::FootnoteDefinition(_)) {
            continue;
        }

        let meta = match &data.value {
            NodeValue::HtmlBlock(_) if start == end => parse_metadata_line(lines[start]),
            _ => None,
        };

        if let Some(meta) = meta {
            close(open, start, &mut items);
            open = OpenItem {
                meta: Some(meta),
                start: end + 1,
                has_block: false,
            };
        } else if open.has_block {
            close(open, start, &mut items);
            open = OpenItem {
                meta: None,
                start,
                has_block: true,
            };
        } else {
            open.has_block = true;
        }
    }
    close(open, lines.len(), &mut items);

    let page = Page {
        front_matter,
        items,
    };
    serde_json::to_string(&page).unwrap()
}

pub fn page_to_ast(input: &str) -> String {
    page_to_ast_with(input, &BodyOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{BodyOptions, LinkResolution};
    use crate::test_support::page;
    use serde_json::{Value, json};

    // ---- Metadata line syntax ----

    const ID1: &str = "2026-09-26-18-14-47-32W15WT";
    const ID2: &str = "2026-09-26-18-15-30-6VZSYB4";
    const ID3: &str = "2026-09-27-15-00-16-W3N8ASY";

    // @の数がレベルになる。IDとプロパティは、なくてもよい。
    #[test]
    fn metadata_level_only() {
        let m = parse_metadata_line("<!-- @ -->").unwrap();
        assert_eq!(
            m,
            ItemMetadata {
                level: 1,
                id: None,
                properties: vec![]
            }
        );
        let m = parse_metadata_line("<!-- @@@ -->").unwrap();
        assert_eq!(m.level, 3);
    }

    // @のあとにIDを書ける。
    #[test]
    fn metadata_with_id() {
        let m = parse_metadata_line(&format!("<!-- @@ {ID1} -->")).unwrap();
        assert_eq!(m.level, 2);
        assert_eq!(m.id.as_deref(), Some(ID1));
        assert!(m.properties.is_empty());
    }

    // IDのあとに、key="value"の形のプロパティを書ける。書いた順に並ぶ。値は空白を含んでよい。
    #[test]
    fn metadata_with_id_and_properties() {
        let m = parse_metadata_line(&format!(
            "<!-- @ {ID1} plan=\"2026-09-28\" title=\"a b\" -->"
        ))
        .unwrap();
        assert_eq!(m.id.as_deref(), Some(ID1));
        assert_eq!(
            m.properties,
            vec![
                ("plan".to_string(), "2026-09-28".to_string()),
                ("title".to_string(), "a b".to_string()),
            ]
        );
    }

    // IDがなくても、プロパティを書ける。
    #[test]
    fn metadata_properties_without_id() {
        let m = parse_metadata_line("<!-- @@ collapsed=\"true\" -->").unwrap();
        assert_eq!(m.level, 2);
        assert_eq!(m.id, None);
        assert_eq!(
            m.properties,
            vec![("collapsed".to_string(), "true".to_string())]
        );
    }

    // 値の中の\"は"に、\\は\になる。
    #[test]
    fn metadata_value_escapes() {
        let m = parse_metadata_line(r#"<!-- @ k="say \"hi\" \\ done" -->"#).unwrap();
        assert_eq!(m.properties[0].1, r#"say "hi" \ done"#);
    }

    // 形の合わない行は、メタデータ行ではない。
    #[test]
    fn metadata_invalid_lines() {
        // Not starting with `<!-- @`
        assert!(parse_metadata_line("<!-- comment -->").is_none());
        assert!(parse_metadata_line("<!--@ -->").is_none());
        // Leading whitespace
        assert!(parse_metadata_line(" <!-- @ -->").is_none());
        // Invalid ID
        assert!(parse_metadata_line("<!-- @ not-an-id -->").is_none());
        // Unquoted value
        assert!(parse_metadata_line("<!-- @ key=value -->").is_none());
        // Key containing `"`
        assert!(parse_metadata_line("<!-- @ k\"ey=\"v\" -->").is_none());
        // Unterminated value
        assert!(parse_metadata_line("<!-- @ key=\"v -->").is_none());
        // ID after a property
        assert!(parse_metadata_line(&format!("<!-- @ k=\"v\" {ID1} -->")).is_none());
        // Missing space before `-->`
        assert!(parse_metadata_line("<!-- @-->").is_none());
    }

    // IDの日付の部分は数字でなければならない。英字が入るとメタデータ行ではない。
    #[test]
    fn metadata_id_with_a_letter_in_its_date_part_is_invalid() {
        assert!(parse_metadata_line("<!-- @ 2026-09-26-18-14-47-32W15WT -->").is_some());
        assert!(parse_metadata_line("<!-- @ 2026-09-2X-18-14-47-32W15WT -->").is_none());
        assert!(parse_metadata_line("<!-- @ X026-09-26-18-14-47-32W15WT -->").is_none());
    }

    // @の直後は空白でなければならない。
    #[test]
    fn metadata_needs_a_space_right_after_the_at_signs() {
        assert!(parse_metadata_line("<!-- @x -->").is_none());
        assert!(parse_metadata_line("<!-- @@k=\"v\" -->").is_none());
    }

    // -->の前の空白は、2つ以上あってもよい。1つのときと同じ結果になる。
    #[test]
    fn metadata_may_have_several_spaces_before_the_end() {
        assert_eq!(
            parse_metadata_line("<!-- @@   -->"),
            parse_metadata_line("<!-- @@ -->")
        );
        assert_eq!(
            parse_metadata_line("<!-- @ k=\"v\"  -->"),
            parse_metadata_line("<!-- @ k=\"v\" -->")
        );
        assert!(parse_metadata_line("<!-- @ k=\"v\"  -->").is_some());
    }

    // 要素の間の空白は、2つ以上あってもよい。1つのときと同じ結果になる。
    #[test]
    fn metadata_may_have_several_spaces_between_its_elements() {
        assert_eq!(
            parse_metadata_line(&format!("<!-- @@   {ID1}  a=\"1\"   b=\"2\" -->")),
            parse_metadata_line(&format!("<!-- @@ {ID1} a=\"1\" b=\"2\" -->"))
        );
        assert!(parse_metadata_line("<!-- @  k=\"v\" -->").is_some());
    }

    // -->の前の空白が1つもなければ、メタデータ行ではない。
    #[test]
    fn metadata_without_a_space_before_the_end_is_invalid() {
        assert!(parse_metadata_line("<!-- @@-->").is_none());
        assert!(parse_metadata_line("<!-- @ k=\"v\"-->").is_none());
    }

    // =で終わるプロパティは、値がないのでメタデータ行ではない。
    #[test]
    fn metadata_property_that_ends_at_the_equals_sign_is_invalid() {
        assert!(parse_metadata_line("<!-- @ k= -->").is_none());
    }

    // 値の中の\は、"か\の前でだけ使える。-->を含む値は使えない。
    #[test]
    fn metadata_value_with_another_escape_or_a_comment_end_is_invalid() {
        assert!(parse_metadata_line(r#"<!-- @ k="a\n" -->"#).is_none());
        assert!(parse_metadata_line(r#"<!-- @ k="a-->b" -->"#).is_none());
    }

    // -->のあとの空白は許される。半角の空白、タブ、全角の空白のどれでも、メタデータ行である。
    #[test]
    fn metadata_line_may_have_whitespace_after_its_end() {
        let plain = parse_metadata_line(&format!("<!-- @@ {ID1} k=\"v\" -->")).unwrap();
        for tail in ["  ", "\t", "\u{3000}", " \t "] {
            let line = format!("<!-- @@ {ID1} k=\"v\" -->{tail}");
            assert_eq!(parse_metadata_line(&line), Some(plain.clone()), "{tail:?}");
        }
    }

    // -->のあとに空白でない文字があれば、メタデータ行ではない。
    #[test]
    fn metadata_line_with_text_after_its_end_is_invalid() {
        assert!(parse_metadata_line("<!-- @ --> x").is_none());
    }

    // IDの末尾の7文字は、CrockfordのBase32である。小文字と、I、L、O、Uは使えない。
    #[test]
    fn metadata_id_with_a_tail_outside_crockford_base32_is_invalid() {
        assert!(parse_metadata_line("<!-- @ 2026-09-26-18-14-47-0189ABZ -->").is_some());
        for tail in ["32w15wt", "32W15WI", "32W15WL", "32W15WO", "32W15WU"] {
            let line = format!("<!-- @ 2026-09-26-18-14-47-{tail} -->");
            assert!(parse_metadata_line(&line).is_none(), "{tail}");
        }
    }

    // ---- Page parsing ----

    // 仕様書の例のページは4つの項目になる。メタデータ行のないブロックはレベル1の項目に、本文のないメタデータ行は空の項目になる。
    #[test]
    fn page_spec_example() {
        let input = format!(
            "<!-- @ {ID1} -->\n\n# Heading 1\n\nParagraph 1\n\n<!-- @@ {ID2} -->\n\n\n<!-- @@ {ID3} plan=\"2026-09-28\" -->\n- [ ] Task1\n"
        );
        let v = page(&input);
        assert_eq!(v["type"], "page");
        assert_eq!(v["front_matter"], Value::Null);
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 4);

        assert_eq!(items[0]["level"], 1);
        assert_eq!(items[0]["id"], ID1);
        assert_eq!(items[0]["body"], "# Heading 1");
        assert_eq!(items[0]["ast"]["children"][0]["type"], "heading");

        assert_eq!(items[1]["level"], 1);
        assert_eq!(items[1]["id"], Value::Null);
        assert_eq!(items[1]["properties"], json!({}));
        assert_eq!(items[1]["body"], "Paragraph 1");

        assert_eq!(items[2]["level"], 2);
        assert_eq!(items[2]["id"], ID2);
        assert_eq!(items[2]["body"], "");
        assert_eq!(items[2]["ast"]["type"], "document");

        assert_eq!(items[3]["level"], 2);
        assert_eq!(items[3]["id"], ID3);
        assert_eq!(items[3]["properties"]["plan"], "2026-09-28");
        assert_eq!(items[3]["body"], "- [ ] Task1");
        assert_eq!(
            items[3]["ast"]["children"][0]["children"][0]["type"],
            "task_item"
        );
    }

    // front matterは項目にならない。ページのfront_matterに、そのままの文字列で入る。
    #[test]
    fn page_front_matter_is_not_an_item() {
        let v = page("---\ntitle: x\n---\n\nHello\n");
        assert_eq!(v["front_matter"], "---\ntitle: x\n---\n\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["body"], "Hello");
    }

    // 見出しと段落は、間に空行がなくても2つの項目になる。
    #[test]
    fn page_heading_and_paragraph_without_blank_line_are_two_items() {
        let v = page("# H\nP");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["body"], "# H");
        assert_eq!(items[1]["body"], "P");
    }

    // 項目の間に空行のあるリストは、入れ子を含めて1つの項目になる。
    #[test]
    fn page_loose_list_is_one_item() {
        let v = page("- a\n\n- b\n  - nested\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["body"], "- a\n\n- b\n  - nested");
    }

    // 複数の段落を持つ引用は、1つの項目になる。
    #[test]
    fn page_block_quote_with_paragraphs_is_one_item() {
        let v = page("> p1\n>\n> p2\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["body"], "> p1\n>\n> p2");
    }

    // メタデータ行の形でないHTMLコメントは、本文として残る。
    #[test]
    fn page_invalid_metadata_stays_in_body() {
        let v = page("<!-- note -->\n\n<!-- @ bad id -->\n\n<!-- @\nx -->\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0]["body"], "<!-- note -->");
        assert_eq!(items[1]["body"], "<!-- @ bad id -->");
        assert_eq!(items[2]["body"], "<!-- @\nx -->");
        assert!(items.iter().all(|i| i["id"].is_null() && i["level"] == 1));
    }

    // ブロックの中にあるメタデータ行の形の行は、本文である。
    #[test]
    fn page_metadata_inside_block_is_body() {
        let v = page("- a\n  <!-- @ -->\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["body"], "- a\n  <!-- @ -->");
    }

    // ページの末尾のメタデータ行は、本文が空の項目になる。
    #[test]
    fn page_trailing_metadata_is_empty_item() {
        let v = page(&format!("P\n\n<!-- @@ {ID1} -->\n\n"));
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[1]["id"], ID1);
        assert_eq!(items[1]["body"], "");
    }

    // メタデータ行と本文の間に、空行はなくてもよい。
    #[test]
    fn page_metadata_without_blank_line_before_body() {
        let v = page(&format!("<!-- @ {ID1} -->\nP1\nP2\n"));
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["id"], ID1);
        assert_eq!(items[0]["body"], "P1\nP2");
    }

    // ノードにならない行（参照の定義、参照されない脚注の定義）は、前のブロックの項目に入る。
    // Lines that produce no comrak node (reference definitions, unreferenced
    // footnote definitions) must not be lost. They stay with the item they follow.
    #[test]
    fn page_nodeless_lines_follow_previous_block() {
        let v = page("P\n\n[ref]: /url\n\nQ\n\n[^1]: unreferenced\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["body"], "P\n\n[ref]: /url");
        assert_eq!(items[1]["body"], "Q\n\n[^1]: unreferenced");
    }

    // 同じ項目に入った参照の定義は、その項目の本文だけを解析しても解決される。
    #[test]
    fn page_reference_definition_resolves_within_item() {
        let v = page("See [site][s].\n\n[s]: https://example.com\n\nNext\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        let link = &items[0]["ast"]["children"][0]["children"][1];
        assert_eq!(link["type"], "link");
        assert_eq!(link["url"], "https://example.com");
    }

    // メタデータ行の前にあるノードにならない行は、前の項目に入る。
    #[test]
    fn page_nodeless_lines_before_metadata_stay_with_previous_item() {
        let v = page(&format!("P\n\n[ref]: /url\n\n<!-- @ {ID1} -->\nQ\n"));
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["body"], "P\n\n[ref]: /url");
        assert_eq!(items[1]["id"], ID1);
        assert_eq!(items[1]["body"], "Q");
    }

    // メタデータ行とブロックの間にあるノードにならない行は、そのメタデータ行の項目に入る。
    #[test]
    fn page_nodeless_lines_between_metadata_and_block_belong_to_that_item() {
        let v = page(&format!("<!-- @ {ID1} -->\n\n[ref]: /url\n\nQ\n"));
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["id"], ID1);
        assert_eq!(items[0]["body"], "[ref]: /url\n\nQ");
    }

    // ページの先頭のノードにならない行は、最初のブロックの項目に入る。
    #[test]
    fn page_leading_nodeless_lines_join_first_block() {
        let v = page("[ref]: /url\n\nP\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["body"], "[ref]: /url\n\nP");
    }

    // ページの先頭のノードにならない行は、すぐあとがメタデータ行なら、それだけで1つの項目になる。
    #[test]
    fn page_leading_nodeless_lines_before_metadata_form_own_item() {
        let v = page(&format!("[ref]: /url\n\n<!-- @ {ID1} -->\nP\n"));
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["id"], Value::Null);
        assert_eq!(items[0]["body"], "[ref]: /url");
        assert_eq!(items[1]["body"], "P");
    }

    // 1つのメタデータ行に同じキーを2回書くと、後の値だけが残る。ほかのキーは変わらない。
    #[test]
    fn page_keeps_the_last_value_of_a_key_written_twice() {
        let v = page("<!-- @ a=\"1\" b=\"x\" a=\"2\" -->\nP\n");
        assert_eq!(v["items"][0]["properties"], json!({"a": "2", "b": "x"}));
    }

    // 空のページは、項目がなく、front_matterはnullである。
    #[test]
    fn page_empty_input() {
        let v = page("");
        assert_eq!(v["items"].as_array().unwrap().len(), 0);
        assert_eq!(v["front_matter"], Value::Null);
    }

    // ページの解析でも、リンク解決のオプションが項目の本文に使われる。
    #[test]
    fn page_applies_link_resolution() {
        let opts = BodyOptions {
            link_resolution: LinkResolution::On {
                prefixes: vec!["../".into()],
            },
        };
        let v: Value = serde_json::from_str(&page_to_ast_with("#tag [p](../p.md)", &opts)).unwrap();
        let inline = v["items"][0]["ast"]["children"][0]["children"]
            .as_array()
            .unwrap();
        assert_eq!(inline[0]["type"], "hashtag");
        assert_eq!(inline[2]["type"], "wikilink");
    }
}
