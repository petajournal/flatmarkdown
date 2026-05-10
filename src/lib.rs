use comrak::nodes::NodeValue;
use comrak::{parse_document, Arena, Options};
use serde_json::{json, Value};

fn options() -> Options<'static> {
    let mut options = Options::default();

    // Render options
    options.render.hardbreaks = true;
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
    options.extension.greentext = true;
    options.extension.alerts = true;
    options.extension.cjk_friendly_emphasis = true;
    options.extension.subtext = true;
    options.extension.highlight = true;
    options.extension.shortcodes = true;
    options.extension.wikilinks_title_after_pipe = true;

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

        "block_quote" => format!("<blockquote>\n{}</blockquote>\n", render_children(node)),

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
            let name = node.get("name").and_then(|n| n.as_str()).unwrap_or("");
            format!(
                "<div id=\"fn-{name}\" class=\"footnote\">\n{}</div>\n",
                render_children(node)
            )
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

        "footnote_reference" => {
            let name = node.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let ix = node.get("ix").and_then(|i| i.as_u64()).unwrap_or(0);
            format!("<sup><a id=\"fnref-{name}\" href=\"#fn-{name}\">{ix}</a></sup>")
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

/// Replaces whitespace-only lines outside code fences with `<br />`.
/// Lines consisting entirely of Unicode White_Space characters (empty, space-only,
/// full-width-space-only, etc.) are normalised to `<br />` before Markdown parsing.
/// Blank lines inside code fences are passed through unchanged.
fn preprocess_body(input: &str) -> String {
    let mut lines_out: Vec<String> = Vec::new();
    let mut in_fence = false;
    let mut fence_char = '`';
    let mut fence_len: usize = 0;

    for line in input.lines() {
        if in_fence {
            let stripped = line.trim_start_matches(|c: char| c == ' ' || c == '\t');
            let close_len = stripped.chars().take_while(|&c| c == fence_char).count();
            let after_fence: String = stripped.chars().skip(close_len).collect();
            if close_len >= fence_len && after_fence.chars().all(|c| c == ' ' || c == '\t') {
                in_fence = false;
            }
            lines_out.push(line.to_string());
        } else {
            let stripped = line.trim_start_matches(|c: char| c == ' ' || c == '\t');
            let chars: Vec<char> = stripped.chars().collect();

            let backtick_count = chars.iter().take_while(|&&c| c == '`').count();
            let tilde_count = chars.iter().take_while(|&&c| c == '~').count();

            if backtick_count >= 3 && !chars[backtick_count..].contains(&'`') {
                in_fence = true;
                fence_char = '`';
                fence_len = backtick_count;
                lines_out.push(line.to_string());
            } else if tilde_count >= 3 {
                in_fence = true;
                fence_char = '~';
                fence_len = tilde_count;
                lines_out.push(line.to_string());
            } else if line.chars().all(|c| c.is_whitespace()) {
                lines_out.push("<br />".to_string());
            } else {
                lines_out.push(line.to_string());
            }
        }
    }

    lines_out.join("\n")
}

/// Converts a Markdown relative link URL (`../page_path.md`, `./page_path.md`, or with `#id`)
/// to a Wikilink URL (`page_path` or `page_path#id`).
/// Returns `None` if the URL does not match the Markdown-serialised wikilink pattern.
fn markdown_link_to_wikilink_url(url: &str) -> Option<String> {
    let without_prefix = url.strip_prefix("../").or_else(|| url.strip_prefix("./"))?;
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

/// Walks the AST and converts `link` nodes whose URL matches the Markdown-serialised
/// wikilink pattern (`../page.md` or `./page.md`) into `wikilink` or `hashtag` nodes.
/// A link of the form `[#tagname](../tagname.md)` or `[#tagname](./tagname.md)` becomes a `hashtag` node;
/// all other matching links become `wikilink` nodes.
fn convert_markdown_links_to_wikilinks(node: &mut Value) {
    if node.get("type").and_then(|t| t.as_str()) == Some("link") {
        if let Some(url) = node.get("url").and_then(|u| u.as_str()) {
            if let Some(wikilink_url) = markdown_link_to_wikilink_url(url) {
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
            convert_markdown_links_to_wikilinks(child);
        }
    }
}

/// Options for parsing a flatmarkdown item body.
#[derive(Default)]
pub struct BodyOptions {
    /// When `true`, `../page.md` links are converted to `wikilink` nodes,
    /// `[#tag](../tag.md)` links to `hashtag` nodes, and `#tag` text patterns
    /// are extracted as `hashtag` nodes.
    /// When `false` (the default), none of these conversions are applied.
    pub resolve_links: bool,
}

/// Parses a flatmarkdown item body into a JSON AST string using the given options.
pub fn body_to_ast_with(input: &str, opts: &BodyOptions) -> String {
    let preprocessed = preprocess_body(input);
    let preprocessed = preprocessed.replace("\\#", &ESCAPED_HASH_PLACEHOLDER.to_string());
    let arena = Arena::new();
    let root = parse_document(&arena, &preprocessed, &options());
    let mut ast = node_to_ast(root);
    if opts.resolve_links {
        convert_markdown_links_to_wikilinks(&mut ast);
        process_hashtags(&mut ast);
    }
    restore_escaped_hashes(&mut ast);
    serde_json::to_string(&ast).unwrap()
}

pub fn body_to_ast(input: &str) -> String {
    body_to_ast_with(input, &BodyOptions::default())
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
        // flatmarkdown always treats soft breaks as hard breaks (linebreak),
        NodeValue::SoftBreak => ("linebreak".into(), Value::Null),
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
        NodeValue::MultilineBlockQuote(_) => unreachable!("multiline_block_quotes extension is disabled"),
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

    #[test]
    fn hardbreaks() {
        let result = body_to_html("line1\nline2");
        assert!(result.contains("<br"));
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
        let result = body_to_ast_with("#tag", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "tag");
    }

    #[test]
    fn ast_hashtag_japanese() {
        let result = body_to_ast_with("#日記", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "日記");
    }

    #[test]
    fn ast_hashtag_with_special_chars() {
        let result = body_to_ast_with("#my_tag-name/sub", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "my_tag-name/sub");
    }

    #[test]
    fn ast_hashtag_in_text() {
        let result = body_to_ast_with("hello #tag world", &BodyOptions { resolve_links: true });
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
        let result = body_to_ast_with("#tag1 #tag2", &BodyOptions { resolve_links: true });
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
        let result = body_to_ast_with("#タグ。本文", &BodyOptions { resolve_links: true });
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
        let result = body_to_ast_with("#tag/", &BodyOptions { resolve_links: true });
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
        let result = body_to_ast_with(r"\#notag #real", &BodyOptions { resolve_links: true });
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

#[test]
    fn ast_newline_produces_linebreak() {
        let result = body_to_ast("line1\nline2");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "line1");
        assert_eq!(children[1]["type"], "linebreak");
        assert_eq!(children[2]["type"], "text");
        assert_eq!(children[2]["value"], "line2");
    }

    // SSoT: GFM hard line break syntax (trailing 2+ spaces) produces the same
    // single linebreak as a regular newline — not an additional break.
    #[test]
    fn ast_gfm_trailing_spaces_same_as_newline() {
        let result = body_to_ast("line1  \nline2");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["value"], "line1");
        assert_eq!(children[1]["type"], "linebreak");
        assert_eq!(children[2]["value"], "line2");
    }

    // SSoT: GFM hard line break syntax (trailing backslash) produces the same
    // single linebreak as a regular newline — not an additional break.
    #[test]
    fn ast_gfm_trailing_backslash_same_as_newline() {
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
        let result = body_to_ast_with("[pagename](../pagename.md)", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "pagename");
    }

    // SSoT: Markdown relative link → wikilink (with label)
    #[test]
    fn ast_md_link_to_wikilink_with_label() {
        let result = body_to_ast_with("[display label](../pagename.md)", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "display label");
    }

    // SSoT: Markdown relative link with path → wikilink URL preserves full path
    #[test]
    fn ast_md_link_to_wikilink_with_path() {
        let result = body_to_ast_with("[label](../journal/2024-01-01.md)", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "journal/2024-01-01");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Markdown relative link with fragment → wikilink URL includes #id
    #[test]
    fn ast_md_link_to_wikilink_with_fragment() {
        let result = body_to_ast_with("[label](../page.md#block-id)", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "page#block-id");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Markdown relative link with ./ prefix → wikilink
    #[test]
    fn ast_md_link_dotslash_to_wikilink() {
        let result = body_to_ast_with("[pagename](./pagename.md)", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "pagename");
    }

    // SSoT: Markdown relative link with ./ prefix and path → wikilink URL preserves path
    #[test]
    fn ast_md_link_dotslash_with_path() {
        let result = body_to_ast_with("[label](./journal/2024-01-01.md)", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "journal/2024-01-01");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Markdown relative link with ./ prefix and fragment → wikilink includes #id
    #[test]
    fn ast_md_link_dotslash_with_fragment() {
        let result = body_to_ast_with("[label](./page.md#block-id)", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "page#block-id");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Tag Markdown link with ./ prefix → hashtag node
    #[test]
    fn ast_md_tag_link_dotslash_to_hashtag() {
        let result = body_to_ast_with("[#mytag](./mytag.md)", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "hashtag");
        assert_eq!(node["value"], "mytag");
        assert!(node.get("url").is_none());
        assert!(node.get("children").is_none());
    }

    // SSoT: Tag Markdown link → hashtag node (resolve_links=true)
    #[test]
    fn ast_md_tag_link_to_hashtag() {
        let result = body_to_ast_with("[#日記](../日記.md)", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "hashtag");
        assert_eq!(node["value"], "日記");
        assert!(node.get("url").is_none());
        assert!(node.get("children").is_none());
    }

    #[test]
    fn ast_md_tag_link_ascii() {
        let result = body_to_ast_with("[#mytag](../mytag.md)", &BodyOptions { resolve_links: true });
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "hashtag");
        assert_eq!(node["value"], "mytag");
    }

    // resolve_links=false: #tag in text stays as text node
    #[test]
    fn ast_no_resolve_hashtag_stays_text() {
        let result = body_to_ast("#tag");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert!(children.iter().all(|c| c["type"] != "hashtag"));
    }

    // resolve_links=false: [#tag](../tag.md) stays as link node
    #[test]
    fn ast_no_resolve_tag_link_stays_link() {
        let result = body_to_ast("[#tag](../tag.md)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // resolve_links=false: [text](../page.md) stays as link node
    #[test]
    fn ast_no_resolve_md_link_stays_link() {
        let result = body_to_ast("[text](../page.md)");
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

    // SSoT: Whitespace-only line preprocessing — blank line becomes <br />
    #[test]
    fn ast_blank_line_becomes_br() {
        // A blank line within a body is converted to <br />, merging into one paragraph
        let result = body_to_ast("line1\n\nline2");
        let v: Value = serde_json::from_str(&result).unwrap();
        // After preprocessing: "line1\n<br />\nline2" — single paragraph
        assert_eq!(v["children"].as_array().unwrap().len(), 1);
        let children = v["children"][0]["children"].as_array().unwrap();
        assert!(
            children.iter().any(|c| c["type"] == "html_inline" && c["value"] == "<br />"),
            "expected html_inline <br /> node"
        );
    }

    // SSoT: Whitespace-only line preprocessing — spaces-only line becomes <br />
    #[test]
    fn ast_spaces_only_line_becomes_br() {
        let result = body_to_ast("line1\n   \nline2");
        let v: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["children"].as_array().unwrap().len(), 1);
        let children = v["children"][0]["children"].as_array().unwrap();
        assert!(
            children.iter().any(|c| c["type"] == "html_inline" && c["value"] == "<br />"),
            "expected html_inline <br /> node"
        );
    }

    // SSoT: Whitespace-only line preprocessing — full-width space line becomes <br />
    #[test]
    fn ast_fullwidth_space_line_becomes_br() {
        let result = body_to_ast("line1\n\u{3000}\nline2");
        let v: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["children"].as_array().unwrap().len(), 1);
        let children = v["children"][0]["children"].as_array().unwrap();
        assert!(
            children.iter().any(|c| c["type"] == "html_inline" && c["value"] == "<br />"),
            "expected html_inline <br /> node"
        );
    }

    // SSoT: Whitespace-only line preprocessing — blank lines inside code fences are preserved
    #[test]
    fn ast_blank_line_in_code_fence_preserved() {
        let result = body_to_ast("```\ncode\n\nmore code\n```");
        let v: Value = serde_json::from_str(&result).unwrap();
        let cb = &v["children"][0];
        assert_eq!(cb["type"], "code_block");
        let literal = cb["literal"].as_str().unwrap();
        assert!(literal.contains("\n\n"), "blank line inside fence must be preserved");
    }
}
