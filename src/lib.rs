use comrak::nodes::NodeValue;
use comrak::{parse_document, Arena, Options};
use serde_json::{json, Value};
use std::collections::HashSet;

/// Builds the comrak options shared by body and page parsing.
/// Front matter is only recognised when parsing a whole page, because a body
/// that happens to start with `---` must stay a thematic break.
fn options(front_matter: bool) -> Options<'static> {
    let mut options = Options::default();

    // Render options
    options.render.full_info_string = true;
    options.render.gfm_quirks = true;
    options.render.r#unsafe = true;
    options.render.tasklist_classes = true;

    // Parse options
    options.parse.relaxed_tasklist_matching = true;
    options.parse.relaxed_autolinks = true;
    options.parse.default_info_string = Some("text".into());

    // Extension options (GFM)
    options.extension.strikethrough = true;
    options.extension.tagfilter = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;

    // Extension options (Comrak custom)
    options.extension.superscript = true;
    options.extension.footnotes = true;
    options.extension.inline_footnotes = true;
    options.extension.math_code = true;
    options.extension.underline = true;
    options.extension.subscript = true;
    options.extension.spoiler = true;
    options.extension.alerts = true;
    options.extension.cjk_friendly_emphasis = true;
    options.extension.subtext = true;
    options.extension.highlight = true;
    options.extension.shortcodes = true;
    options.extension.wikilinks_title_after_pipe = true;
    options.extension.multiline_block_quotes = true;
    if front_matter {
        options.extension.front_matter_delimiter = Some("---".into());
    }

    options
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_children(node: &Value) -> String {
    node.get("children")
        .and_then(|c| c.as_array())
        .map(|arr| arr.iter().map(|child| ast_to_html(child)).collect::<String>())
        .unwrap_or_default()
}

/// Renders an item's children in tight or non-tight mode.
/// In tight mode, top-level paragraph children are rendered without `<p>` wrappers.
fn render_item_content(item: &Value, tight: bool) -> String {
    item.get("children")
        .and_then(|c| c.as_array())
        .map(|children| {
            children
                .iter()
                .map(|child| {
                    if tight
                        && child.get("type").and_then(|t| t.as_str()) == Some("paragraph")
                    {
                        render_children(child)
                    } else {
                        ast_to_html(child)
                    }
                })
                .collect::<String>()
        })
        .unwrap_or_default()
}

fn ast_to_html(node: &Value) -> String {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match node_type {
        "document" => render_children(node),

        "paragraph" => format!("<p>{}</p>\n", render_children(node)),

        "heading" => {
            let level = node.get("level").and_then(|l| l.as_u64()).unwrap_or(1);
            format!("<h{level}>{}</h{level}>\n", render_children(node))
        }

        "code_block" => {
            let info = node.get("info").and_then(|i| i.as_str()).unwrap_or("text");
            let literal = node.get("literal").and_then(|l| l.as_str()).unwrap_or("");
            let class = if info.is_empty() || info == "text" {
                String::new()
            } else {
                format!(" class=\"language-{info}\"")
            };
            format!("<pre><code{class}>{}</code></pre>\n", escape_html(literal))
        }

        "block_quote" | "multiline_block_quote" => {
            format!("<blockquote>\n{}</blockquote>\n", render_children(node))
        }

        // Page metadata is not part of the rendered content.
        "front_matter" => String::new(),

        "list" => {
            let list_type = node
                .get("list_type")
                .and_then(|t| t.as_str())
                .unwrap_or("bullet");
            let start = node.get("start").and_then(|s| s.as_u64()).unwrap_or(1);
            let tight = node
                .get("tight")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);
            let items_html = node
                .get("children")
                .and_then(|c| c.as_array())
                .map(|items| {
                    items
                        .iter()
                        .map(|item| {
                            let item_type =
                                item.get("type").and_then(|t| t.as_str()).unwrap_or("");
                            let content = render_item_content(item, tight);
                            match item_type {
                                "task_item" => {
                                    let checked = item
                                        .get("symbol")
                                        .and_then(|s| s.as_str())
                                        .is_some();
                                    let checkbox = if checked {
                                        "<input type=\"checkbox\" checked disabled>"
                                    } else {
                                        "<input type=\"checkbox\" disabled>"
                                    };
                                    format!("<li>{checkbox}{content}</li>\n")
                                }
                                _ => format!("<li>{content}</li>\n"),
                            }
                        })
                        .collect::<String>()
                })
                .unwrap_or_default();
            if list_type == "ordered" {
                if start == 1 {
                    format!("<ol>\n{items_html}</ol>\n")
                } else {
                    format!("<ol start=\"{start}\">\n{items_html}</ol>\n")
                }
            } else {
                format!("<ul>\n{items_html}</ul>\n")
            }
        }

        // Fallback — normally rendered inside the list handler
        "item" => format!("<li>{}</li>\n", render_children(node)),
        "task_item" => {
            let checked = node.get("symbol").and_then(|s| s.as_str()).is_some();
            let checkbox = if checked {
                "<input type=\"checkbox\" checked disabled>"
            } else {
                "<input type=\"checkbox\" disabled>"
            };
            format!("<li>{checkbox}{}</li>\n", render_children(node))
        }

        "table" => {
            let mut thead = String::new();
            let mut tbody = String::new();
            if let Some(rows) = node.get("children").and_then(|c| c.as_array()) {
                for row in rows {
                    let is_header =
                        row.get("header").and_then(|h| h.as_bool()).unwrap_or(false);
                    let tag = if is_header { "th" } else { "td" };
                    let row_html = row
                        .get("children")
                        .and_then(|c| c.as_array())
                        .map(|cells| {
                            let cells_html: String = cells
                                .iter()
                                .map(|cell| {
                                    format!("<{tag}>{}</{tag}>", render_children(cell))
                                })
                                .collect();
                            format!("<tr>{cells_html}</tr>\n")
                        })
                        .unwrap_or_default();
                    if is_header {
                        thead.push_str(&row_html);
                    } else {
                        tbody.push_str(&row_html);
                    }
                }
            }
            let mut result = "<table>\n".to_string();
            if !thead.is_empty() {
                result.push_str(&format!("<thead>\n{thead}</thead>\n"));
            }
            if !tbody.is_empty() {
                result.push_str(&format!("<tbody>\n{tbody}</tbody>\n"));
            }
            result.push_str("</table>\n");
            result
        }

        "table_row" | "table_cell" => render_children(node),

        "thematic_break" => "<hr />\n".to_string(),

        "html_block" => node
            .get("literal")
            .and_then(|l| l.as_str())
            .unwrap_or("")
            .to_string(),

        "footnote_definition" => {
            let name = escape_html(node.get("name").and_then(|n| n.as_str()).unwrap_or(""));
            let label = format!("<a href=\"#fnref-{name}\" class=\"footnote-label\">[{name}]:</a>");
            let children = node.get("children").and_then(|c| c.as_array());
            // The label goes inside the first paragraph so that the body continues on
            // the same line without any CSS. A definition that starts with another
            // block keeps the label in front of it.
            let body = match children.and_then(|c| c.split_first()) {
                Some((first, rest)) if first.get("type").and_then(|t| t.as_str()) == Some("paragraph") => {
                    let rest: String = rest.iter().map(ast_to_html).collect();
                    format!("<p>{label} {}</p>\n{rest}", render_children(first))
                }
                _ => format!("{label}\n{}", render_children(node)),
            };
            format!("<div id=\"fn-{name}\" class=\"footnote\">\n{body}</div>\n")
        }

        "alert" => {
            let alert_type = node
                .get("alert_type")
                .and_then(|t| t.as_str())
                .unwrap_or("note");
            let title = node
                .get("title")
                .filter(|t| !t.is_null())
                .and_then(|t| t.as_str())
                .unwrap_or(match alert_type {
                    "tip" => "Tip",
                    "important" => "Important",
                    "warning" => "Warning",
                    "caution" => "Caution",
                    _ => "Note",
                });
            format!(
                "<blockquote class=\"alert alert-{alert_type}\">\n<p class=\"alert-title\">{title}</p>\n{}</blockquote>\n",
                render_children(node)
            )
        }

        "subtext" => format!("<sub>{}</sub>", render_children(node)),

        "text" => {
            escape_html(node.get("value").and_then(|v| v.as_str()).unwrap_or(""))
        }

        "softbreak" => "\n".to_string(),
        "linebreak" => "<br />\n".to_string(),

        "emph" => format!("<em>{}</em>", render_children(node)),
        "strong" => format!("<strong>{}</strong>", render_children(node)),
        "strikethrough" => format!("<del>{}</del>", render_children(node)),
        "underline" => format!("<u>{}</u>", render_children(node)),
        "highlight" => format!("<mark>{}</mark>", render_children(node)),
        "superscript" => format!("<sup>{}</sup>", render_children(node)),
        "subscript" => format!("<sub>{}</sub>", render_children(node)),
        "spoilered_text" => format!("<span class=\"spoiler\">{}</span>", render_children(node)),

        "code" => {
            let literal = node
                .get("literal")
                .and_then(|l| l.as_str())
                .unwrap_or("");
            format!("<code>{}</code>", escape_html(literal))
        }

        "link" => {
            let url = node.get("url").and_then(|u| u.as_str()).unwrap_or("");
            let title = node.get("title").and_then(|t| t.as_str()).unwrap_or("");
            if title.is_empty() {
                format!(
                    "<a href=\"{}\">{}</a>",
                    escape_html(url),
                    render_children(node)
                )
            } else {
                format!(
                    "<a href=\"{}\" title=\"{}\">{}</a>",
                    escape_html(url),
                    escape_html(title),
                    render_children(node)
                )
            }
        }

        "embed" => {
            let url = node.get("url").and_then(|u| u.as_str()).unwrap_or("");
            let alt = render_children(node);
            format!("<img src=\"{}\" alt=\"{}\" />", escape_html(url), alt)
        }

        "wikilink" => {
            let url = node.get("url").and_then(|u| u.as_str()).unwrap_or("");
            format!(
                "<a href=\"{}\" data-wikilink=\"true\">{}</a>",
                escape_html(url),
                render_children(node)
            )
        }

        "hashtag" => {
            let value = node.get("value").and_then(|v| v.as_str()).unwrap_or("");
            format!(
                "<a href=\"{}\" class=\"hashtag\">#{value}</a>",
                escape_html(value)
            )
        }

        // The label is shown instead of comrak's index: a body is parsed alone, so the
        // index would restart at 1 in every item.
        "footnote_reference" => {
            let name = escape_html(node.get("name").and_then(|n| n.as_str()).unwrap_or(""));
            format!("<sup id=\"fnref-{name}\" class=\"footnote-ref\"><a href=\"#fn-{name}\">[{name}]</a></sup>")
        }

        "shortcode" => node
            .get("emoji")
            .and_then(|e| e.as_str())
            .unwrap_or("")
            .to_string(),

        "math" => {
            let literal = node
                .get("literal")
                .and_then(|l| l.as_str())
                .unwrap_or("");
            format!("<span class=\"math\">{}</span>", escape_html(literal))
        }

        "html_inline" => node
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),

        "raw" => node
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),

        "escaped" => String::new(),

        "escaped_tag" => {
            let value = node.get("value").and_then(|v| v.as_str()).unwrap_or("");
            format!("&lt;{value}&gt;")
        }

        _ => render_children(node),
    }
}

/// Renders a flatmarkdown item body as an HTML string using the given options.
pub fn body_to_html_with(input: &str, opts: &BodyOptions) -> String {
    let ast_str = body_to_ast_with(input, opts);
    let ast: Value = serde_json::from_str(&ast_str).unwrap();
    ast_to_html(&ast)
}

pub fn body_to_html(input: &str) -> String {
    body_to_html_with(input, &BodyOptions::default())
}

/// Placeholder for escaped `\#` – Unicode Private Use Area character
/// unlikely to appear in normal text.
const ESCAPED_HASH_PLACEHOLDER: char = '\u{FDD0}';

/// Strips a matching prefix from `url` and removes the `.md` suffix, returning the wikilink URL.
/// Returns `None` if no prefix matches or the URL does not end with `.md`.
fn markdown_link_to_wikilink_url(url: &str, prefixes: &[String]) -> Option<String> {
    let without_prefix = prefixes.iter().find_map(|p| url.strip_prefix(p.as_str()))?;
    let (path_part, fragment) = match without_prefix.find('#') {
        Some(pos) => (&without_prefix[..pos], &without_prefix[pos..]),
        None => (without_prefix, ""),
    };
    let page_path = path_part.strip_suffix(".md")?;
    Some(format!("{}{}", page_path, fragment))
}

/// Returns `true` if the `link` node represents a serialised tag:
/// its only child is a text node whose value is `#` + `wikilink_url`.
fn is_tag_link(link_node: &Value, wikilink_url: &str) -> bool {
    let expected = format!("#{}", wikilink_url);
    link_node
        .get("children")
        .and_then(|c| c.as_array())
        .filter(|c| c.len() == 1)
        .and_then(|c| c.first())
        .filter(|n| n.get("type").and_then(|t| t.as_str()) == Some("text"))
        .and_then(|n| n.get("value").and_then(|v| v.as_str()))
        .map_or(false, |s| s == expected)
}

/// Walks the AST and converts `link` nodes whose URL starts with one of `prefixes`
/// into `wikilink` or `hashtag` nodes. The matching prefix and `.md` suffix are stripped.
/// A link of the form `[#tagname](<prefix>tagname.md)` becomes a `hashtag` node;
/// all other matching links become `wikilink` nodes.
fn convert_markdown_links_to_wikilinks(node: &mut Value, prefixes: &[String]) {
    if node.get("type").and_then(|t| t.as_str()) == Some("link") {
        if let Some(url) = node.get("url").and_then(|u| u.as_str()) {
            if let Some(wikilink_url) = markdown_link_to_wikilink_url(url, prefixes) {
                let is_tag = is_tag_link(node, &wikilink_url);
                if let Some(obj) = node.as_object_mut() {
                    if is_tag {
                        obj.insert("type".into(), Value::String("hashtag".into()));
                        obj.insert("value".into(), Value::String(wikilink_url));
                        obj.remove("url");
                        obj.remove("title");
                        obj.remove("children");
                    } else {
                        obj.insert("type".into(), Value::String("wikilink".into()));
                        obj.insert("url".into(), Value::String(wikilink_url));
                        obj.remove("title");
                    }
                }
                return;
            }
        }
    }
    if let Some(Value::Array(children)) = node.get_mut("children") {
        for child in children.iter_mut() {
            convert_markdown_links_to_wikilinks(child, prefixes);
        }
    }
}

/// Controls whether and how relative Markdown links are resolved.
#[derive(Default)]
pub enum LinkResolution {
    /// No conversion — links remain as `link` nodes, `#tag` text remains plain.
    #[default]
    Off,
    /// Convert markdown links to `wikilink` / `hashtag` nodes when the URL starts with
    /// one of the given prefixes. The matching prefix and `.md` suffix are stripped
    /// to form the wikilink URL.
    On { prefixes: Vec<String> },
}

/// Options for parsing a flatmarkdown item body.
#[derive(Default)]
pub struct BodyOptions {
    pub link_resolution: LinkResolution,
}

/// Marks the placeholder text that completes unpaired footnotes before parsing.
/// A noncharacter is used because it cannot occur in valid text.
const FOOTNOTE_PLACEHOLDER: char = '\u{FDD1}';

/// Footnote labels found by a raw scan of a body: `[^label]` occurrences and
/// `[^label]:` definition lines. The scan does not know about code spans or
/// code blocks, so a label may be a false positive. That is harmless, because the
/// completion only adds placeholders that comrak drops again when they stay unpaired.
struct FootnoteLabels {
    refs: Vec<String>,
    defs: Vec<String>,
}

/// Reads a footnote label after `[^` at `start`. Returns the label and the index
/// just past the closing `]`. Labels may not be empty or contain whitespace or `[`.
fn read_footnote_label(s: &str, start: usize) -> Option<(&str, usize)> {
    let rest = &s[start..];
    let end = rest.find(']')?;
    let label = &rest[..end];
    if label.is_empty() || label.chars().any(|c| c.is_whitespace() || c == '[') {
        return None;
    }
    Some((label, start + end + 1))
}

fn scan_footnote_labels(input: &str) -> FootnoteLabels {
    let mut labels = FootnoteLabels { refs: Vec::new(), defs: Vec::new() };
    for line in input.lines() {
        // A definition may be indented by up to three spaces, like any block start.
        let trimmed = line.trim_start_matches(' ');
        let indent = line.len() - trimmed.len();
        let def_label = if indent <= 3 && trimmed.starts_with("[^") {
            read_footnote_label(trimmed, 2)
                .filter(|(_, end)| trimmed[*end..].starts_with(':'))
                .map(|(label, _)| label.to_string())
        } else {
            None
        };

        let mut pos = 0;
        while let Some(found) = line[pos..].find("[^") {
            let label_start = pos + found + 2;
            match read_footnote_label(line, label_start) {
                Some((label, end)) => {
                    let is_definition = def_label.as_deref() == Some(label) && label_start == indent + 2;
                    if !is_definition {
                        labels.refs.push(label.to_string());
                    }
                    pos = end;
                }
                None => pos = label_start,
            }
        }
        if let Some(label) = def_label {
            labels.defs.push(label);
        }
    }
    labels
}

/// Collects the `name` of every node of `node_type` in the AST.
fn collect_footnote_names(node: &Value, node_type: &str, out: &mut HashSet<String>) {
    if node.get("type").and_then(|t| t.as_str()) == Some(node_type)
        && let Some(name) = node.get("name").and_then(|n| n.as_str())
    {
        out.insert(name.to_string());
    }
    if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
        for child in children {
            collect_footnote_names(child, node_type, out);
        }
    }
}

/// Builds the text that completes the unpaired footnotes of `input`, or returns
/// `None` when every footnote is already paired.
///
/// comrak keeps a footnote only when its reference and its definition are in the
/// same document. An item body is parsed alone, so a body that holds only one of
/// the two loses it. The missing side is added as a placeholder in front of the
/// body: placeholder definitions first, then one paragraph holding the placeholder
/// references. The paragraph also separates the definitions from the body, so an
/// indented first line of the body cannot be read as a definition continuation.
/// Placing the placeholders after the body is not an option, because a body that
/// ends in an unclosed code fence would swallow them.
fn complete_footnotes(input: &str, ast: &Value) -> Option<(String, HashSet<String>)> {
    let labels = scan_footnote_labels(input);
    let mut real_defs = HashSet::new();
    let mut real_refs = HashSet::new();
    collect_footnote_names(ast, "footnote_definition", &mut real_defs);
    collect_footnote_names(ast, "footnote_reference", &mut real_refs);

    let mut missing_defs: Vec<String> = Vec::new();
    for label in labels.refs {
        if !real_defs.contains(&label) && !missing_defs.contains(&label) {
            missing_defs.push(label);
        }
    }
    let mut missing_refs: Vec<String> = Vec::new();
    for label in labels.defs {
        if !real_refs.contains(&label) && !missing_refs.contains(&label) {
            missing_refs.push(label);
        }
    }
    if missing_defs.is_empty() && missing_refs.is_empty() {
        return None;
    }

    let mut completed = String::new();
    for label in &missing_defs {
        completed.push_str(&format!("[^{label}]: {FOOTNOTE_PLACEHOLDER}\n\n"));
    }
    completed.push(FOOTNOTE_PLACEHOLDER);
    for label in &missing_refs {
        completed.push_str(&format!("[^{label}]"));
    }
    completed.push_str("\n\n");
    completed.push_str(input);
    Some((completed, missing_defs.into_iter().collect()))
}

/// Removes the nodes that the placeholders of `complete_footnotes` produced.
fn strip_footnote_placeholders(ast: &mut Value, placeholder_defs: &HashSet<String>) {
    let Some(Value::Array(children)) = ast.get_mut("children") else {
        return;
    };
    children.retain(|child| {
        let node_type = child.get("type").and_then(|t| t.as_str());
        match node_type {
            Some("footnote_definition") => {
                let name = child.get("name").and_then(|n| n.as_str()).unwrap_or("");
                !placeholder_defs.contains(name)
            }
            Some("paragraph") => {
                let first_text = child
                    .get("children")
                    .and_then(|c| c.as_array())
                    .and_then(|c| c.first())
                    .and_then(|n| n.get("value"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                !first_text.starts_with(FOOTNOTE_PLACEHOLDER)
            }
            _ => true,
        }
    });
}

/// Parses a flatmarkdown item body into a JSON AST value using the given options.
fn body_to_ast_value(input: &str, opts: &BodyOptions) -> Value {
    // `\#` is replaced before comrak sees it so that the hashtag pass cannot
    // mistake the escaped `#` for a tag start.
    let preprocessed = input.replace("\\#", &ESCAPED_HASH_PLACEHOLDER.to_string());
    let arena = Arena::new();
    let root = parse_document(&arena, &preprocessed, &options(false));
    let mut ast = node_to_ast(root);
    if let Some((completed, placeholder_defs)) = complete_footnotes(&preprocessed, &ast) {
        let arena = Arena::new();
        let root = parse_document(&arena, &completed, &options(false));
        ast = node_to_ast(root);
        strip_footnote_placeholders(&mut ast, &placeholder_defs);
    }
    if let LinkResolution::On { prefixes } = &opts.link_resolution {
        convert_markdown_links_to_wikilinks(&mut ast, prefixes);
        process_hashtags(&mut ast);
    }
    restore_escaped_hashes(&mut ast);
    ast
}

/// Parses a flatmarkdown item body into a JSON AST string using the given options.
pub fn body_to_ast_with(input: &str, opts: &BodyOptions) -> String {
    serde_json::to_string(&body_to_ast_value(input, opts)).unwrap()
}

pub fn body_to_ast(input: &str) -> String {
    body_to_ast_with(input, &BodyOptions::default())
}

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
    let inner = line.trim_end().strip_prefix("<!-- @")?.strip_suffix(" -->")?;
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
    Some(ItemMetadata { level, id, properties })
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
    let meta = meta.unwrap_or(ItemMetadata { level: 1, id: None, properties: Vec::new() });
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
    let mut open = OpenItem { meta: None, start: 0, has_block: false };

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
            open = OpenItem { meta: Some(meta), start: end + 1, has_block: false };
        } else if open.has_block {
            close(open, start, &mut items);
            open = OpenItem { meta: None, start, has_block: true };
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

/// Returns true if `c` is a valid hashtag character:
/// Unicode alphanumeric, `_`, `-`, or `/`.
fn is_hashtag_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '/'
}

/// Splits a text string into a sequence of `text` and `hashtag` nodes.
/// A hashtag starts with `#` at the beginning of the string or after a half-width space (U+0020),
/// and ends when a non-hashtag character is encountered.
/// `/` is not allowed as the first or last character of the tag name.
fn split_hashtags(text: &str) -> Vec<Value> {
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut buf_start = 0;

    while i < chars.len() {
        if chars[i] == '#' && (i == 0 || chars[i - 1] == ' ') {
            // Collect hashtag name
            let tag_start = i + 1;
            let mut tag_end = tag_start;
            while tag_end < chars.len() && is_hashtag_char(chars[tag_end]) {
                tag_end += 1;
            }

            // Trim trailing '/'
            let mut actual_end = tag_end;
            while actual_end > tag_start && chars[actual_end - 1] == '/' {
                actual_end -= 1;
            }

            // Reject if empty after trimming or starts with '/'
            if actual_end > tag_start && chars[tag_start] != '/' {
                // Flush preceding text
                if i > buf_start {
                    let s: String = chars[buf_start..i].iter().collect();
                    result.push(json!({"type": "text", "value": s}));
                }
                let tag: String = chars[tag_start..actual_end].iter().collect();
                result.push(json!({"type": "hashtag", "value": tag}));
                i = actual_end;
                buf_start = i;
                continue;
            }
        }
        i += 1;
    }

    // Flush remaining text
    if buf_start < chars.len() {
        let s: String = chars[buf_start..].iter().collect();
        result.push(json!({"type": "text", "value": s}));
    }

    result
}

/// Restores the escaped-hash placeholder back to `#` in all text node values.
fn restore_escaped_hashes(node: &mut Value) {
    if let Some(val) = node.get_mut("value") {
        if let Some(s) = val.as_str() {
            if s.contains(ESCAPED_HASH_PLACEHOLDER) {
                *val = Value::String(s.replace(ESCAPED_HASH_PLACEHOLDER, "#"));
            }
        }
    }
    if let Some(Value::Array(children)) = node.get_mut("children") {
        for child in children.iter_mut() {
            restore_escaped_hashes(child);
        }
    }
}

/// Post-processes the AST to extract hashtags from text nodes.
fn process_hashtags(node: &mut Value) {
    if let Some(Value::Array(children)) = node.get_mut("children") {
        // Recurse first
        for child in children.iter_mut() {
            process_hashtags(child);
        }

        // Split text nodes containing hashtags
        let mut new_children: Vec<Value> = Vec::with_capacity(children.len());
        for child in children.drain(..) {
            if child.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(text) = child.get("value").and_then(|v| v.as_str()) {
                    let parts = split_hashtags(text);
                    if parts.len() > 1 || parts.first()
                        .and_then(|p| p.get("type"))
                        .and_then(|t| t.as_str()) == Some("hashtag")
                    {
                        new_children.extend(parts);
                        continue;
                    }
                }
            }
            new_children.push(child);
        }

        *children = new_children;
    }
}

fn node_to_ast<'a>(node: &'a comrak::arena_tree::Node<'a, std::cell::RefCell<comrak::nodes::Ast>>) -> Value {
    let ast = node.data.borrow();
    let (node_type, attrs) = serialize_node_value(&ast.value);

    let children: Vec<Value> = node.children().map(node_to_ast).collect();

    let mut obj = serde_json::Map::new();
    obj.insert("type".into(), Value::String(node_type));
    if !attrs.is_null() {
        if let Value::Object(map) = attrs {
            for (k, v) in map {
                obj.insert(k, v);
            }
        }
    }
    if !children.is_empty() {
        obj.insert("children".into(), Value::Array(children));
    }

    Value::Object(obj)
}

fn parse_embed_url(url: &str) -> (String, serde_json::Map<String, Value>) {
    match url.find('#') {
        Some(pos) => {
            let base = url[..pos].to_string();
            let props = url[pos + 1..]
                .split('&')
                .filter(|s| !s.is_empty())
                .map(|kv| match kv.find('=') {
                    Some(eq) => (kv[..eq].to_string(), Value::String(kv[eq + 1..].to_string())),
                    None => (kv.to_string(), Value::Bool(true)),
                })
                .collect();
            (base, props)
        }
        None => (url.to_string(), serde_json::Map::new()),
    }
}

fn serialize_node_value(value: &NodeValue) -> (String, Value) {
    match value {
        NodeValue::Document => ("document".into(), Value::Null),
        NodeValue::FrontMatter(s) => ("front_matter".into(), json!({ "value": s })),
        NodeValue::BlockQuote => ("block_quote".into(), Value::Null),
        NodeValue::List(nl) => ("list".into(), json!({
            "list_type": match nl.list_type {
                comrak::nodes::ListType::Bullet => "bullet",
                comrak::nodes::ListType::Ordered => "ordered",
            },
            "start": nl.start,
            "tight": nl.tight,
            "delimiter": match nl.delimiter {
                comrak::nodes::ListDelimType::Period => "period",
                comrak::nodes::ListDelimType::Paren => "paren",
            },
        })),
        NodeValue::Item(nl) => ("item".into(), json!({
            "list_type": match nl.list_type {
                comrak::nodes::ListType::Bullet => "bullet",
                comrak::nodes::ListType::Ordered => "ordered",
            },
            "start": nl.start,
            "tight": nl.tight,
        })),
        NodeValue::DescriptionList => ("description_list".into(), Value::Null),
        NodeValue::DescriptionItem(_) => ("description_item".into(), Value::Null),
        NodeValue::DescriptionTerm => ("description_term".into(), Value::Null),
        NodeValue::DescriptionDetails => ("description_details".into(), Value::Null),
        NodeValue::CodeBlock(cb) => ("code_block".into(), json!({
            "info": cb.info,
            "literal": cb.literal,
        })),
        NodeValue::HtmlBlock(hb) => ("html_block".into(), json!({
            "block_type": hb.block_type,
            "literal": hb.literal,
        })),
        NodeValue::HeexBlock(_) => ("heex_block".into(), Value::Null),
        NodeValue::Paragraph => ("paragraph".into(), Value::Null),
        NodeValue::Heading(h) => ("heading".into(), json!({
            "level": h.level,
        })),
        NodeValue::ThematicBreak => ("thematic_break".into(), Value::Null),
        NodeValue::FootnoteDefinition(fd) => ("footnote_definition".into(), json!({
            "name": fd.name,
        })),
        NodeValue::Table(t) => ("table".into(), json!({
            "alignments": t.alignments.iter().map(|a| match a {
                comrak::nodes::TableAlignment::None => "none",
                comrak::nodes::TableAlignment::Left => "left",
                comrak::nodes::TableAlignment::Center => "center",
                comrak::nodes::TableAlignment::Right => "right",
            }).collect::<Vec<_>>(),
            "num_columns": t.num_columns,
            "num_rows": t.num_rows,
        })),
        NodeValue::TableRow(header) => ("table_row".into(), json!({
            "header": header,
        })),
        NodeValue::TableCell => ("table_cell".into(), Value::Null),
        NodeValue::Text(s) => ("text".into(), json!({ "value": s.as_ref() })),
        NodeValue::TaskItem(ti) => ("task_item".into(), json!({
            "symbol": ti.symbol.map(|c| c.to_string()),
        })),
        NodeValue::SoftBreak => ("softbreak".into(), Value::Null),
        NodeValue::LineBreak => ("linebreak".into(), Value::Null),
        NodeValue::Code(c) => ("code".into(), json!({
            "literal": c.literal,
        })),
        NodeValue::HtmlInline(s) => ("html_inline".into(), json!({ "value": s })),
        NodeValue::HeexInline(s) => ("heex_inline".into(), json!({ "value": s })),
        NodeValue::Raw(s) => ("raw".into(), json!({ "value": s })),
        NodeValue::Emph => ("emph".into(), Value::Null),
        NodeValue::Strong => ("strong".into(), Value::Null),
        NodeValue::Strikethrough => ("strikethrough".into(), Value::Null),
        NodeValue::Highlight => ("highlight".into(), Value::Null),
        NodeValue::Superscript => ("superscript".into(), Value::Null),
        NodeValue::Link(link) => ("link".into(), json!({
            "url": link.url,
            "title": link.title,
        })),
        NodeValue::Image(link) => {
            let (url, props) = parse_embed_url(link.url.as_str());
            let mut attrs = json!({ "url": url, "title": link.title });
            if !props.is_empty() {
                attrs.as_object_mut().unwrap().insert("props".into(), Value::Object(props));
            }
            ("embed".into(), attrs)
        }
        NodeValue::FootnoteReference(fr) => ("footnote_reference".into(), json!({
            "name": fr.name,
            "ref_num": fr.ref_num,
            "ix": fr.ix,
        })),
        NodeValue::ShortCode(sc) => ("shortcode".into(), json!({
            "code": sc.code,
            "emoji": sc.emoji,
        })),
        NodeValue::Math(m) => ("math".into(), json!({
            "literal": m.literal,
        })),
        NodeValue::Escaped => ("escaped".into(), Value::Null),
        NodeValue::WikiLink(wl) => ("wikilink".into(), json!({
            "url": wl.url,
        })),
        NodeValue::Underline => ("underline".into(), Value::Null),
        NodeValue::Subscript => ("subscript".into(), Value::Null),
        NodeValue::SpoileredText => ("spoilered_text".into(), Value::Null),
        NodeValue::EscapedTag(s) => ("escaped_tag".into(), json!({ "value": s })),
        NodeValue::Alert(a) => ("alert".into(), json!({
            "alert_type": format!("{:?}", a.alert_type).to_lowercase(),
            "title": a.title,
        })),
        NodeValue::Subtext => ("subtext".into(), Value::Null),
        NodeValue::Insert => ("insert".into(), Value::Null),
        NodeValue::MultilineBlockQuote(_) => ("multiline_block_quote".into(), Value::Null),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_paragraph() {
        let result = body_to_html("Hello, world!");
        assert_eq!(result, "<p>Hello, world!</p>\n");
    }

    #[test]
    fn heading() {
        let result = body_to_html("# Title");
        assert_eq!(result, "<h1>Title</h1>\n");
    }

    #[test]
    fn strikethrough() {
        let result = body_to_html("~~deleted~~");
        assert_eq!(result, "<p><del>deleted</del></p>\n");
    }

    #[test]
    fn tasklist() {
        let result = body_to_html("- [x] done\n- [ ] todo");
        assert!(result.contains("checked"));
        assert!(result.contains("type=\"checkbox\""));
    }

    // SSoT: a plain newline is a GFM soft break, not a hard break.
    #[test]
    fn softbreak_html() {
        let result = body_to_html("line1\nline2");
        assert_eq!(result, "<p>line1\nline2</p>\n");
    }

    // SSoT: GFM hard line break (trailing backslash) renders as <br />.
    #[test]
    fn hardbreak_html() {
        let result = body_to_html("line1\\\nline2");
        assert_eq!(result, "<p>line1<br />\nline2</p>\n");
    }

    // SSoT: multiline block quote (>>>) is available.
    #[test]
    fn multiline_block_quote_html() {
        let result = body_to_html(">>>\nq1\n\nq2\n>>>");
        assert_eq!(result, "<blockquote>\n<p>q1</p>\n<p>q2</p>\n</blockquote>\n");
    }

    #[test]
    fn ast_multiline_block_quote() {
        let result = body_to_ast(">>>\nq1\n\nq2\n>>>");
        let v: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["children"][0]["type"], "multiline_block_quote");
        assert_eq!(v["children"][0]["children"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn ast_basic() {
        let result = body_to_ast("Hello");
        let v: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["type"], "document");
        assert_eq!(v["children"][0]["type"], "paragraph");
        assert_eq!(v["children"][0]["children"][0]["type"], "text");
        assert_eq!(v["children"][0]["children"][0]["value"], "Hello");
    }

    #[test]
    fn ast_heading() {
        let result = body_to_ast("## Sub");
        let v: Value = serde_json::from_str(&result).unwrap();
        let heading = &v["children"][0];
        assert_eq!(heading["type"], "heading");
        assert_eq!(heading["level"], 2);
        assert_eq!(heading["children"][0]["value"], "Sub");
    }

    #[test]
    fn ast_link() {
        let result = body_to_ast("[text](https://example.com)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let link = &v["children"][0]["children"][0];
        assert_eq!(link["type"], "link");
        assert_eq!(link["url"], "https://example.com");
        assert_eq!(link["children"][0]["value"], "text");
    }

    #[test]
    fn ast_code_block() {
        let result = body_to_ast("```rust\nfn main() {}\n```");
        let v: Value = serde_json::from_str(&result).unwrap();
        let cb = &v["children"][0];
        assert_eq!(cb["type"], "code_block");
        assert_eq!(cb["info"], "rust");
        assert_eq!(cb["literal"], "fn main() {}\n");
    }

    #[test]
    fn wikilink_html_basic() {
        let result = body_to_html("[[page]]");
        assert_eq!(result, "<p><a href=\"page\" data-wikilink=\"true\">page</a></p>\n");
    }

    #[test]
    fn wikilink_html_with_label() {
        let result = body_to_html("[[url|link label]]");
        assert_eq!(result, "<p><a href=\"url\" data-wikilink=\"true\">link label</a></p>\n");
    }

    #[test]
    fn wikilink_html_inline() {
        let result = body_to_html("See [[page]] for details.");
        assert!(result.contains("<a href=\"page\" data-wikilink=\"true\">page</a>"));
        assert!(result.contains("See "));
        assert!(result.contains(" for details."));
    }

    #[test]
    fn wikilink_ast_basic() {
        let result = body_to_ast("[[page]]");
        let v: Value = serde_json::from_str(&result).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "page");
        assert_eq!(wl["children"][0]["type"], "text");
        assert_eq!(wl["children"][0]["value"], "page");
    }

    #[test]
    fn wikilink_ast_with_label() {
        let result = body_to_ast("[[url|link label]]");
        let v: Value = serde_json::from_str(&result).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "url");
        assert_eq!(wl["children"][0]["value"], "link label");
    }

    #[test]
    fn wikilink_multiple() {
        let result = body_to_html("[[page1]] and [[page2]]");
        assert!(result.contains("<a href=\"page1\" data-wikilink=\"true\">page1</a>"));
        assert!(result.contains("<a href=\"page2\" data-wikilink=\"true\">page2</a>"));
    }

    #[test]
    fn ast_hashtag_basic() {
        let result = body_to_ast_with("#tag", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "tag");
    }

    #[test]
    fn ast_hashtag_japanese() {
        let result = body_to_ast_with("#日記", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "日記");
    }

    #[test]
    fn ast_hashtag_with_special_chars() {
        let result = body_to_ast_with("#my_tag-name/sub", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "my_tag-name/sub");
    }

    #[test]
    fn ast_hashtag_in_text() {
        let result = body_to_ast_with("hello #tag world", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "hello ");
        assert_eq!(children[1]["type"], "hashtag");
        assert_eq!(children[1]["value"], "tag");
        assert_eq!(children[2]["type"], "text");
        assert_eq!(children[2]["value"], " world");
    }

    #[test]
    fn ast_hashtag_multiple() {
        let result = body_to_ast_with("#tag1 #tag2", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "tag1");
        assert_eq!(children[1]["type"], "text");
        assert_eq!(children[1]["value"], " ");
        assert_eq!(children[2]["type"], "hashtag");
        assert_eq!(children[2]["value"], "tag2");
    }

    #[test]
    fn ast_hashtag_not_in_middle_of_word() {
        let result = body_to_ast("foo#bar");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "foo#bar");
    }

    #[test]
    fn ast_hashtag_terminated_by_punctuation() {
        let result = body_to_ast_with("#タグ。本文", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "タグ");
        assert_eq!(children[1]["type"], "text");
        assert_eq!(children[1]["value"], "。本文");
    }

    #[test]
    fn ast_hashtag_leading_slash_not_hashtag() {
        // "#/tag" must not be recognized as a hashtag (leading '/' is invalid)
        let result = body_to_ast("#/tag");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#/tag");
    }

    #[test]
    fn ast_hashtag_trailing_slash_trimmed() {
        // "#tag/" must be recognized as hashtag "tag" (trailing '/' is trimmed)
        let result = body_to_ast_with("#tag/", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "tag");
        assert_eq!(children[1]["type"], "text");
        assert_eq!(children[1]["value"], "/");
    }

    #[test]
    fn ast_hashtag_leading_and_trailing_slash_not_hashtag() {
        // "#/tag/" must not be recognized as a hashtag (leading '/' is invalid)
        let result = body_to_ast("#/tag/");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#/tag/");
    }

    #[test]
    fn ast_hash_only_not_hashtag() {
        let result = body_to_ast("# ");
        let v: Value = serde_json::from_str(&result).unwrap();
        // "# " is a heading, not a hashtag
        assert_eq!(v["children"][0]["type"], "heading");
    }

    #[test]
    fn ast_escaped_hashtag_basic() {
        let result = body_to_ast(r"\#tag");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#tag");
    }

    #[test]
    fn ast_escaped_hashtag_in_text() {
        let result = body_to_ast(r"hello \#tag world");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "hello #tag world");
    }

    #[test]
    fn ast_escaped_hashtag_with_real_hashtag() {
        let result = body_to_ast_with(r"\#notag #real", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#notag ");
        assert_eq!(children[1]["type"], "hashtag");
        assert_eq!(children[1]["value"], "real");
    }

    #[test]
    fn ast_escaped_hashtag_multiple() {
        let result = body_to_ast(r"\#a \#b");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#a #b");
    }

    #[test]
    fn ast_embed_no_props() {
        let result = body_to_ast("![alt text](img.png)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "img.png");
        assert_eq!(embed["props"], Value::Null);
        assert_eq!(embed["children"][0]["value"], "alt text");
    }

    #[test]
    fn ast_embed_with_kv_props() {
        let result = body_to_ast("![alt](img.png#w=300&h=240)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "img.png");
        assert_eq!(embed["props"]["w"], "300");
        assert_eq!(embed["props"]["h"], "240");
    }

    #[test]
    fn ast_embed_with_boolean_flag() {
        let result = body_to_ast("![alt](video.mp4#w=640&autoplay)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "video.mp4");
        assert_eq!(embed["props"]["w"], "640");
        assert_eq!(embed["props"]["autoplay"], true);
    }

    #[test]
    fn ast_embed_url_with_query_and_fragment() {
        let result = body_to_ast("![alt](https://youtube.com/watch?v=xxx#w=640&start=30)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "https://youtube.com/watch?v=xxx");
        assert_eq!(embed["props"]["w"], "640");
        assert_eq!(embed["props"]["start"], "30");
    }

    // SSoT: a plain newline produces a softbreak node (standard GFM).
    #[test]
    fn ast_newline_produces_softbreak() {
        let result = body_to_ast("line1\nline2");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "line1");
        assert_eq!(children[1]["type"], "softbreak");
        assert_eq!(children[2]["type"], "text");
        assert_eq!(children[2]["value"], "line2");
    }

    // SSoT: GFM hard line break syntax (trailing 2+ spaces) produces a linebreak.
    #[test]
    fn ast_gfm_trailing_spaces_produce_linebreak() {
        let result = body_to_ast("line1  \nline2");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["value"], "line1");
        assert_eq!(children[1]["type"], "linebreak");
        assert_eq!(children[2]["value"], "line2");
    }

    // SSoT: GFM hard line break syntax (trailing backslash) produces a linebreak.
    #[test]
    fn ast_gfm_trailing_backslash_produces_linebreak() {
        let result = body_to_ast("line1\\\nline2");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["value"], "line1");
        assert_eq!(children[1]["type"], "linebreak");
        assert_eq!(children[2]["value"], "line2");
    }

    // SSoT: Extended syntax - highlight (==text==)
    #[test]
    fn ast_highlight() {
        let result = body_to_ast("==highlighted==");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "highlight");
        assert_eq!(node["children"][0]["value"], "highlighted");
    }

    // SSoT: Extended syntax - superscript (^text^)
    #[test]
    fn ast_superscript() {
        let result = body_to_ast("^super^");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "superscript");
        assert_eq!(node["children"][0]["value"], "super");
    }

    // SSoT: Extended syntax - subscript (~text~)
    #[test]
    fn ast_subscript() {
        let result = body_to_ast("~sub~");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "subscript");
        assert_eq!(node["children"][0]["value"], "sub");
    }

    // SSoT: Extended syntax - spoilered_text (||text||)
    #[test]
    fn ast_spoilered_text() {
        let result = body_to_ast("||spoiler||");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "spoilered_text");
        assert_eq!(node["children"][0]["value"], "spoiler");
    }

    // SSoT: Extended syntax - underline (__text__ is underline, not strong)
    #[test]
    fn ast_underline() {
        let result = body_to_ast("__underlined__");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "underline");
        assert_eq!(node["children"][0]["value"], "underlined");
    }

    // SSoT: Extended syntax - shortcode (:name:)
    #[test]
    fn ast_shortcode() {
        let result = body_to_ast(":rabbit:");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "shortcode");
        assert_eq!(node["code"], "rabbit");
        assert!(node["emoji"].is_string());
    }

    // SSoT: Extended syntax - math ($`formula`$ syntax)
    #[test]
    fn ast_math_inline() {
        let result = body_to_ast("$`x^2`$");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "math");
        assert_eq!(node["literal"], "x^2");
    }

    // SSoT: Extended syntax - alert (> [!NOTE])
    #[test]
    fn ast_alert_note() {
        let result = body_to_ast("> [!NOTE]\n> Content");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0];
        assert_eq!(node["type"], "alert");
        assert_eq!(node["alert_type"], "note");
    }

    // SSoT: Extended syntax - footnote (reference + definition)
    #[test]
    fn ast_footnote() {
        let result = body_to_ast("Text[^1].\n\n[^1]: Footnote.");
        let v: Value = serde_json::from_str(&result).unwrap();
        let para_children = v["children"][0]["children"].as_array().unwrap();
        let fref = para_children.iter().find(|n| n["type"] == "footnote_reference");
        assert!(fref.is_some(), "expected footnote_reference in paragraph");
        assert_eq!(fref.unwrap()["name"], "1");
        let blocks = v["children"].as_array().unwrap();
        let fdef = blocks.iter().find(|n| n["type"] == "footnote_definition");
        assert!(fdef.is_some(), "expected footnote_definition block");
        assert_eq!(fdef.unwrap()["name"], "1");
    }

    // SSoT: Task list - checked item has symbol, unchecked has null
    #[test]
    fn ast_task_item_checked() {
        let result = body_to_ast("- [x] done");
        let v: Value = serde_json::from_str(&result).unwrap();
        let item = &v["children"][0]["children"][0];
        assert_eq!(item["type"], "task_item");
        assert_eq!(item["symbol"], "x");
    }

    #[test]
    fn ast_task_item_unchecked() {
        let result = body_to_ast("- [ ] todo");
        let v: Value = serde_json::from_str(&result).unwrap();
        let item = &v["children"][0]["children"][0];
        assert_eq!(item["type"], "task_item");
        assert_eq!(item["symbol"], Value::Null);
    }

    // SSoT: Wikilink block link [[page#id]]
    #[test]
    fn ast_wikilink_block_link() {
        let result = body_to_ast("[[page#block-id]]");
        let v: Value = serde_json::from_str(&result).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "page#block-id");
    }

    // SSoT: Wikilink block link with label [[page#id|label]]
    #[test]
    fn ast_wikilink_block_link_with_label() {
        let result = body_to_ast("[[page#block-id|label]]");
        let v: Value = serde_json::from_str(&result).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "page#block-id");
        assert_eq!(wl["children"][0]["value"], "label");
    }

    // SSoT: Markdown relative link → wikilink (no label: link text equals page name)
    #[test]
    fn ast_md_link_to_wikilink_no_label() {
        let result = body_to_ast_with("[pagename](../pagename.md)", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "pagename");
    }

    // SSoT: Markdown relative link → wikilink (with label)
    #[test]
    fn ast_md_link_to_wikilink_with_label() {
        let result = body_to_ast_with("[display label](../pagename.md)", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "display label");
    }

    // SSoT: Markdown relative link with path → wikilink URL preserves full path
    #[test]
    fn ast_md_link_to_wikilink_with_path() {
        let result = body_to_ast_with("[label](../journal/2024-01-01.md)", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "journal/2024-01-01");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Markdown relative link with fragment → wikilink URL includes #id
    #[test]
    fn ast_md_link_to_wikilink_with_fragment() {
        let result = body_to_ast_with("[label](../page.md#block-id)", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "page#block-id");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Markdown relative link with ./ prefix → wikilink
    #[test]
    fn ast_md_link_dotslash_to_wikilink() {
        let result = body_to_ast_with("[pagename](./pagename.md)", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "pagename");
    }

    // SSoT: Markdown relative link with ./ prefix and path → wikilink URL preserves path
    #[test]
    fn ast_md_link_dotslash_with_path() {
        let result = body_to_ast_with("[label](./journal/2024-01-01.md)", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "journal/2024-01-01");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Markdown relative link with ./ prefix and fragment → wikilink includes #id
    #[test]
    fn ast_md_link_dotslash_with_fragment() {
        let result = body_to_ast_with("[label](./page.md#block-id)", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "page#block-id");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Tag Markdown link with ./ prefix → hashtag node
    #[test]
    fn ast_md_tag_link_dotslash_to_hashtag() {
        let result = body_to_ast_with("[#mytag](./mytag.md)", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "hashtag");
        assert_eq!(node["value"], "mytag");
        assert!(node.get("url").is_none());
        assert!(node.get("children").is_none());
    }

    // SSoT: Tag Markdown link → hashtag node
    #[test]
    fn ast_md_tag_link_to_hashtag() {
        let result = body_to_ast_with("[#日記](../日記.md)", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "hashtag");
        assert_eq!(node["value"], "日記");
        assert!(node.get("url").is_none());
        assert!(node.get("children").is_none());
    }

    #[test]
    fn ast_md_tag_link_ascii() {
        let result = body_to_ast_with("[#mytag](../mytag.md)", &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] } });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "hashtag");
        assert_eq!(node["value"], "mytag");
    }

    // LinkResolution::Off: #tag in text stays as text node
    #[test]
    fn ast_no_resolve_hashtag_stays_text() {
        let result = body_to_ast("#tag");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert!(children.iter().all(|c| c["type"] != "hashtag"));
    }

    // LinkResolution::Off: [#tag](../tag.md) stays as link node
    #[test]
    fn ast_no_resolve_tag_link_stays_link() {
        let result = body_to_ast("[#tag](../tag.md)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // LinkResolution::Off: [text](../page.md) stays as link node
    #[test]
    fn ast_no_resolve_md_link_stays_link() {
        let result = body_to_ast("[text](../page.md)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // LinkResolution::On with empty prefixes: no links are converted
    #[test]
    fn ast_on_empty_prefixes_md_link_stays_link() {
        let result = body_to_ast_with(
            "[text](../page.md)",
            &BodyOptions { link_resolution: LinkResolution::On { prefixes: vec![] } },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // External links must not be converted to wikilinks
    #[test]
    fn ast_external_link_not_converted_to_wikilink() {
        let result = body_to_ast("[text](https://example.com)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // Relative links not ending in .md must not be converted to wikilinks
    #[test]
    fn ast_non_md_relative_link_not_converted() {
        let result = body_to_ast("[text](../page.html)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // SSoT: blank lines are no longer rewritten; a loose list stays one block.
    #[test]
    fn ast_blank_line_between_list_items_keeps_one_list() {
        let result = body_to_ast("- a\n\n- b");
        let v: Value = serde_json::from_str(&result).unwrap();
        let blocks = v["children"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "list");
        assert_eq!(blocks[0]["tight"], false);
    }

    #[test]
    fn ast_blank_line_in_code_fence_preserved() {
        let result = body_to_ast("```\ncode\n\nmore code\n```");
        let v: Value = serde_json::from_str(&result).unwrap();
        let cb = &v["children"][0];
        assert_eq!(cb["type"], "code_block");
        let literal = cb["literal"].as_str().unwrap();
        assert!(literal.contains("\n\n"), "blank line inside fence must be preserved");
    }

    // ---- Metadata line syntax ----

    const ID1: &str = "2026-09-26-18-14-47-32W15WT";
    const ID2: &str = "2026-09-26-18-15-30-6VZSYB4";
    const ID3: &str = "2026-09-27-15-00-16-W3N8ASY";

    #[test]
    fn metadata_level_only() {
        let m = parse_metadata_line("<!-- @ -->").unwrap();
        assert_eq!(m, ItemMetadata { level: 1, id: None, properties: vec![] });
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
        assert_eq!(m.properties, vec![("collapsed".to_string(), "true".to_string())]);
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

    fn page(input: &str) -> Value {
        serde_json::from_str(&page_to_ast(input)).unwrap()
    }

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
        assert_eq!(items[3]["ast"]["children"][0]["children"][0]["type"], "task_item");
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
            link_resolution: LinkResolution::On { prefixes: vec!["../".into()] },
        };
        let v: Value = serde_json::from_str(&page_to_ast_with("#tag [p](../p.md)", &opts)).unwrap();
        let inline = v["items"][0]["ast"]["children"][0]["children"].as_array().unwrap();
        assert_eq!(inline[0]["type"], "hashtag");
        assert_eq!(inline[2]["type"], "wikilink");
    }

    // ---- Footnotes split across items ----

    fn footnote_nodes(v: &Value, node_type: &str) -> Vec<String> {
        let mut out = HashSet::new();
        collect_footnote_names(v, node_type, &mut out);
        let mut names: Vec<String> = out.into_iter().collect();
        names.sort();
        names
    }

    fn contains_placeholder(v: &Value) -> bool {
        v.to_string().contains(FOOTNOTE_PLACEHOLDER)
    }

    #[test]
    fn ast_footnote_reference_without_definition_is_kept() {
        let v: Value = serde_json::from_str(&body_to_ast("Text[^note].")).unwrap();
        assert_eq!(footnote_nodes(&v, "footnote_reference"), vec!["note"]);
        assert!(footnote_nodes(&v, "footnote_definition").is_empty());
        assert_eq!(v["children"].as_array().unwrap().len(), 1);
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_definition_without_reference_is_kept() {
        let v: Value = serde_json::from_str(&body_to_ast("[^note]: The note.")).unwrap();
        assert_eq!(footnote_nodes(&v, "footnote_definition"), vec!["note"]);
        assert!(footnote_nodes(&v, "footnote_reference").is_empty());
        let blocks = v["children"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["children"][0]["children"][0]["value"], "The note.");
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_paired_in_one_body_is_unchanged() {
        let v: Value = serde_json::from_str(&body_to_ast("Text[^1].\n\n[^1]: Footnote.")).unwrap();
        assert_eq!(v["children"].as_array().unwrap().len(), 2);
        assert_eq!(footnote_nodes(&v, "footnote_reference"), vec!["1"]);
        assert_eq!(footnote_nodes(&v, "footnote_definition"), vec!["1"]);
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_definition_may_reference_another_footnote() {
        let v: Value = serde_json::from_str(&body_to_ast("[^a]: See [^b].")).unwrap();
        assert_eq!(footnote_nodes(&v, "footnote_definition"), vec!["a"]);
        assert_eq!(footnote_nodes(&v, "footnote_reference"), vec!["b"]);
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_syntax_in_code_stays_text() {
        let v: Value = serde_json::from_str(&body_to_ast("`[^1]`\n\n```\n[^2]: x\n```")).unwrap();
        assert!(footnote_nodes(&v, "footnote_reference").is_empty());
        assert!(footnote_nodes(&v, "footnote_definition").is_empty());
        assert_eq!(v["children"].as_array().unwrap().len(), 2);
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_reference_before_unclosed_fence() {
        let v: Value = serde_json::from_str(&body_to_ast("Text[^1].\n\n```\ncode")).unwrap();
        assert_eq!(footnote_nodes(&v, "footnote_reference"), vec!["1"]);
        assert_eq!(v["children"][1]["type"], "code_block");
        assert_eq!(v["children"][1]["literal"], "code");
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_definition_before_indented_code() {
        let v: Value = serde_json::from_str(&body_to_ast("    code\n\n[^1]: note")).unwrap();
        assert_eq!(v["children"][0]["type"], "code_block");
        assert_eq!(v["children"][0]["literal"], "code\n");
        assert_eq!(footnote_nodes(&v, "footnote_definition"), vec!["1"]);
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn page_footnote_definition_after_another_block_is_not_duplicated() {
        let v = page("Text[^1].\n\nMiddle.\n\n[^1]: Footnote.\n\nLast.\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0]["body"], "Text[^1].");
        assert_eq!(items[1]["body"], "Middle.\n\n[^1]: Footnote.");
        assert_eq!(items[2]["body"], "Last.");
        assert_eq!(footnote_nodes(&items[0]["ast"], "footnote_reference"), vec!["1"]);
        assert_eq!(footnote_nodes(&items[1]["ast"], "footnote_definition"), vec!["1"]);
    }
}
