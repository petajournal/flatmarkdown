//! Turns relative Markdown links into wikilink and hashtag nodes.

use serde_json::Value;

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
        .is_some_and(|s| s == expected)
}

/// Walks the AST and converts `link` nodes whose URL starts with one of `prefixes`
/// into `wikilink` or `hashtag` nodes. The matching prefix and `.md` suffix are stripped.
/// A link of the form `[#tagname](<prefix>tagname.md)` becomes a `hashtag` node;
/// all other matching links become `wikilink` nodes.
pub(crate) fn convert_markdown_links_to_wikilinks(node: &mut Value, prefixes: &[String]) {
    if node.get("type").and_then(|t| t.as_str()) == Some("link")
        && let Some(url) = node.get("url").and_then(|u| u.as_str())
        && let Some(wikilink_url) = markdown_link_to_wikilink_url(url, prefixes)
    {
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
    if let Some(Value::Array(children)) = node.get_mut("children") {
        for child in children.iter_mut() {
            convert_markdown_links_to_wikilinks(child, prefixes);
        }
    }
}
