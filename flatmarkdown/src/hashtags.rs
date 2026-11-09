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

/// Restores the escaped-hash placeholder back to `#` in every node that has a `value`.
pub(crate) fn restore_escaped_hashes(node: &mut Node) {
    match node {
        Node::Text { value }
        | Node::FrontMatter { value }
        | Node::HtmlInline { value }
        | Node::Raw { value }
        | Node::EscapedTag { value }
        | Node::Hashtag { value }
            if value.contains(ESCAPED_HASH_PLACEHOLDER) =>
        {
            *value = value.replace(ESCAPED_HASH_PLACEHOLDER, "#");
        }
        _ => {}
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
