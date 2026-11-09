//! Turns the tree that comrak parses into the typed AST.

use comrak::Options;
use comrak::nodes::{ListDelimType, ListType, NodeValue, TableAlignment};
use flatmarkdown_ast::Node;
use serde_json::Value;
use std::collections::BTreeMap;

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

/// Turns a comrak node and its descendants into a [`Node`].
pub(crate) fn node_to_ast<'a>(
    node: &'a comrak::arena_tree::Node<'a, std::cell::RefCell<comrak::nodes::Ast>>,
) -> Node {
    let children: Vec<Node> = node.children().map(node_to_ast).collect();
    to_node(&node.data.borrow().value, children)
}

/// Splits the URL of an image into the URL without its fragment and the properties that
/// the fragment holds (`#key=value&flag`). A flag without `=` is `true`.
fn parse_embed_url(url: &str) -> (String, BTreeMap<String, Value>) {
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
        None => (url.to_string(), BTreeMap::new()),
    }
}

fn list_type(list_type: ListType) -> String {
    match list_type {
        ListType::Bullet => "bullet",
        ListType::Ordered => "ordered",
    }
    .to_string()
}

/// A comrak node that SPEC_AST.md does not define. The parser turns none of the extensions
/// on that make these nodes, so this is not reached with the options of this crate.
fn unknown(node_type: &str, children: Vec<Node>) -> Node {
    Node::Unknown {
        node_type: node_type.to_string(),
        children,
    }
}

/// Builds the [`Node`] of one comrak node. `children` are its children, already built.
/// A leaf node of comrak has no children, so the leaf arms do not use them.
fn to_node(value: &NodeValue, children: Vec<Node>) -> Node {
    match value {
        NodeValue::Document => Node::Document { children },
        NodeValue::FrontMatter(s) => Node::FrontMatter { value: s.clone() },
        NodeValue::BlockQuote => Node::BlockQuote { children },
        NodeValue::List(nl) => Node::List {
            list_type: list_type(nl.list_type),
            start: nl.start as u32,
            tight: nl.tight,
            delimiter: match nl.delimiter {
                ListDelimType::Period => "period",
                ListDelimType::Paren => "paren",
            }
            .to_string(),
            children,
        },
        NodeValue::Item(nl) => Node::Item {
            list_type: list_type(nl.list_type),
            start: nl.start as u32,
            tight: nl.tight,
            children,
        },
        NodeValue::CodeBlock(cb) => Node::CodeBlock {
            info: cb.info.clone(),
            literal: cb.literal.clone(),
        },
        NodeValue::HtmlBlock(hb) => Node::HtmlBlock {
            block_type: hb.block_type,
            literal: hb.literal.clone(),
        },
        NodeValue::Paragraph => Node::Paragraph { children },
        NodeValue::Heading(h) => Node::Heading {
            level: h.level,
            children,
        },
        NodeValue::ThematicBreak => Node::ThematicBreak,
        NodeValue::FootnoteDefinition(fd) => Node::FootnoteDefinition {
            name: fd.name.clone(),
            children,
        },
        NodeValue::Table(t) => Node::Table {
            alignments: t
                .alignments
                .iter()
                .map(|a| {
                    match a {
                        TableAlignment::None => "none",
                        TableAlignment::Left => "left",
                        TableAlignment::Center => "center",
                        TableAlignment::Right => "right",
                    }
                    .to_string()
                })
                .collect(),
            num_columns: t.num_columns,
            num_rows: t.num_rows,
            children,
        },
        NodeValue::TableRow(header) => Node::TableRow {
            header: *header,
            children,
        },
        NodeValue::TableCell => Node::TableCell { children },
        NodeValue::Text(s) => Node::Text {
            value: s.to_string(),
        },
        NodeValue::TaskItem(ti) => Node::TaskItem {
            symbol: ti.symbol.map(|c| c.to_string()),
            children,
        },
        NodeValue::SoftBreak => Node::Softbreak,
        NodeValue::LineBreak => Node::Linebreak,
        NodeValue::Code(c) => Node::Code {
            literal: c.literal.clone(),
        },
        NodeValue::HtmlInline(s) => Node::HtmlInline { value: s.clone() },
        NodeValue::Raw(s) => Node::Raw { value: s.clone() },
        NodeValue::Emph => Node::Emph { children },
        NodeValue::Strong => Node::Strong { children },
        NodeValue::Strikethrough => Node::Strikethrough { children },
        NodeValue::Highlight => Node::Highlight { children },
        NodeValue::Superscript => Node::Superscript { children },
        NodeValue::Link(link) => Node::Link {
            url: link.url.clone(),
            title: link.title.clone(),
            children,
        },
        NodeValue::Image(link) => {
            let (url, props) = parse_embed_url(link.url.as_str());
            Node::Embed {
                url,
                title: link.title.clone(),
                props,
                children,
            }
        }
        NodeValue::FootnoteReference(fr) => Node::FootnoteReference {
            name: fr.name.clone(),
            ref_num: fr.ref_num,
            ix: fr.ix,
        },
        NodeValue::ShortCode(sc) => Node::Shortcode {
            code: sc.code.clone(),
            emoji: sc.emoji.clone(),
        },
        NodeValue::Math(m) => Node::Math {
            literal: m.literal.clone(),
        },
        NodeValue::Escaped => Node::Escaped,
        NodeValue::WikiLink(wl) => Node::Wikilink {
            url: wl.url.clone(),
            children,
        },
        NodeValue::Underline => Node::Underline { children },
        NodeValue::Subscript => Node::Subscript { children },
        NodeValue::SpoileredText => Node::SpoileredText { children },
        NodeValue::EscapedTag(s) => Node::EscapedTag {
            value: s.to_string(),
        },
        NodeValue::Alert(a) => Node::Alert {
            alert_type: format!("{:?}", a.alert_type).to_lowercase(),
            title: a.title.clone(),
            children,
        },
        NodeValue::Subtext => Node::Subtext { children },
        NodeValue::MultilineBlockQuote(_) => Node::MultilineBlockQuote { children },
        NodeValue::DescriptionList => unknown("description_list", children),
        NodeValue::DescriptionItem(_) => unknown("description_item", children),
        NodeValue::DescriptionTerm => unknown("description_term", children),
        NodeValue::DescriptionDetails => unknown("description_details", children),
        NodeValue::HeexBlock(_) => unknown("heex_block", children),
        NodeValue::HeexInline(_) => unknown("heex_inline", children),
        NodeValue::Insert => unknown("insert", children),
    }
}
