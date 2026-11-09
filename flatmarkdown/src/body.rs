//! Parses the body of one item into the AST.

use crate::ast::{node_to_ast, options};
use crate::footnotes::{complete_footnotes, strip_footnote_placeholders};
use crate::hashtags::{ESCAPED_HASH_PLACEHOLDER, process_hashtags, restore_escaped_hashes};
use crate::links::convert_markdown_links_to_wikilinks;
use comrak::{Arena, parse_document};
use flatmarkdown_ast::Node;

/// Controls whether and how relative Markdown links are resolved.
#[derive(Default)]
pub enum LinkResolution {
    /// No conversion — links remain as `link` nodes, `#tag` text remains plain.
    #[default]
    Off,
    /// Convert markdown links to `wikilink` / `hashtag` nodes when the URL starts with
    /// one of the given prefixes. The matching prefix and `.md` suffix are stripped
    /// to form the wikilink URL.
    On { prefixes: Vec<String> },
}

/// Options for parsing a flatmarkdown item body.
#[derive(Default)]
pub struct BodyOptions {
    pub link_resolution: LinkResolution,
}

/// Parses a flatmarkdown item body into the typed AST using the given options.
pub(crate) fn body_to_node(input: &str, opts: &BodyOptions) -> Node {
    // `\#` is replaced before comrak sees it so that the hashtag pass cannot
    // mistake the escaped `#` for a tag start.
    let preprocessed = input.replace("\\#", &ESCAPED_HASH_PLACEHOLDER.to_string());
    let arena = Arena::new();
    let root = parse_document(&arena, &preprocessed, &options(false));
    let mut ast = node_to_ast(root);
    if let Some((completed, placeholder_defs)) = complete_footnotes(&preprocessed, &ast) {
        let arena = Arena::new();
        let root = parse_document(&arena, &completed, &options(false));
        ast = node_to_ast(root);
        strip_footnote_placeholders(&mut ast, &placeholder_defs);
    }
    if let LinkResolution::On { prefixes } = &opts.link_resolution {
        convert_markdown_links_to_wikilinks(&mut ast, prefixes);
        process_hashtags(&mut ast);
    }
    restore_escaped_hashes(&mut ast);
    ast
}

/// Parses a flatmarkdown item body into a JSON AST string using the given options.
pub fn body_to_ast_with(input: &str, opts: &BodyOptions) -> String {
    serde_json::to_string(&body_to_node(input, opts)).unwrap()
}

pub fn body_to_ast(input: &str) -> String {
    body_to_ast_with(input, &BodyOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    /// Link resolution on. Hashtags and wikilinks are only made with it, so a test that says
    /// "this is not a hashtag" or "this stays a link" proves nothing without it.
    fn with_link_resolution() -> BodyOptions {
        BodyOptions {
            link_resolution: LinkResolution::On {
                prefixes: vec!["../".into(), "./".into()],
            },
        }
    }

    #[test]
    fn ast_multiline_block_quote() {
        let result = body_to_ast(">>>\nq1\n\nq2\n>>>");
        let v: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["children"][0]["type"], "multiline_block_quote");
        assert_eq!(v["children"][0]["children"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn wikilink_ast_basic() {
        let result = body_to_ast("[[page]]");
        let v: Value = serde_json::from_str(&result).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "page");
        assert_eq!(wl["children"][0]["type"], "text");
        assert_eq!(wl["children"][0]["value"], "page");
    }

    #[test]
    fn wikilink_ast_with_label() {
        let result = body_to_ast("[[url|link label]]");
        let v: Value = serde_json::from_str(&result).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "url");
        assert_eq!(wl["children"][0]["value"], "link label");
    }

    #[test]
    fn ast_hashtag_basic() {
        let result = body_to_ast_with(
            "#tag",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "tag");
    }

    #[test]
    fn ast_hashtag_japanese() {
        let result = body_to_ast_with(
            "#日記",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "日記");
    }

    #[test]
    fn ast_hashtag_with_special_chars() {
        let result = body_to_ast_with(
            "#my_tag-name/sub",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "my_tag-name/sub");
    }

    #[test]
    fn ast_hashtag_in_text() {
        let result = body_to_ast_with(
            "hello #tag world",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "hello ");
        assert_eq!(children[1]["type"], "hashtag");
        assert_eq!(children[1]["value"], "tag");
        assert_eq!(children[2]["type"], "text");
        assert_eq!(children[2]["value"], " world");
    }

    #[test]
    fn ast_hashtag_multiple() {
        let result = body_to_ast_with(
            "#tag1 #tag2",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "tag1");
        assert_eq!(children[1]["type"], "text");
        assert_eq!(children[1]["value"], " ");
        assert_eq!(children[2]["type"], "hashtag");
        assert_eq!(children[2]["value"], "tag2");
    }

    #[test]
    fn ast_hashtag_not_in_middle_of_word() {
        let result = body_to_ast_with("foo#bar", &with_link_resolution());
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "foo#bar");
    }

    #[test]
    fn ast_hashtag_terminated_by_punctuation() {
        let result = body_to_ast_with(
            "#タグ。本文",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "タグ");
        assert_eq!(children[1]["type"], "text");
        assert_eq!(children[1]["value"], "。本文");
    }

    #[test]
    fn ast_hashtag_leading_slash_not_hashtag() {
        // "#/tag" must not be recognized as a hashtag (leading '/' is invalid)
        let result = body_to_ast_with("#/tag", &with_link_resolution());
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#/tag");
    }

    #[test]
    fn ast_hashtag_trailing_slash_trimmed() {
        // "#tag/" must be recognized as hashtag "tag" (trailing '/' is trimmed)
        let result = body_to_ast_with(
            "#tag/",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "tag");
        assert_eq!(children[1]["type"], "text");
        assert_eq!(children[1]["value"], "/");
    }

    #[test]
    fn ast_hashtag_leading_and_trailing_slash_not_hashtag() {
        // "#/tag/" must not be recognized as a hashtag (leading '/' is invalid)
        let result = body_to_ast_with("#/tag/", &with_link_resolution());
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#/tag/");
    }

    #[test]
    fn ast_hash_only_not_hashtag() {
        let result = body_to_ast("# ");
        let v: Value = serde_json::from_str(&result).unwrap();
        // "# " is a heading, not a hashtag
        assert_eq!(v["children"][0]["type"], "heading");
    }

    #[test]
    fn ast_escaped_hashtag_basic() {
        let result = body_to_ast_with(r"\#tag", &with_link_resolution());
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#tag");
    }

    #[test]
    fn ast_escaped_hashtag_with_real_hashtag() {
        let result = body_to_ast_with(
            r"\#notag #real",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#notag ");
        assert_eq!(children[1]["type"], "hashtag");
        assert_eq!(children[1]["value"], "real");
    }

    #[test]
    fn ast_escaped_hashtag_multiple() {
        let result = body_to_ast_with(r"\#a \#b", &with_link_resolution());
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#a #b");
    }

    #[test]
    fn ast_embed_no_props() {
        let result = body_to_ast("![alt text](img.png)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "img.png");
        assert_eq!(embed["props"], Value::Null);
        assert_eq!(embed["children"][0]["value"], "alt text");
    }

    #[test]
    fn ast_embed_with_kv_props() {
        let result = body_to_ast("![alt](img.png#w=300&h=240)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "img.png");
        assert_eq!(embed["props"]["w"], "300");
        assert_eq!(embed["props"]["h"], "240");
    }

    #[test]
    fn ast_embed_with_boolean_flag() {
        let result = body_to_ast("![alt](video.mp4#w=640&autoplay)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "video.mp4");
        assert_eq!(embed["props"]["w"], "640");
        assert_eq!(embed["props"]["autoplay"], true);
    }

    #[test]
    fn ast_embed_url_with_query_and_fragment() {
        let result = body_to_ast("![alt](https://youtube.com/watch?v=xxx#w=640&start=30)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "https://youtube.com/watch?v=xxx");
        assert_eq!(embed["props"]["w"], "640");
        assert_eq!(embed["props"]["start"], "30");
    }

    // SSoT: Extended syntax - highlight (==text==)
    #[test]
    fn ast_highlight() {
        let result = body_to_ast("==highlighted==");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "highlight");
        assert_eq!(node["children"][0]["value"], "highlighted");
    }

    // SSoT: Extended syntax - superscript (^text^)
    #[test]
    fn ast_superscript() {
        let result = body_to_ast("^super^");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "superscript");
        assert_eq!(node["children"][0]["value"], "super");
    }

    // SSoT: Extended syntax - subscript (~text~)
    #[test]
    fn ast_subscript() {
        let result = body_to_ast("~sub~");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "subscript");
        assert_eq!(node["children"][0]["value"], "sub");
    }

    // SSoT: Extended syntax - spoilered_text (||text||)
    #[test]
    fn ast_spoilered_text() {
        let result = body_to_ast("||spoiler||");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "spoilered_text");
        assert_eq!(node["children"][0]["value"], "spoiler");
    }

    // SSoT: Extended syntax - underline (__text__ is underline, not strong)
    #[test]
    fn ast_underline() {
        let result = body_to_ast("__underlined__");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "underline");
        assert_eq!(node["children"][0]["value"], "underlined");
    }

    // SSoT: Extended syntax - shortcode (:name:)
    #[test]
    fn ast_shortcode() {
        let result = body_to_ast(":rabbit:");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "shortcode");
        assert_eq!(node["code"], "rabbit");
        assert!(node["emoji"].is_string());
    }

    // SSoT: Extended syntax - math ($`formula`$ syntax)
    #[test]
    fn ast_math_inline() {
        let result = body_to_ast("$`x^2`$");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "math");
        assert_eq!(node["literal"], "x^2");
    }

    // SSoT: Extended syntax - alert (> [!NOTE])
    #[test]
    fn ast_alert_note() {
        let result = body_to_ast("> [!NOTE]\n> Content");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0];
        assert_eq!(node["type"], "alert");
        assert_eq!(node["alert_type"], "note");
    }

    // SSoT: Wikilink block link [[page#id]]
    #[test]
    fn ast_wikilink_block_link() {
        let result = body_to_ast("[[page#block-id]]");
        let v: Value = serde_json::from_str(&result).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "page#block-id");
    }

    // SSoT: Wikilink block link with label [[page#id|label]]
    #[test]
    fn ast_wikilink_block_link_with_label() {
        let result = body_to_ast("[[page#block-id|label]]");
        let v: Value = serde_json::from_str(&result).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "page#block-id");
        assert_eq!(wl["children"][0]["value"], "label");
    }

    // SSoT: Markdown relative link → wikilink (no label: link text equals page name)
    #[test]
    fn ast_md_link_to_wikilink_no_label() {
        let result = body_to_ast_with(
            "[pagename](../pagename.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "pagename");
    }

    // SSoT: Markdown relative link → wikilink (with label)
    #[test]
    fn ast_md_link_to_wikilink_with_label() {
        let result = body_to_ast_with(
            "[display label](../pagename.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "display label");
    }

    // SSoT: Markdown relative link with path → wikilink URL preserves full path
    #[test]
    fn ast_md_link_to_wikilink_with_path() {
        let result = body_to_ast_with(
            "[label](../journal/2024-01-01.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "journal/2024-01-01");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Markdown relative link with fragment → wikilink URL includes #id
    #[test]
    fn ast_md_link_to_wikilink_with_fragment() {
        let result = body_to_ast_with(
            "[label](../page.md#block-id)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "page#block-id");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Markdown relative link with ./ prefix → wikilink
    #[test]
    fn ast_md_link_dotslash_to_wikilink() {
        let result = body_to_ast_with(
            "[pagename](./pagename.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "pagename");
    }

    // SSoT: Markdown relative link with ./ prefix and path → wikilink URL preserves path
    #[test]
    fn ast_md_link_dotslash_with_path() {
        let result = body_to_ast_with(
            "[label](./journal/2024-01-01.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "journal/2024-01-01");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Markdown relative link with ./ prefix and fragment → wikilink includes #id
    #[test]
    fn ast_md_link_dotslash_with_fragment() {
        let result = body_to_ast_with(
            "[label](./page.md#block-id)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "page#block-id");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // SSoT: Tag Markdown link with ./ prefix → hashtag node
    #[test]
    fn ast_md_tag_link_dotslash_to_hashtag() {
        let result = body_to_ast_with(
            "[#mytag](./mytag.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "hashtag");
        assert_eq!(node["value"], "mytag");
        assert!(node.get("url").is_none());
        assert!(node.get("children").is_none());
    }

    // SSoT: Tag Markdown link → hashtag node
    #[test]
    fn ast_md_tag_link_to_hashtag() {
        let result = body_to_ast_with(
            "[#日記](../日記.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "hashtag");
        assert_eq!(node["value"], "日記");
        assert!(node.get("url").is_none());
        assert!(node.get("children").is_none());
    }

    // LinkResolution::Off: #tag in text stays as text node
    #[test]
    fn ast_no_resolve_hashtag_stays_text() {
        let result = body_to_ast("#tag");
        let v: Value = serde_json::from_str(&result).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert!(children.iter().all(|c| c["type"] != "hashtag"));
    }

    // LinkResolution::Off: [#tag](../tag.md) stays as link node
    #[test]
    fn ast_no_resolve_tag_link_stays_link() {
        let result = body_to_ast("[#tag](../tag.md)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // LinkResolution::Off: [text](../page.md) stays as link node
    #[test]
    fn ast_no_resolve_md_link_stays_link() {
        let result = body_to_ast("[text](../page.md)");
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // LinkResolution::On with empty prefixes: no links are converted
    #[test]
    fn ast_on_empty_prefixes_md_link_stays_link() {
        let result = body_to_ast_with(
            "[text](../page.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On { prefixes: vec![] },
            },
        );
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // External links must not be converted to wikilinks
    #[test]
    fn ast_external_link_not_converted_to_wikilink() {
        let result = body_to_ast_with("[text](https://example.com)", &with_link_resolution());
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // Relative links not ending in .md must not be converted to wikilinks
    #[test]
    fn ast_non_md_relative_link_not_converted() {
        let result = body_to_ast_with("[text](../page.html)", &with_link_resolution());
        let v: Value = serde_json::from_str(&result).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // SSoT: blank lines are no longer rewritten; a loose list stays one block.
    #[test]
    fn ast_blank_line_between_list_items_keeps_one_list() {
        let result = body_to_ast("- a\n\n- b");
        let v: Value = serde_json::from_str(&result).unwrap();
        let blocks = v["children"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "list");
        assert_eq!(blocks[0]["tight"], false);
    }

    #[test]
    fn ast_blank_line_in_code_fence_preserved() {
        let result = body_to_ast("```\ncode\n\nmore code\n```");
        let v: Value = serde_json::from_str(&result).unwrap();
        let cb = &v["children"][0];
        assert_eq!(cb["type"], "code_block");
        let literal = cb["literal"].as_str().unwrap();
        assert!(
            literal.contains("\n\n"),
            "blank line inside fence must be preserved"
        );
    }
}
