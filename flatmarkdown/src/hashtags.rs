//! Finds `#tag` in text nodes, and restores the escaped `\#`.

use serde_json::{Value, json};

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
pub(crate) fn restore_escaped_hashes(node: &mut Value) {
    if let Some(val) = node.get_mut("value")
        && let Some(s) = val.as_str()
        && s.contains(ESCAPED_HASH_PLACEHOLDER)
    {
        *val = Value::String(s.replace(ESCAPED_HASH_PLACEHOLDER, "#"));
    }
    if let Some(Value::Array(children)) = node.get_mut("children") {
        for child in children.iter_mut() {
            restore_escaped_hashes(child);
        }
    }
}

/// Post-processes the AST to extract hashtags from text nodes.
pub(crate) fn process_hashtags(node: &mut Value) {
    if let Some(Value::Array(children)) = node.get_mut("children") {
        // Recurse first
        for child in children.iter_mut() {
            process_hashtags(child);
        }

        // Split text nodes containing hashtags
        let mut new_children: Vec<Value> = Vec::with_capacity(children.len());
        for child in children.drain(..) {
            if child.get("type").and_then(|t| t.as_str()) == Some("text")
                && let Some(text) = child.get("value").and_then(|v| v.as_str())
            {
                let parts = split_hashtags(text);
                if parts.len() > 1
                    || parts
                        .first()
                        .and_then(|p| p.get("type"))
                        .and_then(|t| t.as_str())
                        == Some("hashtag")
                {
                    new_children.extend(parts);
                    continue;
                }
            }
            new_children.push(child);
        }

        *children = new_children;
    }
}
