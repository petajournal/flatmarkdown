//! Renders the Flat Markdown AST to HTML.
//!
//! The AST is the [`Node`] of the `flatmarkdown-ast` crate. This crate has no Markdown parser:
//! the `flatmarkdown` crate makes the AST from Markdown, and `flatmarkdown_ast::parse` reads it
//! from JSON.
//!
//! The HTML is made for content that you trust. Raw HTML of the AST is written as it is, and
//! the URL of a link is not checked.

use flatmarkdown_ast::Node;

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The text of a node and its descendants, without any markup: what the `alt` of an image holds.
/// Raw inline HTML counts as text, as in the CommonMark reference renderer.
fn plain_text(node: &Node) -> String {
    match node {
        Node::Text { value } | Node::HtmlInline { value } => value.clone(),
        Node::Code { literal } => literal.clone(),
        Node::Hashtag { value } => format!("#{value}"),
        Node::Softbreak | Node::Linebreak => " ".to_string(),
        _ => node.children().iter().map(plain_text).collect(),
    }
}

fn render_all(nodes: &[Node]) -> String {
    nodes.iter().map(ast_to_html).collect()
}

fn render_children(node: &Node) -> String {
    render_all(node.children())
}

/// Renders the children of a list item in tight or non-tight mode.
/// In tight mode, top-level paragraph children are rendered without `<p>` wrappers.
fn render_item_content(item: &Node, tight: bool) -> String {
    item.children()
        .iter()
        .map(|child| match child {
            Node::Paragraph { children } if tight => render_all(children),
            _ => ast_to_html(child),
        })
        .collect()
}

/// The checkbox of a task item. Any symbol in the brackets counts as checked.
fn checkbox(symbol: &Option<String>) -> &'static str {
    if symbol.is_some() {
        "<input type=\"checkbox\" checked disabled>"
    } else {
        "<input type=\"checkbox\" disabled>"
    }
}

fn list_to_html(list_type: &str, start: u32, tight: bool, items: &[Node]) -> String {
    let items_html: String = items
        .iter()
        .map(|item| {
            let content = render_item_content(item, tight);
            match item {
                Node::TaskItem { symbol, .. } => {
                    format!("<li>{}{content}</li>\n", checkbox(symbol))
                }
                _ => format!("<li>{content}</li>\n"),
            }
        })
        .collect();
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

/// The `align` attribute of a cell, as GFM writes it. A column without an alignment has none.
fn align_attribute(alignment: Option<&String>) -> String {
    match alignment.map(String::as_str) {
        Some(alignment @ ("left" | "center" | "right")) => format!(" align=\"{alignment}\""),
        _ => String::new(),
    }
}

fn table_to_html(alignments: &[String], rows: &[Node]) -> String {
    let mut thead = String::new();
    let mut tbody = String::new();
    for row in rows {
        let is_header = matches!(row, Node::TableRow { header: true, .. });
        let tag = if is_header { "th" } else { "td" };
        let cells_html: String = row
            .children()
            .iter()
            .enumerate()
            .map(|(column, cell)| {
                let align = align_attribute(alignments.get(column));
                format!("<{tag}{align}>{}</{tag}>", render_children(cell))
            })
            .collect();
        let row_html = format!("<tr>{cells_html}</tr>\n");
        if is_header {
            thead.push_str(&row_html);
        } else {
            tbody.push_str(&row_html);
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

/// The text that a reader sees for a footnote. An inline footnote has a name that the parser
/// made (`__inline_1`), so its number is shown instead.
fn footnote_label(name: &str) -> String {
    escape_html(name.strip_prefix("__inline_").unwrap_or(name))
}

fn footnote_definition_to_html(name: &str, children: &[Node]) -> String {
    let shown = footnote_label(name);
    let name = escape_html(name);
    let label = format!("<a href=\"#fnref-{name}\" class=\"footnote-label\">[{shown}]:</a>");
    // The label goes inside the first paragraph so that the body continues on
    // the same line without any CSS. A definition that starts with another
    // block keeps the label in front of it.
    let body = match children.split_first() {
        Some((Node::Paragraph { children: first }, rest)) => {
            format!("<p>{label} {}</p>\n{}", render_all(first), render_all(rest))
        }
        _ => format!("{label}\n{}", render_all(children)),
    };
    format!("<div id=\"fn-{name}\" class=\"footnote\">\n{body}</div>\n")
}

fn alert_to_html(alert_type: &str, title: &Option<String>, children: &[Node]) -> String {
    let title = title.as_deref().unwrap_or(match alert_type {
        "tip" => "Tip",
        "important" => "Important",
        "warning" => "Warning",
        "caution" => "Caution",
        _ => "Note",
    });
    format!(
        "<blockquote class=\"alert alert-{alert_type}\">\n<p class=\"alert-title\">{}</p>\n{}</blockquote>\n",
        escape_html(title),
        render_all(children)
    )
}

/// Renders a node of the AST and its descendants as an HTML string.
///
/// The node is normally the `Node::Document` of an item body, but any node can be given.
pub fn ast_to_html(node: &Node) -> String {
    match node {
        Node::Document { children } => render_all(children),

        Node::Paragraph { children } => format!("<p>{}</p>\n", render_all(children)),

        Node::Heading { level, children } => {
            format!("<h{level}>{}</h{level}>\n", render_all(children))
        }

        Node::CodeBlock { info, literal, .. } => {
            let class = if info.is_empty() || info == "text" {
                String::new()
            } else {
                format!(" class=\"language-{}\"", escape_html(info))
            };
            format!("<pre><code{class}>{}</code></pre>\n", escape_html(literal))
        }

        Node::BlockQuote { children } | Node::MultilineBlockQuote { children } => {
            format!("<blockquote>\n{}</blockquote>\n", render_all(children))
        }

        Node::List {
            list_type,
            start,
            tight,
            children,
            ..
        } => list_to_html(list_type, *start, *tight, children),

        // Fallback — normally rendered inside the list handler
        Node::Item { children, .. } => format!("<li>{}</li>\n", render_all(children)),
        Node::TaskItem { symbol, children } => {
            format!("<li>{}{}</li>\n", checkbox(symbol), render_all(children))
        }

        Node::Table {
            alignments,
            children,
            ..
        } => table_to_html(alignments, children),

        // Fallback — normally rendered inside the table handler
        Node::TableRow { children, .. } | Node::TableCell { children } => render_all(children),

        Node::ThematicBreak => "<hr />\n".to_string(),

        Node::HtmlBlock { literal, .. } => literal.clone(),

        Node::FootnoteDefinition { name, children } => footnote_definition_to_html(name, children),

        Node::Alert {
            alert_type,
            title,
            children,
        } => alert_to_html(alert_type, title, children),

        // The same HTML as comrak writes for `-# text`.
        Node::Subtext { children } => format!("<p><sub>{}</sub></p>\n", render_all(children)),

        Node::Text { value } => escape_html(value),

        Node::Softbreak => "\n".to_string(),
        Node::Linebreak => "<br />\n".to_string(),

        Node::Emph { children } => format!("<em>{}</em>", render_all(children)),
        Node::Strong { children } => format!("<strong>{}</strong>", render_all(children)),
        Node::Strikethrough { children } => format!("<del>{}</del>", render_all(children)),
        Node::Underline { children } => format!("<u>{}</u>", render_all(children)),
        Node::Highlight { children } => format!("<mark>{}</mark>", render_all(children)),
        Node::Superscript { children } => format!("<sup>{}</sup>", render_all(children)),
        Node::Subscript { children } => format!("<sub>{}</sub>", render_all(children)),
        Node::SpoileredText { children } => {
            format!("<span class=\"spoiler\">{}</span>", render_all(children))
        }

        Node::Code { literal } => format!("<code>{}</code>", escape_html(literal)),

        Node::Link {
            url,
            title,
            children,
        } => {
            if title.is_empty() {
                format!(
                    "<a href=\"{}\">{}</a>",
                    escape_html(url),
                    render_all(children)
                )
            } else {
                format!(
                    "<a href=\"{}\" title=\"{}\">{}</a>",
                    escape_html(url),
                    escape_html(title),
                    render_all(children)
                )
            }
        }

        // `props` are not written. How a renderer uses them differs from app to app.
        Node::Embed { url, title, .. } => {
            let title = if title.is_empty() {
                String::new()
            } else {
                format!(" title=\"{}\"", escape_html(title))
            };
            format!(
                "<img src=\"{}\" alt=\"{}\"{title} />",
                escape_html(url),
                escape_html(&plain_text(node))
            )
        }

        Node::Wikilink { url, children } => format!(
            "<a href=\"{}\" data-wikilink=\"true\">{}</a>",
            escape_html(url),
            render_all(children)
        ),

        Node::Hashtag { value } => {
            let name = escape_html(value);
            format!("<a href=\"{name}\" class=\"hashtag\">#{name}</a>")
        }

        // The label is shown instead of comrak's index: a body is parsed alone, so the
        // index would restart at 1 in every item.
        Node::FootnoteReference { name, ref_num, .. } => {
            let shown = footnote_label(name);
            let name = escape_html(name);
            // An `id` must be unique. The second reference to the same footnote carries its
            // number; the first does not, so that the label of the definition links to it.
            let id = match ref_num {
                1 => format!("fnref-{name}"),
                n => format!("fnref-{name}-{n}"),
            };
            format!(
                "<sup id=\"{id}\" class=\"footnote-ref\"><a href=\"#fn-{name}\">[{shown}]</a></sup>"
            )
        }

        Node::Shortcode { emoji, .. } => emoji.clone(),

        Node::Math { literal } => format!("<span class=\"math\">{}</span>", escape_html(literal)),

        Node::HtmlInline { value } => value.clone(),

        Node::Unknown { children, .. } => render_all(children),
    }
}

// Run the code example in the README as a doctest, so that it keeps working
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
