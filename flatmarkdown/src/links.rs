//! Turns relative Markdown links into wikilink and hashtag nodes.

use crate::hashtags::is_tag_name;
use flatmarkdown_ast::Node;

/// Strips the longest matching prefix from `url` and removes the `.md` suffix, returning the
/// wikilink URL.
/// Returns `None` if no prefix matches or the URL does not end with `.md`.
///
/// Also returns `None` if the file has no name: `.md` alone or after only whitespace, at the
/// top or in a folder. The name of a `.md` file can come from outside, so such a link must
/// not make a wikilink or a hashtag with an empty name.
fn markdown_link_to_wikilink_url(url: &str, prefixes: &[String]) -> Option<String> {
    // The longest prefix that matches is used, so the order of the list does not matter.
    // What is left is then the shortest.
    let without_prefix = prefixes
        .iter()
        .filter_map(|p| url.strip_prefix(p.as_str()))
        .min_by_key(|rest| rest.len())?;
    let (path_part, fragment) = match without_prefix.find('#') {
        Some(pos) => (&without_prefix[..pos], &without_prefix[pos..]),
        None => (without_prefix, ""),
    };
    let page_path = path_part.strip_suffix(".md")?;
    // The file name is what stands after the last `/`. Whitespace alone is no name.
    let file_name = page_path.rsplit('/').next().unwrap_or(page_path);
    if file_name.trim().is_empty() {
        return None;
    }
    // The whitespace around the page path is dropped, as it is for `[[ page ]]`.
    Some(format!("{}{}", page_path.trim(), fragment))
}

/// Returns `true` if the children of a `link` node represent a serialised tag:
/// the only child is a text node whose value is `#` + `wikilink_url`.
///
/// The name must also follow the rules of a tag name. A link is the only way to a hashtag
/// that does not go through those rules, and the name of a `.md` file can come from outside.
fn is_tag_link(children: &[Node], wikilink_url: &str) -> bool {
    match children {
        [Node::Text { value }] => {
            value.strip_prefix('#') == Some(wikilink_url) && is_tag_name(wikilink_url)
        }
        _ => false,
    }
}

/// Walks the AST and converts `link` nodes whose URL starts with one of `prefixes`
/// into `wikilink` or `hashtag` nodes. The matching prefix and `.md` suffix are stripped.
/// A link of the form `[#tagname](<prefix>tagname.md)` becomes a `hashtag` node;
/// all other matching links become `wikilink` nodes.
pub(crate) fn convert_markdown_links_to_wikilinks(node: &mut Node, prefixes: &[String]) {
    if let Node::Link { url, children, .. } = node
        && let Some(wikilink_url) = markdown_link_to_wikilink_url(url, prefixes)
    {
        *node = if is_tag_link(children, &wikilink_url) {
            Node::Hashtag {
                value: wikilink_url,
            }
        } else {
            Node::Wikilink {
                url: wikilink_url,
                children: std::mem::take(children),
            }
        };
        return;
    }
    if let Some(children) = node.children_mut() {
        for child in children.iter_mut() {
            convert_markdown_links_to_wikilinks(child, prefixes);
        }
    }
}
