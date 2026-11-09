//! Renders the AST of an item body to HTML.

use crate::body::{BodyOptions, body_to_node};
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
    nodes.iter().map(node_to_html).collect()
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
            _ => node_to_html(child),
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

fn table_to_html(rows: &[Node]) -> String {
    let mut thead = String::new();
    let mut tbody = String::new();
    for row in rows {
        let is_header = matches!(row, Node::TableRow { header: true, .. });
        let tag = if is_header { "th" } else { "td" };
        let cells_html: String = row
            .children()
            .iter()
            .map(|cell| format!("<{tag}>{}</{tag}>", render_children(cell)))
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

fn footnote_definition_to_html(name: &str, children: &[Node]) -> String {
    let name = escape_html(name);
    let label = format!("<a href=\"#fnref-{name}\" class=\"footnote-label\">[{name}]:</a>");
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

fn node_to_html(node: &Node) -> String {
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

        // Page metadata is not part of the rendered content.
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

        Node::Table { children, .. } => table_to_html(children),

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

        Node::Subtext { children } => format!("<sub>{}</sub>", render_all(children)),

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

        Node::Embed { url, .. } => format!(
            "<img src=\"{}\" alt=\"{}\" />",
            escape_html(url),
            escape_html(&plain_text(node))
        ),

        Node::Wikilink { url, children } => format!(
            "<a href=\"{}\" data-wikilink=\"true\">{}</a>",
            escape_html(url),
            render_all(children)
        ),

        Node::Hashtag { value } => format!(
            "<a href=\"{}\" class=\"hashtag\">#{value}</a>",
            escape_html(value)
        ),

        // The label is shown instead of comrak's index: a body is parsed alone, so the
        // index would restart at 1 in every item.
        Node::FootnoteReference { name, .. } => {
            let name = escape_html(name);
            format!(
                "<sup id=\"fnref-{name}\" class=\"footnote-ref\"><a href=\"#fn-{name}\">[{name}]</a></sup>"
            )
        }

        Node::Shortcode { emoji, .. } => emoji.clone(),

        Node::Math { literal } => format!("<span class=\"math\">{}</span>", escape_html(literal)),

        Node::HtmlInline { value } => value.clone(),

        Node::Unknown { children, .. } => render_all(children),
    }
}

/// Renders a flatmarkdown item body as an HTML string using the given options.
pub fn body_to_html_with(input: &str, opts: &BodyOptions) -> String {
    node_to_html(&body_to_node(input, opts))
}

pub fn body_to_html(input: &str) -> String {
    body_to_html_with(input, &BodyOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{BodyOptions, LinkResolution};

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
        assert_eq!(
            result,
            "<blockquote>\n<p>q1</p>\n<p>q2</p>\n</blockquote>\n"
        );
    }

    #[test]
    fn wikilink_html_basic() {
        let result = body_to_html("[[page]]");
        assert_eq!(
            result,
            "<p><a href=\"page\" data-wikilink=\"true\">page</a></p>\n"
        );
    }

    #[test]
    fn wikilink_html_with_label() {
        let result = body_to_html("[[url|link label]]");
        assert_eq!(
            result,
            "<p><a href=\"url\" data-wikilink=\"true\">link label</a></p>\n"
        );
    }

    #[test]
    fn wikilink_html_inline() {
        let result = body_to_html("See [[page]] for details.");
        assert!(result.contains("<a href=\"page\" data-wikilink=\"true\">page</a>"));
        assert!(result.contains("See "));
        assert!(result.contains(" for details."));
    }

    #[test]
    fn wikilink_multiple() {
        let result = body_to_html("[[page1]] and [[page2]]");
        assert!(result.contains("<a href=\"page1\" data-wikilink=\"true\">page1</a>"));
        assert!(result.contains("<a href=\"page2\" data-wikilink=\"true\">page2</a>"));
    }

    // コードフェンスの言語名は、class属性の中でエスケープされる。
    #[test]
    fn html_code_fence_info_is_escaped_in_the_class_attribute() {
        let result = body_to_html("```a\"><b>\ncode\n```");
        assert_eq!(
            result,
            "<pre><code class=\"language-a&quot;&gt;&lt;b&gt;\">code\n</code></pre>\n"
        );
    }

    // アラートの題名は、文字としてエスケープされる。
    #[test]
    fn html_alert_title_is_escaped_as_text() {
        let result = body_to_html("> [!NOTE] a < b & c\n> body");
        assert_eq!(
            result,
            "<blockquote class=\"alert alert-note\">\n<p class=\"alert-title\">a &lt; b &amp; c</p>\n<p>body</p>\n</blockquote>\n"
        );
    }

    // 画像のaltには、子の文字だけが入る。子のタグは入らない。
    #[test]
    fn html_image_alt_holds_only_the_text_of_its_children() {
        let result = body_to_html("![a **b** [c](d) <e>](u.png)");
        assert_eq!(
            result,
            "<p><img src=\"u.png\" alt=\"a b c &lt;e&gt;\" /></p>\n"
        );
    }

    // 画像のaltには、コードの文字とハッシュタグも入る。改行は空白になる。
    #[test]
    fn html_image_alt_holds_code_and_hashtags_and_turns_a_line_break_into_a_space() {
        let opts = BodyOptions {
            link_resolution: LinkResolution::On {
                prefixes: vec!["./".into()],
            },
        };
        let result = body_to_html_with("![a `b`\nc #t](u.png)", &opts);
        assert_eq!(result, "<p><img src=\"u.png\" alt=\"a b c #t\" /></p>\n");
    }

    // 本文の生のHTMLは、そのまま出る。自分のノートを描くためのものなので、取り除かない。
    #[test]
    fn html_raw_html_in_the_body_is_written_as_it_is() {
        assert_eq!(
            body_to_html("<script>alert(1)</script>"),
            "<script>alert(1)</script>"
        );
        assert_eq!(
            body_to_html("a <img src=x onerror=alert(1)> b"),
            "<p>a <img src=x onerror=alert(1)> b</p>\n"
        );
    }

    // リンクのURLは調べない。javascript:のリンクも、リンクのまま出る。
    #[test]
    fn html_link_url_is_not_checked() {
        assert_eq!(
            body_to_html("[a](javascript:alert(1))"),
            "<p><a href=\"javascript:alert(1)\">a</a></p>\n"
        );
    }
}
