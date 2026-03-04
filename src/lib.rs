use comrak::nodes::NodeValue;
use comrak::{markdown_to_html as comrak_markdown_to_html, parse_document, Arena, Options};
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
    options.extension.multiline_block_quotes = true;
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

pub fn markdown_to_html(input: &str) -> String {
    comrak_markdown_to_html(input, &options())
}

pub fn markdown_to_ast(input: &str) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, input, &options());
    let ast = node_to_ast(root);
    serde_json::to_string(&ast).unwrap()
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
            "fenced": cb.fenced,
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
            "setext": h.setext,
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
        NodeValue::Image(link) => ("image".into(), json!({
            "url": link.url,
            "title": link.title,
        })),
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
            "dollar_math": m.dollar_math,
            "display_math": m.display_math,
            "literal": m.literal,
        })),
        NodeValue::MultilineBlockQuote(_) => ("multiline_block_quote".into(), Value::Null),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_paragraph() {
        let result = markdown_to_html("Hello, world!");
        assert_eq!(result, "<p>Hello, world!</p>\n");
    }

    #[test]
    fn heading() {
        let result = markdown_to_html("# Title");
        assert_eq!(result, "<h1>Title</h1>\n");
    }

    #[test]
    fn strikethrough() {
        let result = markdown_to_html("~~deleted~~");
        assert_eq!(result, "<p><del>deleted</del></p>\n");
    }

    #[test]
    fn tasklist() {
        let result = markdown_to_html("- [x] done\n- [ ] todo");
        assert!(result.contains("checked"));
        assert!(result.contains("type=\"checkbox\""));
    }

    #[test]
    fn hardbreaks() {
        let result = markdown_to_html("line1\nline2");
        assert!(result.contains("<br"));
    }

    #[test]
    fn ast_basic() {
        let result = markdown_to_ast("Hello");
        let v: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["type"], "document");
        assert_eq!(v["children"][0]["type"], "paragraph");
        assert_eq!(v["children"][0]["children"][0]["type"], "text");
        assert_eq!(v["children"][0]["children"][0]["value"], "Hello");
    }

    #[test]
    fn ast_heading() {
        let result = markdown_to_ast("## Sub");
        let v: Value = serde_json::from_str(&result).unwrap();
        let heading = &v["children"][0];
        assert_eq!(heading["type"], "heading");
        assert_eq!(heading["level"], 2);
        assert_eq!(heading["children"][0]["value"], "Sub");
    }

    #[test]
    fn ast_link() {
        let result = markdown_to_ast("[text](https://example.com)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let link = &v["children"][0]["children"][0];
        assert_eq!(link["type"], "link");
        assert_eq!(link["url"], "https://example.com");
        assert_eq!(link["children"][0]["value"], "text");
    }

    #[test]
    fn ast_code_block() {
        let result = markdown_to_ast("```rust\nfn main() {}\n```");
        let v: Value = serde_json::from_str(&result).unwrap();
        let cb = &v["children"][0];
        assert_eq!(cb["type"], "code_block");
        assert_eq!(cb["info"], "rust");
        assert_eq!(cb["literal"], "fn main() {}\n");
    }

    #[test]
    fn wikilink_html_basic() {
        let result = markdown_to_html("[[page]]");
        assert_eq!(result, "<p><a href=\"page\" data-wikilink=\"true\">page</a></p>\n");
    }

    #[test]
    fn wikilink_html_with_label() {
        let result = markdown_to_html("[[url|link label]]");
        assert_eq!(result, "<p><a href=\"url\" data-wikilink=\"true\">link label</a></p>\n");
    }

    #[test]
    fn wikilink_html_inline() {
        let result = markdown_to_html("See [[page]] for details.");
        assert!(result.contains("<a href=\"page\" data-wikilink=\"true\">page</a>"));
        assert!(result.contains("See "));
        assert!(result.contains(" for details."));
    }

    #[test]
    fn wikilink_ast_basic() {
        let result = markdown_to_ast("[[page]]");
        let v: Value = serde_json::from_str(&result).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "page");
        assert_eq!(wl["children"][0]["type"], "text");
        assert_eq!(wl["children"][0]["value"], "page");
    }

    #[test]
    fn wikilink_ast_with_label() {
        let result = markdown_to_ast("[[url|link label]]");
        let v: Value = serde_json::from_str(&result).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "url");
        assert_eq!(wl["children"][0]["value"], "link label");
    }

    #[test]
    fn wikilink_multiple() {
        let result = markdown_to_html("[[page1]] and [[page2]]");
        assert!(result.contains("<a href=\"page1\" data-wikilink=\"true\">page1</a>"));
        assert!(result.contains("<a href=\"page2\" data-wikilink=\"true\">page2</a>"));
    }
}
