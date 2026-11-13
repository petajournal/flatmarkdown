//! Parses the body of one item into the AST.

use crate::ast::{node_to_ast, options};
use crate::footnotes::{complete_footnotes, renumber_footnotes, strip_footnote_placeholders};
use crate::hashtags::{mark_escaped_hashes, process_hashtags, restore_escaped_hashes};
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

impl BodyOptions {
    /// The options for pages that are `.md` files linked to each other by relative paths:
    /// a link to `./page.md` or `../page.md` is a wikilink, and `#tag` is a hashtag.
    ///
    /// Every program that parses the same pages takes these options, so that the same body
    /// always gives the same AST. Pages laid out another way give their own prefixes with
    /// [`LinkResolution::On`].
    pub fn relative_page_links() -> BodyOptions {
        BodyOptions {
            link_resolution: LinkResolution::On {
                prefixes: vec!["./".to_string(), "../".to_string()],
            },
        }
    }
}

/// Parses a flatmarkdown item body into the AST using the given options.
/// The root is a [`Node::Document`]. [`Node::to_json`] writes it as JSON.
pub fn parse_body_with(input: &str, opts: &BodyOptions) -> Node {
    // `\#` is replaced before comrak sees it so that the hashtag pass cannot
    // mistake the escaped `#` for a tag start.
    let preprocessed = mark_escaped_hashes(input);
    let arena = Arena::new();
    let root = parse_document(&arena, &preprocessed, &options(false));
    let mut ast = node_to_ast(root);
    if let Some((completed, placeholder_defs)) = complete_footnotes(&preprocessed, &ast) {
        let arena = Arena::new();
        let root = parse_document(&arena, &completed, &options(false));
        ast = node_to_ast(root);
        strip_footnote_placeholders(&mut ast, &placeholder_defs);
        renumber_footnotes(&mut ast);
    }
    if let LinkResolution::On { prefixes } = &opts.link_resolution {
        convert_markdown_links_to_wikilinks(&mut ast, prefixes);
        process_hashtags(&mut ast);
    }
    restore_escaped_hashes(&mut ast);
    ast
}

/// Parses a flatmarkdown item body into the AST with the default options.
pub fn parse_body(input: &str) -> Node {
    parse_body_with(input, &BodyOptions::default())
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

    // >>>で囲んだ複数行の引用は、multiline_block_quoteになる。中の段落は子になる。
    #[test]
    fn ast_multiline_block_quote() {
        let result = parse_body(">>>\nq1\n\nq2\n>>>");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        assert_eq!(v["children"][0]["type"], "multiline_block_quote");
        assert_eq!(v["children"][0]["children"].as_array().unwrap().len(), 2);
    }

    // [[page]]はwikilinkになる。urlと表示する文字は、どちらもページ名である。
    #[test]
    fn wikilink_ast_basic() {
        let result = parse_body("[[page]]");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "page");
        assert_eq!(wl["children"][0]["type"], "text");
        assert_eq!(wl["children"][0]["value"], "page");
    }

    // [[url|ラベル]]は、|の前がurlに、後ろが表示する文字になる。
    #[test]
    fn wikilink_ast_with_label() {
        let result = parse_body("[[url|link label]]");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "url");
        assert_eq!(wl["children"][0]["value"], "link label");
    }

    // リンク解決がOnのとき、#tagはhashtagになる。valueに#は含まない。
    #[test]
    fn ast_hashtag_basic() {
        let result = parse_body_with(
            "#tag",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "tag");
    }

    // 日本語のハッシュタグもhashtagになる。
    #[test]
    fn ast_hashtag_japanese() {
        let result = parse_body_with(
            "#日記",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "日記");
    }

    // ハッシュタグの名前には、_と-と/を使える。
    #[test]
    fn ast_hashtag_with_special_chars() {
        let result = parse_body_with(
            "#my_tag-name/sub",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "my_tag-name/sub");
    }

    // 文の途中のハッシュタグは、前後の文字と分かれてhashtagになる。
    #[test]
    fn ast_hashtag_in_text() {
        let result = parse_body_with(
            "hello #tag world",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "hello ");
        assert_eq!(children[1]["type"], "hashtag");
        assert_eq!(children[1]["value"], "tag");
        assert_eq!(children[2]["type"], "text");
        assert_eq!(children[2]["value"], " world");
    }

    // 1つの段落に、複数のハッシュタグを書ける。
    #[test]
    fn ast_hashtag_multiple() {
        let result = parse_body_with(
            "#tag1 #tag2",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "tag1");
        assert_eq!(children[1]["type"], "text");
        assert_eq!(children[1]["value"], " ");
        assert_eq!(children[2]["type"], "hashtag");
        assert_eq!(children[2]["value"], "tag2");
    }

    // 単語の途中の#は、ハッシュタグにならない。
    #[test]
    fn ast_hashtag_not_in_middle_of_word() {
        let result = parse_body_with("foo#bar", &with_link_resolution());
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "foo#bar");
    }

    // ハッシュタグは、句読点の前で終わる。
    #[test]
    fn ast_hashtag_terminated_by_punctuation() {
        let result = parse_body_with(
            "#タグ。本文",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "タグ");
        assert_eq!(children[1]["type"], "text");
        assert_eq!(children[1]["value"], "。本文");
    }

    // /で始まる名前は、ハッシュタグにならない。
    #[test]
    fn ast_hashtag_leading_slash_not_hashtag() {
        // "#/tag" must not be recognized as a hashtag (leading '/' is invalid)
        let result = parse_body_with("#/tag", &with_link_resolution());
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#/tag");
    }

    // 末尾の/は、ハッシュタグの名前に含めない。/は文字として残る。
    #[test]
    fn ast_hashtag_trailing_slash_trimmed() {
        // "#tag/" must be recognized as hashtag "tag" (trailing '/' is trimmed)
        let result = parse_body_with(
            "#tag/",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "hashtag");
        assert_eq!(children[0]["value"], "tag");
        assert_eq!(children[1]["type"], "text");
        assert_eq!(children[1]["value"], "/");
    }

    // /で始まる名前は、/で終わっていてもハッシュタグにならない。
    #[test]
    fn ast_hashtag_leading_and_trailing_slash_not_hashtag() {
        // "#/tag/" must not be recognized as a hashtag (leading '/' is invalid)
        let result = parse_body_with("#/tag/", &with_link_resolution());
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#/tag/");
    }

    // #と空白だけの行は見出しである。ハッシュタグではない。
    #[test]
    fn ast_hash_only_not_hashtag() {
        let result = parse_body("# ");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        // "# " is a heading, not a hashtag
        assert_eq!(v["children"][0]["type"], "heading");
    }

    // \#tagはハッシュタグにならない。\は取り除かれ、#tagという文字になる。
    #[test]
    fn ast_escaped_hashtag_basic() {
        let result = parse_body_with(r"\#tag", &with_link_resolution());
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#tag");
    }

    // \をつけた#と、つけない#を、同じ段落に書ける。つけない方だけがhashtagになる。
    #[test]
    fn ast_escaped_hashtag_with_real_hashtag() {
        let result = parse_body_with(
            r"\#notag #real",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#notag ");
        assert_eq!(children[1]["type"], "hashtag");
        assert_eq!(children[1]["value"], "real");
    }

    // \をつけた#が複数あっても、どれもハッシュタグにならない。
    #[test]
    fn ast_escaped_hashtag_multiple() {
        let result = parse_body_with(r"\#a \#b", &with_link_resolution());
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert_eq!(children[0]["type"], "text");
        assert_eq!(children[0]["value"], "#a #b");
    }

    // 画像の記法はembedになる。URLに#がなければ、propsはnullである。
    #[test]
    fn ast_embed_no_props() {
        let result = parse_body("![alt text](img.png)");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "img.png");
        assert_eq!(embed["props"], Value::Null);
        assert_eq!(embed["children"][0]["value"], "alt text");
    }

    // URLの#のあとのkey=valueは、propsに入る。urlには含まない。
    #[test]
    fn ast_embed_with_kv_props() {
        let result = parse_body("![alt](img.png#w=300&h=240)");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "img.png");
        assert_eq!(embed["props"]["w"], "300");
        assert_eq!(embed["props"]["h"], "240");
    }

    // 値のないキーは、propsでtrueになる。
    #[test]
    fn ast_embed_with_boolean_flag() {
        let result = parse_body("![alt](video.mp4#w=640&autoplay)");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "video.mp4");
        assert_eq!(embed["props"]["w"], "640");
        assert_eq!(embed["props"]["autoplay"], true);
    }

    // URLの?のクエリはurlに残る。#のあとだけがpropsになる。
    #[test]
    fn ast_embed_url_with_query_and_fragment() {
        let result = parse_body("![alt](https://youtube.com/watch?v=xxx#w=640&start=30)");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let embed = &v["children"][0]["children"][0];
        assert_eq!(embed["type"], "embed");
        assert_eq!(embed["url"], "https://youtube.com/watch?v=xxx");
        assert_eq!(embed["props"]["w"], "640");
        assert_eq!(embed["props"]["start"], "30");
    }

    // ==text==はhighlightになる。
    #[test]
    fn ast_highlight() {
        let result = parse_body("==highlighted==");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "highlight");
        assert_eq!(node["children"][0]["value"], "highlighted");
    }

    // ^text^はsuperscriptになる。
    #[test]
    fn ast_superscript() {
        let result = parse_body("^super^");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "superscript");
        assert_eq!(node["children"][0]["value"], "super");
    }

    // ~text~はsubscriptになる。
    #[test]
    fn ast_subscript() {
        let result = parse_body("~sub~");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "subscript");
        assert_eq!(node["children"][0]["value"], "sub");
    }

    // ||text||はspoilered_textになる。
    #[test]
    fn ast_spoilered_text() {
        let result = parse_body("||spoiler||");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "spoilered_text");
        assert_eq!(node["children"][0]["value"], "spoiler");
    }

    // __text__はunderlineになる。strongではない。
    #[test]
    fn ast_underline() {
        let result = parse_body("__underlined__");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "underline");
        assert_eq!(node["children"][0]["value"], "underlined");
    }

    // :name:はshortcodeになる。codeに名前が、emojiに絵文字が入る。
    #[test]
    fn ast_shortcode() {
        let result = parse_body(":rabbit:");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "shortcode");
        assert_eq!(node["code"], "rabbit");
        assert!(node["emoji"].is_string());
    }

    // $`式`$はmathになる。literalに式が入る。
    #[test]
    fn ast_math_inline() {
        let result = parse_body("$`x^2`$");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "math");
        assert_eq!(node["literal"], "x^2");
    }

    // > [!NOTE]で始まる引用はalertになる。alert_typeは小文字である。
    #[test]
    fn ast_alert_note() {
        let result = parse_body("> [!NOTE]\n> Content");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0];
        assert_eq!(node["type"], "alert");
        assert_eq!(node["alert_type"], "note");
    }

    // [[page#id]]のurlは、#idを含む。
    #[test]
    fn ast_wikilink_block_link() {
        let result = parse_body("[[page#block-id]]");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "page#block-id");
    }

    // [[page#id|ラベル]]のurlは#idを含み、表示する文字はラベルになる。
    #[test]
    fn ast_wikilink_block_link_with_label() {
        let result = parse_body("[[page#block-id|label]]");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let wl = &v["children"][0]["children"][0];
        assert_eq!(wl["type"], "wikilink");
        assert_eq!(wl["url"], "page#block-id");
        assert_eq!(wl["children"][0]["value"], "label");
    }

    // リンク解決がOnのとき、../で始まる.mdへのリンクはwikilinkになる。urlは.mdを除いたページ名である。
    #[test]
    fn ast_md_link_to_wikilink_no_label() {
        let result = parse_body_with(
            "[pagename](../pagename.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "pagename");
    }

    // wikilinkになったリンクの表示する文字は、元のリンクの文字である。
    #[test]
    fn ast_md_link_to_wikilink_with_label() {
        let result = parse_body_with(
            "[display label](../pagename.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "display label");
    }

    // フォルダを含むパスは、接頭辞と.mdだけを除いてurlになる。
    #[test]
    fn ast_md_link_to_wikilink_with_path() {
        let result = parse_body_with(
            "[label](../journal/2024-01-01.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "journal/2024-01-01");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // .mdのあとの#idは、wikilinkのurlに残る。
    #[test]
    fn ast_md_link_to_wikilink_with_fragment() {
        let result = parse_body_with(
            "[label](../page.md#block-id)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "page#block-id");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // ./で始まる.mdへのリンクもwikilinkになる。
    #[test]
    fn ast_md_link_dotslash_to_wikilink() {
        let result = parse_body_with(
            "[pagename](./pagename.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "pagename");
        assert_eq!(node["children"][0]["value"], "pagename");
    }

    // ./で始まるリンクでも、フォルダを含むパスはurlに残る。
    #[test]
    fn ast_md_link_dotslash_with_path() {
        let result = parse_body_with(
            "[label](./journal/2024-01-01.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "journal/2024-01-01");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // ./で始まるリンクでも、#idはurlに残る。
    #[test]
    fn ast_md_link_dotslash_with_fragment() {
        let result = parse_body_with(
            "[label](./page.md#block-id)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "wikilink");
        assert_eq!(node["url"], "page#block-id");
        assert_eq!(node["children"][0]["value"], "label");
    }

    // 文字が#で始まる./のリンクは、hashtagになる。urlと子は持たない。
    #[test]
    fn ast_md_tag_link_dotslash_to_hashtag() {
        let result = parse_body_with(
            "[#mytag](./mytag.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "hashtag");
        assert_eq!(node["value"], "mytag");
        assert!(node.get("url").is_none());
        assert!(node.get("children").is_none());
    }

    // 文字が#で始まる../のリンクは、hashtagになる。日本語の名前でも同じである。
    #[test]
    fn ast_md_tag_link_to_hashtag() {
        let result = parse_body_with(
            "[#日記](../日記.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On {
                    prefixes: vec!["../".into(), "./".into()],
                },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "hashtag");
        assert_eq!(node["value"], "日記");
        assert!(node.get("url").is_none());
        assert!(node.get("children").is_none());
    }

    // リンク解決がOffのとき、#tagは文字のままである。
    #[test]
    fn ast_no_resolve_hashtag_stays_text() {
        let result = parse_body("#tag");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let children = v["children"][0]["children"].as_array().unwrap();
        assert!(children.iter().all(|c| c["type"] != "hashtag"));
    }

    // リンク解決がOffのとき、文字が#で始まるリンクはlinkのままである。
    #[test]
    fn ast_no_resolve_tag_link_stays_link() {
        let result = parse_body("[#tag](../tag.md)");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // リンク解決がOffのとき、.mdへのリンクはlinkのままである。
    #[test]
    fn ast_no_resolve_md_link_stays_link() {
        let result = parse_body("[text](../page.md)");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // リンク解決がOnでも、接頭辞が1つもなければ、リンクはwikilinkにならない。
    #[test]
    fn ast_on_empty_prefixes_md_link_stays_link() {
        let result = parse_body_with(
            "[text](../page.md)",
            &BodyOptions {
                link_resolution: LinkResolution::On { prefixes: vec![] },
            },
        );
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // リンク解決がOnでも、外部のURLへのリンクはlinkのままである。
    #[test]
    fn ast_external_link_not_converted_to_wikilink() {
        let result = parse_body_with("[text](https://example.com)", &with_link_resolution());
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // リンク解決がOnでも、.mdで終わらない相対リンクはlinkのままである。
    #[test]
    fn ast_non_md_relative_link_not_converted() {
        let result = parse_body_with("[text](../page.html)", &with_link_resolution());
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let node = &v["children"][0]["children"][0];
        assert_eq!(node["type"], "link");
    }

    // 項目の間に空行のあるリストは、1つのlistのままである。tightはfalseになる。
    #[test]
    fn ast_blank_line_between_list_items_keeps_one_list() {
        let result = parse_body("- a\n\n- b");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let blocks = v["children"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "list");
        assert_eq!(blocks[0]["tight"], false);
    }

    // コードフェンスの中の空行は、literalに残る。
    #[test]
    fn ast_blank_line_in_code_fence_preserved() {
        let result = parse_body("```\ncode\n\nmore code\n```");
        let v: Value = serde_json::from_str(&result.to_json()).unwrap();
        let cb = &v["children"][0];
        assert_eq!(cb["type"], "code_block");
        let literal = cb["literal"].as_str().unwrap();
        assert!(
            literal.contains("\n\n"),
            "blank line inside fence must be preserved"
        );
    }
}
