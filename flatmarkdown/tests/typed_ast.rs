//! Parses Flat Markdown bodies (with the options of pages linked by relative paths) and checks the
//! typed nodes that the parser makes: their kinds, their attributes, and the text they hold.

use flatmarkdown::BodyOptions;
use flatmarkdown_ast::{Alignment, ListDelimiter, ListType, Node, extract_text};

fn options() -> BodyOptions {
    BodyOptions::relative_page_links()
}

/// Parses `md` and asserts no node fell back to `Unknown`.
fn ast(md: &str) -> Node {
    let json = flatmarkdown::parse_body_with(md, &options()).to_json();
    let root =
        Node::from_json(&json).unwrap_or_else(|e| panic!("parse failed for {md:?}: {e}\n{json}"));
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

fn text(value: &str) -> Node {
    Node::Text {
        value: value.to_string(),
    }
}

// インライン書式を含む段落は document 配下の paragraph として解析され、子ノードの型が書いた順に並ぶ。
#[test]
fn paragraph_with_inline_formatting() {
    let root = ast("hello *em* **strong** `code`");
    assert!(matches!(root, Node::Document { .. }));
    let para = first_block(&root);
    assert!(matches!(para, Node::Paragraph { .. }));
    let kinds: Vec<&str> = para.children().iter().map(Node::type_name).collect();
    assert_eq!(kinds, ["text", "emph", "text", "strong", "text", "code"]);
}

// レベル1から6の見出しは、そのレベルを持つ Heading ノードとして解析される。
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
    }
}

// 情報文字列つきのフェンスコードブロックは info と literal を持つ CodeBlock として解析される。
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
            assert_eq!(*list_type, ListType::Bullet);
            assert!(*tight);
            assert_eq!(children.len(), 2);
            assert!(
                matches!(&children[0], Node::Item { list_type, .. } if *list_type == ListType::Bullet)
            );
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
            assert_eq!(*list_type, ListType::Ordered);
            assert!(!*tight);
            assert_eq!(*start, 1);
            assert_eq!(*delimiter, ListDelimiter::Period);
        }
        other => panic!("expected list, got {other:?}"),
    }
    let offset = ast("3) c\n4) d");
    assert!(matches!(first_block(&offset), Node::List { start: 3, .. }));
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
            assert_eq!(
                alignments,
                &[Alignment::Left, Alignment::Center, Alignment::Right]
            );
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

// リンクは url と title を持つ Link として解析され、Markdown の画像は Embed になる。
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

// 段落の中のただの改行は softbreak になる。
#[test]
fn a_plain_newline_in_a_paragraph_is_a_softbreak() {
    let root = ast("line1\nline2");
    assert_eq!(
        first_block(&root).children(),
        [text("line1"), Node::Softbreak, text("line2")]
    );
}

// 行末の2つ以上の空白と、行末の\は、linebreak になる。
#[test]
fn trailing_spaces_and_a_trailing_backslash_are_a_linebreak() {
    for md in ["line1  \nline2", "line1\\\nline2"] {
        let root = ast(md);
        assert_eq!(
            first_block(&root).children(),
            [text("line1"), Node::Linebreak, text("line2")],
            "{md:?}"
        );
    }
}
