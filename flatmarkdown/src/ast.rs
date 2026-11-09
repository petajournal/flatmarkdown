//! Turns the tree that comrak parses into the JSON AST.

use comrak::Options;
use comrak::nodes::NodeValue;
use serde_json::{Value, json};

/// Builds the comrak options shared by body and page parsing.
/// Front matter is only recognised when parsing a whole page, because a body
/// that happens to start with `---` must stay a thematic break.
pub(crate) fn options(front_matter: bool) -> Options<'static> {
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

pub(crate) fn node_to_ast<'a>(
    node: &'a comrak::arena_tree::Node<'a, std::cell::RefCell<comrak::nodes::Ast>>,
) -> Value {
    let ast = node.data.borrow();
    let (node_type, attrs) = serialize_node_value(&ast.value);

    let children: Vec<Value> = node.children().map(node_to_ast).collect();

    let mut obj = serde_json::Map::new();
    obj.insert("type".into(), Value::String(node_type));
    if !attrs.is_null()
        && let Value::Object(map) = attrs
    {
        for (k, v) in map {
            obj.insert(k, v);
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
                    Some(eq) => (
                        kv[..eq].to_string(),
                        Value::String(kv[eq + 1..].to_string()),
                    ),
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
        NodeValue::List(nl) => (
            "list".into(),
            json!({
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
            }),
        ),
        NodeValue::Item(nl) => (
            "item".into(),
            json!({
                "list_type": match nl.list_type {
                    comrak::nodes::ListType::Bullet => "bullet",
                    comrak::nodes::ListType::Ordered => "ordered",
                },
                "start": nl.start,
                "tight": nl.tight,
            }),
        ),
        NodeValue::DescriptionList => ("description_list".into(), Value::Null),
        NodeValue::DescriptionItem(_) => ("description_item".into(), Value::Null),
        NodeValue::DescriptionTerm => ("description_term".into(), Value::Null),
        NodeValue::DescriptionDetails => ("description_details".into(), Value::Null),
        NodeValue::CodeBlock(cb) => (
            "code_block".into(),
            json!({
                "info": cb.info,
                "literal": cb.literal,
            }),
        ),
        NodeValue::HtmlBlock(hb) => (
            "html_block".into(),
            json!({
                "block_type": hb.block_type,
                "literal": hb.literal,
            }),
        ),
        NodeValue::HeexBlock(_) => ("heex_block".into(), Value::Null),
        NodeValue::Paragraph => ("paragraph".into(), Value::Null),
        NodeValue::Heading(h) => (
            "heading".into(),
            json!({
                "level": h.level,
            }),
        ),
        NodeValue::ThematicBreak => ("thematic_break".into(), Value::Null),
        NodeValue::FootnoteDefinition(fd) => (
            "footnote_definition".into(),
            json!({
                "name": fd.name,
            }),
        ),
        NodeValue::Table(t) => (
            "table".into(),
            json!({
                "alignments": t.alignments.iter().map(|a| match a {
                    comrak::nodes::TableAlignment::None => "none",
                    comrak::nodes::TableAlignment::Left => "left",
                    comrak::nodes::TableAlignment::Center => "center",
                    comrak::nodes::TableAlignment::Right => "right",
                }).collect::<Vec<_>>(),
                "num_columns": t.num_columns,
                "num_rows": t.num_rows,
            }),
        ),
        NodeValue::TableRow(header) => (
            "table_row".into(),
            json!({
                "header": header,
            }),
        ),
        NodeValue::TableCell => ("table_cell".into(), Value::Null),
        NodeValue::Text(s) => ("text".into(), json!({ "value": s.as_ref() })),
        NodeValue::TaskItem(ti) => (
            "task_item".into(),
            json!({
                "symbol": ti.symbol.map(|c| c.to_string()),
            }),
        ),
        NodeValue::SoftBreak => ("softbreak".into(), Value::Null),
        NodeValue::LineBreak => ("linebreak".into(), Value::Null),
        NodeValue::Code(c) => (
            "code".into(),
            json!({
                "literal": c.literal,
            }),
        ),
        NodeValue::HtmlInline(s) => ("html_inline".into(), json!({ "value": s })),
        NodeValue::HeexInline(s) => ("heex_inline".into(), json!({ "value": s })),
        NodeValue::Raw(s) => ("raw".into(), json!({ "value": s })),
        NodeValue::Emph => ("emph".into(), Value::Null),
        NodeValue::Strong => ("strong".into(), Value::Null),
        NodeValue::Strikethrough => ("strikethrough".into(), Value::Null),
        NodeValue::Highlight => ("highlight".into(), Value::Null),
        NodeValue::Superscript => ("superscript".into(), Value::Null),
        NodeValue::Link(link) => (
            "link".into(),
            json!({
                "url": link.url,
                "title": link.title,
            }),
        ),
        NodeValue::Image(link) => {
            let (url, props) = parse_embed_url(link.url.as_str());
            let mut attrs = json!({ "url": url, "title": link.title });
            if !props.is_empty() {
                attrs
                    .as_object_mut()
                    .unwrap()
                    .insert("props".into(), Value::Object(props));
            }
            ("embed".into(), attrs)
        }
        NodeValue::FootnoteReference(fr) => (
            "footnote_reference".into(),
            json!({
                "name": fr.name,
                "ref_num": fr.ref_num,
                "ix": fr.ix,
            }),
        ),
        NodeValue::ShortCode(sc) => (
            "shortcode".into(),
            json!({
                "code": sc.code,
                "emoji": sc.emoji,
            }),
        ),
        NodeValue::Math(m) => (
            "math".into(),
            json!({
                "literal": m.literal,
            }),
        ),
        NodeValue::Escaped => ("escaped".into(), Value::Null),
        NodeValue::WikiLink(wl) => (
            "wikilink".into(),
            json!({
                "url": wl.url,
            }),
        ),
        NodeValue::Underline => ("underline".into(), Value::Null),
        NodeValue::Subscript => ("subscript".into(), Value::Null),
        NodeValue::SpoileredText => ("spoilered_text".into(), Value::Null),
        NodeValue::EscapedTag(s) => ("escaped_tag".into(), json!({ "value": s })),
        NodeValue::Alert(a) => (
            "alert".into(),
            json!({
                "alert_type": format!("{:?}", a.alert_type).to_lowercase(),
                "title": a.title,
            }),
        ),
        NodeValue::Subtext => ("subtext".into(), Value::Null),
        NodeValue::Insert => ("insert".into(), Value::Null),
        NodeValue::MultilineBlockQuote(_) => ("multiline_block_quote".into(), Value::Null),
    }
}
