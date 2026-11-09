//! Parses a whole page: the front matter, the metadata lines, and the items.

use crate::ast::options;
use crate::body::{BodyOptions, body_to_ast_value};
use comrak::nodes::NodeValue;
use comrak::{Arena, parse_document};
use serde_json::{Value, json};

/// Item metadata parsed from a `<!-- @@ ID key="value" -->` line.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ItemMetadata {
    level: usize,
    id: Option<String>,
    properties: Vec<(String, String)>,
}

/// Returns `true` if `s` has the HMTID shape: `YYYY-MM-DD-hh-mm-ss-XXXXXXX`
/// (27 characters, ASCII digits in the date part and ASCII alphanumerics in the tail).
fn is_hmtid(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 27 {
        return false;
    }
    b.iter().enumerate().all(|(i, &c)| match i {
        4 | 7 | 10 | 13 | 16 | 19 => c == b'-',
        0..=18 => c.is_ascii_digit(),
        _ => c.is_ascii_alphanumeric(),
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
        .strip_suffix(" -->")?;
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
        while i < chars.len() && chars[i] == ' ' {
            i += 1;
        }
        if i >= chars.len() {
            return None;
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

fn item_json(meta: Option<ItemMetadata>, body: String, opts: &BodyOptions) -> Value {
    let meta = meta.unwrap_or(ItemMetadata {
        level: 1,
        id: None,
        properties: Vec::new(),
    });
    let properties: serde_json::Map<String, Value> = meta
        .properties
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();
    let ast = body_to_ast_value(&body, opts);
    json!({
        "level": meta.level,
        "id": meta.id,
        "properties": properties,
        "body": body,
        "ast": ast,
    })
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
    let mut items: Vec<Value> = Vec::new();
    // The implicit leading item collects lines that precede the first block, so that
    // nothing before the first block or metadata line is lost.
    let mut open = OpenItem {
        meta: None,
        start: 0,
        has_block: false,
    };

    let close = |open: OpenItem, end_exclusive: usize, items: &mut Vec<Value>| {
        let body = if end_exclusive > open.start {
            body_text(&lines, open.start, end_exclusive - 1)
        } else {
            String::new()
        };
        // A metadata line alone is an item with an empty body. Anything else
        // without text is just blank space between items.
        if open.meta.is_some() || !body.is_empty() {
            items.push(item_json(open.meta, body, opts));
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

    let page = json!({
        "type": "page",
        "front_matter": front_matter,
        "items": items,
    });
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

    // ---- Metadata line syntax ----

    const ID1: &str = "2026-09-26-18-14-47-32W15WT";
    const ID2: &str = "2026-09-26-18-15-30-6VZSYB4";
    const ID3: &str = "2026-09-27-15-00-16-W3N8ASY";

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

    #[test]
    fn metadata_with_id() {
        let m = parse_metadata_line(&format!("<!-- @@ {ID1} -->")).unwrap();
        assert_eq!(m.level, 2);
        assert_eq!(m.id.as_deref(), Some(ID1));
        assert!(m.properties.is_empty());
    }

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

    #[test]
    fn metadata_value_escapes() {
        let m = parse_metadata_line(r#"<!-- @ k="say \"hi\" \\ done" -->"#).unwrap();
        assert_eq!(m.properties[0].1, r#"say "hi" \ done"#);
    }

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

    // ---- Page parsing ----

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

    #[test]
    fn page_front_matter_is_not_an_item() {
        let v = page("---\ntitle: x\n---\n\nHello\n");
        assert_eq!(v["front_matter"], "---\ntitle: x\n---\n\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["body"], "Hello");
    }

    #[test]
    fn page_heading_and_paragraph_without_blank_line_are_two_items() {
        let v = page("# H\nP");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["body"], "# H");
        assert_eq!(items[1]["body"], "P");
    }

    #[test]
    fn page_loose_list_is_one_item() {
        let v = page("- a\n\n- b\n  - nested\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["body"], "- a\n\n- b\n  - nested");
    }

    #[test]
    fn page_block_quote_with_paragraphs_is_one_item() {
        let v = page("> p1\n>\n> p2\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["body"], "> p1\n>\n> p2");
    }

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

    #[test]
    fn page_metadata_inside_block_is_body() {
        let v = page("- a\n  <!-- @ -->\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["body"], "- a\n  <!-- @ -->");
    }

    #[test]
    fn page_trailing_metadata_is_empty_item() {
        let v = page(&format!("P\n\n<!-- @@ {ID1} -->\n\n"));
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[1]["id"], ID1);
        assert_eq!(items[1]["body"], "");
    }

    #[test]
    fn page_metadata_without_blank_line_before_body() {
        let v = page(&format!("<!-- @ {ID1} -->\nP1\nP2\n"));
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["id"], ID1);
        assert_eq!(items[0]["body"], "P1\nP2");
    }

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

    // A reference definition kept in the same item resolves when the body is parsed alone.
    #[test]
    fn page_reference_definition_resolves_within_item() {
        let v = page("See [site][s].\n\n[s]: https://example.com\n\nNext\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        let link = &items[0]["ast"]["children"][0]["children"][1];
        assert_eq!(link["type"], "link");
        assert_eq!(link["url"], "https://example.com");
    }

    #[test]
    fn page_nodeless_lines_before_metadata_stay_with_previous_item() {
        let v = page(&format!("P\n\n[ref]: /url\n\n<!-- @ {ID1} -->\nQ\n"));
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["body"], "P\n\n[ref]: /url");
        assert_eq!(items[1]["id"], ID1);
        assert_eq!(items[1]["body"], "Q");
    }

    #[test]
    fn page_nodeless_lines_between_metadata_and_block_belong_to_that_item() {
        let v = page(&format!("<!-- @ {ID1} -->\n\n[ref]: /url\n\nQ\n"));
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["id"], ID1);
        assert_eq!(items[0]["body"], "[ref]: /url\n\nQ");
    }

    #[test]
    fn page_leading_nodeless_lines_join_first_block() {
        let v = page("[ref]: /url\n\nP\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["body"], "[ref]: /url\n\nP");
    }

    #[test]
    fn page_leading_nodeless_lines_before_metadata_form_own_item() {
        let v = page(&format!("[ref]: /url\n\n<!-- @ {ID1} -->\nP\n"));
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["id"], Value::Null);
        assert_eq!(items[0]["body"], "[ref]: /url");
        assert_eq!(items[1]["body"], "P");
    }

    #[test]
    fn page_empty_input() {
        let v = page("");
        assert_eq!(v["items"].as_array().unwrap().len(), 0);
        assert_eq!(v["front_matter"], Value::Null);
    }

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
