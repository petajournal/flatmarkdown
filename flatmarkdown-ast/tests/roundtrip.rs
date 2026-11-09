//! Round-trips Flat Markdown bodies through the `flatmarkdown` crate (with the options of the
//! PetaJournal journal server) and checks that the typed model covers every node the parser emits.

use flatmarkdown::{BodyOptions, LinkResolution};
use flatmarkdown_ast::{Node, extract_text, heading_level, parse};

/// The same options as the PetaJournal journal server: link resolution on.
fn options() -> BodyOptions {
    BodyOptions {
        link_resolution: LinkResolution::On {
            prefixes: vec!["./".to_string(), "../".to_string()],
        },
    }
}

/// Parses `md` and asserts no node fell back to `Unknown`.
fn ast(md: &str) -> Node {
    let json = flatmarkdown::body_to_ast_with(md, &options());
    let root = parse(&json).unwrap_or_else(|e| panic!("parse failed for {md:?}: {e}\n{json}"));
    let mut unknown = Vec::new();
    root.walk(&mut |n| {
        if let Node::Unknown { node_type, .. } = n {
            unknown.push(node_type.clone());
        }
    });
    assert!(
        unknown.is_empty(),
        "unknown node types {unknown:?} in {json}"
    );
    root
}

/// All type names in the tree, depth first.
fn type_names(root: &Node) -> Vec<&str> {
    let mut names = Vec::new();
    root.walk(&mut |n| names.push(n.type_name()));
    names
}

fn first_block(root: &Node) -> &Node {
    &root.children()[0]
}

// インライン書式を含む段落は document 配下の paragraph として解析され、子ノードの型の並びと抽出テキストがそろう。
#[test]
fn paragraph_with_inline_formatting() {
    let root = ast("hello *em* **strong** `code`");
    assert!(matches!(root, Node::Document { .. }));
    let para = first_block(&root);
    assert!(matches!(para, Node::Paragraph { .. }));
    let kinds: Vec<&str> = para.children().iter().map(Node::type_name).collect();
    assert_eq!(kinds, ["text", "emph", "text", "strong", "text", "code"]);
    assert_eq!(extract_text(&root), "hello em strong ");
    assert_eq!(heading_level(&root), None);
}

// レベル1から6の見出しは Heading ノードとして解析され、heading_level はトップレベルの最初の見出しのレベルを返す。
#[test]
fn headings_one_to_six() {
    for level in 1..=6u8 {
        let md = format!("{} Title {level}", "#".repeat(level as usize));
        let root = ast(&md);
        match first_block(&root) {
            Node::Heading {
                level: l, children, ..
            } => {
                assert_eq!(*l, level);
                assert_eq!(extract_text(&children[0]), format!("Title {level}"));
            }
            other => panic!("expected heading, got {other:?}"),
        }
        assert_eq!(heading_level(&root), Some(level));
    }
    // The first heading among the top-level children wins, even after a paragraph.
    let root = ast("intro\n\n### Third");
    assert_eq!(heading_level(&root), Some(3));
}

// 情報文字列つきのフェンスコードブロックは info と literal を持つ CodeBlock として解析され、その中身は抽出テキストに含まれない。
#[test]
fn fenced_code_block_with_info() {
    let root = ast("```rust\nfn main() {}\n```");
    match first_block(&root) {
        Node::CodeBlock { info, literal, .. } => {
            assert_eq!(info, "rust");
            assert_eq!(literal, "fn main() {}\n");
        }
        other => panic!("expected code_block, got {other:?}"),
    }
    assert_eq!(extract_text(&root), "");
}

// タイトな箇条書きとルーズな番号付きリストは List ノードとして解析され、tight・start・delimiter が正しく入る。
#[test]
fn tight_and_loose_lists() {
    let tight = ast("- a\n- b");
    match first_block(&tight) {
        Node::List {
            list_type,
            tight,
            children,
            ..
        } => {
            assert_eq!(list_type, "bullet");
            assert!(*tight);
            assert_eq!(children.len(), 2);
            // comrak only records tightness on the list; its items say `tight: false`.
            assert!(matches!(&children[0], Node::Item { list_type, .. } if list_type == "bullet"));
        }
        other => panic!("expected list, got {other:?}"),
    }
    let loose = ast("1. a\n\n2. b");
    match first_block(&loose) {
        Node::List {
            list_type,
            tight,
            start,
            delimiter,
            ..
        } => {
            assert_eq!(list_type, "ordered");
            assert!(!*tight);
            assert_eq!(*start, Some(1));
            assert_eq!(delimiter, "period");
        }
        other => panic!("expected list, got {other:?}"),
    }
    let offset = ast("3) c\n4) d");
    assert!(matches!(
        first_block(&offset),
        Node::List { start: Some(3), .. }
    ));
    assert_eq!(extract_text(&tight), "ab");
}

// タスクリストの項目は TaskItem として解析され、未完了は symbol が None、完了は "x" になる。
#[test]
fn task_items() {
    let root = ast("- [ ] todo\n- [x] done");
    let items = first_block(&root).children();
    assert!(matches!(&items[0], Node::TaskItem { symbol: None, .. }));
    match &items[1] {
        Node::TaskItem { symbol, .. } => assert_eq!(symbol.as_deref(), Some("x")),
        other => panic!("expected task_item, got {other:?}"),
    }
    assert_eq!(extract_text(&root), "tododone");
}

// 配置指定つきのテーブルは alignments・列数・行数・ヘッダ行を持つ Table として解析される。
#[test]
fn table_with_alignment() {
    let root = ast("| a | b | c |\n|:--|:-:|--:|\n| 1 | 2 | 3 |");
    match first_block(&root) {
        Node::Table {
            alignments,
            num_columns,
            num_rows,
            children,
        } => {
            assert_eq!(alignments, &["left", "center", "right"]);
            assert_eq!(*num_columns, 3);
            assert_eq!(*num_rows, 2);
            assert!(matches!(&children[0], Node::TableRow { header: true, .. }));
            assert!(matches!(&children[1], Node::TableRow { header: false, .. }));
            assert!(matches!(&children[0].children()[0], Node::TableCell { .. }));
        }
        other => panic!("expected table, got {other:?}"),
    }
    assert_eq!(extract_text(&root), "abc123");
}

// 引用と水平線は BlockQuote と ThematicBreak として解析される。
#[test]
fn blockquote_and_thematic_break() {
    let root = ast("> quoted\n\n---");
    assert!(matches!(first_block(&root), Node::BlockQuote { .. }));
    assert!(matches!(root.children()[1], Node::ThematicBreak));
    assert_eq!(extract_text(&root), "quoted");
}

// リンクは url と title を持つ Link として解析され、Markdown の画像は flatmarkdown が出す Embed になる。npm パッケージ形式の image ノードも Image として解析される。
#[test]
fn link_and_image() {
    let root = ast("[site](https://example.com \"Title\") ![alt](pic.png)");
    let inlines = first_block(&root).children();
    match &inlines[0] {
        Node::Link {
            url,
            title,
            children,
        } => {
            assert_eq!(url, "https://example.com");
            assert_eq!(title, "Title");
            assert_eq!(extract_text(&children[0]), "site");
        }
        other => panic!("expected link, got {other:?}"),
    }
    // The flatmarkdown crate emits `embed` for Markdown images.
    match &inlines[2] {
        Node::Embed {
            url,
            props,
            children,
            ..
        } => {
            assert_eq!(url, "pic.png");
            assert!(props.is_empty());
            assert_eq!(extract_text(&children[0]), "alt");
        }
        other => panic!("expected embed, got {other:?}"),
    }
    // An `image` node as defined by the npm package parses too.
    let image = parse(
        r#"{"type":"image","url":"a.png","title":"","children":[{"type":"text","value":"x"}]}"#,
    )
    .unwrap();
    assert!(matches!(image, Node::Image { .. }));
    assert_eq!(extract_text(&image), "x");
}

// ウィキリンクとハッシュタグは Wikilink と Hashtag として解析され、サーバーのオプションでは相対 Markdown リンクも同じノードに解決される。
#[test]
fn wikilink_and_hashtag() {
    let root = ast("see [[Other Page]] and #topic");
    let inlines = first_block(&root).children();
    match &inlines[1] {
        Node::Wikilink { url, children } => {
            assert_eq!(url, "Other Page");
            assert_eq!(extract_text(&children[0]), "Other Page");
        }
        other => panic!("expected wikilink, got {other:?}"),
    }
    match &inlines[3] {
        Node::Hashtag { value } => assert_eq!(value, "topic"),
        other => panic!("expected hashtag, got {other:?}"),
    }
    // Relative Markdown links are resolved to wikilinks / hashtags by the server options.
    let root = ast("[Note](./note.md) [#tag](../tag.md)");
    let inlines = first_block(&root).children();
    assert!(matches!(&inlines[0], Node::Wikilink { url, .. } if url == "note"));
    assert!(matches!(&inlines[2], Node::Hashtag { value } if value == "tag"));
}

// 脚注参照と脚注定義は名前を持つ FootnoteReference と FootnoteDefinition として解析される。
#[test]
fn footnote_reference_and_definition() {
    let root = ast("text[^1]\n\n[^1]: the note");
    let names = type_names(&root);
    assert!(names.contains(&"footnote_reference"), "{names:?}");
    assert!(names.contains(&"footnote_definition"), "{names:?}");
    let reference = &first_block(&root).children()[1];
    match reference {
        Node::FootnoteReference { name, .. } => assert_eq!(name, "1"),
        other => panic!("expected footnote_reference, got {other:?}"),
    }
    let definition = root
        .children()
        .iter()
        .find(|n| matches!(n, Node::FootnoteDefinition { .. }))
        .unwrap();
    match definition {
        Node::FootnoteDefinition { name, children } => {
            assert_eq!(name, "1");
            assert!(matches!(&children[0], Node::Paragraph { .. }));
            assert_eq!(extract_text(definition), "the note");
        }
        other => panic!("expected footnote_definition, got {other:?}"),
    }
}

// 取り消し線は Strikethrough ノードとして解析される。
#[test]
fn strikethrough() {
    let root = ast("~~gone~~");
    assert!(matches!(
        &first_block(&root).children()[0],
        Node::Strikethrough { .. }
    ));
    assert_eq!(extract_text(&root), "gone");
}

// 段落内の改行は linebreak か softbreak として解析され、JSON の softbreak と linebreak ノードはそれぞれの型になる。
#[test]
fn softbreak_and_linebreak() {
    // In Flat Markdown a newline inside a paragraph is a hard line break.
    let root = ast("first\nsecond");
    let kinds: Vec<&str> = first_block(&root)
        .children()
        .iter()
        .map(Node::type_name)
        .collect();
    assert!(
        kinds.contains(&"linebreak") || kinds.contains(&"softbreak"),
        "{kinds:?}"
    );
    assert_eq!(extract_text(&root), "firstsecond");
    let soft = parse(r#"{"type":"paragraph","children":[{"type":"text","value":"a"},{"type":"softbreak"},{"type":"text","value":"b"}]}"#).unwrap();
    assert!(matches!(soft.children()[1], Node::Softbreak));
    let hard = parse(r#"{"type":"paragraph","children":[{"type":"linebreak"}]}"#).unwrap();
    assert!(matches!(hard.children()[0], Node::Linebreak));
}

// インライン HTML は html_inline として、ブロック HTML は literal を持つ HtmlBlock として解析される。
#[test]
fn html_inline_and_block() {
    let root = ast("a <b>bold</b> c");
    let kinds: Vec<&str> = first_block(&root)
        .children()
        .iter()
        .map(Node::type_name)
        .collect();
    assert!(kinds.contains(&"html_inline"), "{kinds:?}");
    let root = ast("<div>\nblock\n</div>");
    match first_block(&root) {
        Node::HtmlBlock { literal, .. } => assert!(literal.contains("<div>")),
        other => panic!("expected html_block, got {other:?}"),
    }
}

// 未知の型のノードはエラーにならず Unknown として子を保持し、余分なフィールドは無視される。
#[test]
fn unknown_type_is_tolerated_and_keeps_children() {
    let json = r#"{"type":"document","children":[
        {"type":"paragraph","children":[
            {"type":"text","value":"a"},
            {"type":"insert","children":[{"type":"text","value":"b"}]},
            {"type":"text","value":"c","extra_field":1}
        ]}]}"#;
    let root = parse(json).unwrap();
    let para = first_block(&root);
    match &para.children()[1] {
        Node::Unknown {
            node_type,
            children,
        } => {
            assert_eq!(node_type, "insert");
            assert!(matches!(&children[0], Node::Text { value } if value == "b"));
        }
        other => panic!("expected unknown, got {other:?}"),
    }
    assert_eq!(extract_text(&root), "abc");
    assert_eq!(
        type_names(&root),
        ["document", "paragraph", "text", "insert", "text", "text"]
    );
}

// 既知の型でフィールドの型が合わないノード、type のないノード、JSON でない文字列の解析はエラーになる。
#[test]
fn malformed_known_node_is_an_error() {
    assert!(parse(r#"{"type":"heading","level":"two"}"#).is_err());
    assert!(parse(r#"{"children":[]}"#).is_err());
    assert!(parse("not json").is_err());
}

// 省略された任意フィールドは既定値になる。children は空、fenced と display_math は false である。
#[test]
fn missing_optional_fields_default() {
    let root = parse(r#"{"type":"document"}"#).unwrap();
    assert!(root.children().is_empty());
    let cb = parse(r#"{"type":"code_block"}"#).unwrap();
    assert!(matches!(cb, Node::CodeBlock { fenced: false, .. }));
    let math = parse(r#"{"type":"math","literal":"x"}"#).unwrap();
    assert!(matches!(
        math,
        Node::Math {
            display_math: false,
            ..
        }
    ));
}

// 解析した AST は type タグつきの JSON にシリアライズされ、Unknown ノードの type は "unknown" になる。
#[test]
fn serializes_back_to_tagged_json() {
    let root = ast("# Hi");
    let json = serde_json::to_value(&root).unwrap();
    assert_eq!(json["type"], "document");
    assert_eq!(json["children"][0]["type"], "heading");
    assert_eq!(json["children"][0]["level"], 1);
    let unknown = Node::Unknown {
        node_type: "insert".into(),
        children: vec![],
    };
    assert_eq!(serde_json::to_value(&unknown).unwrap()["type"], "unknown");
}

// パーサーの出力は、型で読んで書き戻しても、同じJSONの値である。本文でもページでも成り立つ。
#[test]
fn parser_output_is_written_back_unchanged() {
    let bodies = [
        "# H\n\npara *e* **s** ~~d~~ `c` ==h== ^s^ ~s~ ||sp|| __u__ :smile: $`x`$ <b>i</b>\nline",
        "```rust\ncode\n```\n\n> q\n\n>>>\nmq\n>>>\n\n---\n\n<div>\nb\n</div>",
        "- a\n- b\n\n1. x\n\n2. y\n\n- [ ] t\n- [x] d\n\n| a | b |\n|:--|--:|\n| 1 | 2 |",
        "> [!NOTE]\n> n\n\n> [!WARNING] Title\n> n\n\ntext[^1] ^[inline]\n\n[^1]: note",
        "[l](https://e.com \"T\") ![alt](p.png) ![a](p.png#w=10&flag) [[P]] [[P#id|lab]] #tag [N](./n.md) [#t](../t.md)",
        "",
        "-# sub\n\n- \n",
    ];
    for md in bodies {
        let json = flatmarkdown::body_to_ast_with(md, &options());
        let original: serde_json::Value = serde_json::from_str(&json).unwrap();
        let written = serde_json::to_value(parse(&json).unwrap()).unwrap();
        assert_eq!(written, original, "body {md:?}");
    }

    let page_md = "---\nk: v\n---\n\n<!-- @ 2026-09-26-18-14-47-32W15WT plan=\"2026-09-28\" -->\n# Title\n\nText[^1].\n\n[^1]: Note.\n\n<!-- @@ -->\n";
    for md in [page_md, "", "plain"] {
        let json = flatmarkdown::page_to_ast_with(md, &options());
        let original: serde_json::Value = serde_json::from_str(&json).unwrap();
        let written = serde_json::to_value(flatmarkdown_ast::parse_page(&json).unwrap()).unwrap();
        assert_eq!(written, original, "page {md:?}");
    }
}
