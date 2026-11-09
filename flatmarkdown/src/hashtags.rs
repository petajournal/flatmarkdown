//! Finds `#tag` in text nodes, and restores the escaped `\#`.

use flatmarkdown_ast::Node;

/// Placeholder for escaped `\#` – Unicode Private Use Area character
/// unlikely to appear in normal text.
pub(crate) const ESCAPED_HASH_PLACEHOLDER: char = '\u{FDD0}';

/// Returns true if `c` is a valid hashtag character:
/// Unicode alphanumeric, `_`, `-`, or `/`.
fn is_hashtag_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '/'
}

/// Splits a text string into a sequence of `text` and `hashtag` nodes.
/// A hashtag starts with `#` at the beginning of the string or after a half-width space (U+0020),
/// and ends when a non-hashtag character is encountered.
/// `/` is not allowed as the first or last character of the tag name.
fn split_hashtags(text: &str) -> Vec<Node> {
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
                    result.push(Node::Text {
                        value: chars[buf_start..i].iter().collect(),
                    });
                }
                result.push(Node::Hashtag {
                    value: chars[tag_start..actual_end].iter().collect(),
                });
                i = actual_end;
                buf_start = i;
                continue;
            }
        }
        i += 1;
    }

    // Flush remaining text
    if buf_start < chars.len() {
        result.push(Node::Text {
            value: chars[buf_start..].iter().collect(),
        });
    }

    result
}

/// Puts a `#` where the placeholder is. For text in which Markdown reads backslash escapes:
/// there `\#` means a plain `#`.
fn restore_as_hash(text: &mut String) {
    if text.contains(ESCAPED_HASH_PLACEHOLDER) {
        *text = text.replace(ESCAPED_HASH_PLACEHOLDER, "#");
    }
}

/// Puts `\#` back where the placeholder is. For text that Markdown keeps raw (code, math, and
/// raw HTML): there a backslash is an ordinary character, so the text stays as it was written.
fn restore_as_written(text: &mut String) {
    if text.contains(ESCAPED_HASH_PLACEHOLDER) {
        *text = text.replace(ESCAPED_HASH_PLACEHOLDER, "\\#");
    }
}

/// Restores the escaped-hash placeholder in every text that a node holds, and in its descendants.
///
/// The `match` names every kind of node on purpose. A new kind of node does not compile until
/// someone decides how its text is restored, so a placeholder cannot be left in the output.
pub(crate) fn restore_escaped_hashes(node: &mut Node) {
    match node {
        Node::Text { value }
        | Node::FrontMatter { value }
        | Node::EscapedTag { value }
        | Node::Hashtag { value } => restore_as_hash(value),
        Node::FootnoteDefinition { name, .. } | Node::FootnoteReference { name, .. } => {
            restore_as_hash(name)
        }
        Node::Wikilink { url, .. } => restore_as_hash(url),
        Node::Link { url, title, .. } => {
            restore_as_hash(url);
            restore_as_hash(title);
        }
        Node::Embed {
            url, title, props, ..
        } => {
            restore_as_hash(url);
            restore_as_hash(title);
            *props = std::mem::take(props)
                .into_iter()
                .map(|(mut key, mut value)| {
                    restore_as_hash(&mut key);
                    if let serde_json::Value::String(text) = &mut value {
                        restore_as_hash(text);
                    }
                    (key, value)
                })
                .collect();
        }
        Node::Alert { title, .. } => {
            if let Some(title) = title {
                restore_as_hash(title);
            }
        }
        Node::CodeBlock { info, literal, .. } => {
            restore_as_hash(info);
            restore_as_written(literal);
        }
        Node::Code { literal } | Node::Math { literal } | Node::HtmlBlock { literal, .. } => {
            restore_as_written(literal)
        }
        Node::HtmlInline { value } | Node::Raw { value } => restore_as_written(value),
        // These hold no text of the user: only children, numbers, flags, or fixed words.
        Node::Document { .. }
        | Node::Paragraph { .. }
        | Node::Heading { .. }
        | Node::BlockQuote { .. }
        | Node::MultilineBlockQuote { .. }
        | Node::List { .. }
        | Node::Item { .. }
        | Node::TaskItem { .. }
        | Node::Table { .. }
        | Node::TableRow { .. }
        | Node::TableCell { .. }
        | Node::ThematicBreak
        | Node::Subtext { .. }
        | Node::Softbreak
        | Node::Linebreak
        | Node::Emph { .. }
        | Node::Strong { .. }
        | Node::Strikethrough { .. }
        | Node::Underline { .. }
        | Node::Highlight { .. }
        | Node::Superscript { .. }
        | Node::Subscript { .. }
        | Node::SpoileredText { .. }
        | Node::Shortcode { .. }
        | Node::Escaped
        | Node::Unknown { .. } => {}
    }
    if let Some(children) = node.children_mut() {
        for child in children.iter_mut() {
            restore_escaped_hashes(child);
        }
    }
}

/// Post-processes the AST to extract hashtags from text nodes.
pub(crate) fn process_hashtags(node: &mut Node) {
    let Some(children) = node.children_mut() else {
        return;
    };
    // Recurse first
    for child in children.iter_mut() {
        process_hashtags(child);
    }

    // Split text nodes containing hashtags
    let mut new_children: Vec<Node> = Vec::with_capacity(children.len());
    for child in children.drain(..) {
        if let Node::Text { value } = &child {
            let parts = split_hashtags(value);
            if parts.len() > 1 || matches!(parts.first(), Some(Node::Hashtag { .. })) {
                new_children.extend(parts);
                continue;
            }
        }
        new_children.push(child);
    }
    *children = new_children;
}
