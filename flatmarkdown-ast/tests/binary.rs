//! A binary format (bincode) gets the plain form: each variant by its index in the order of the
//! types, and each field in order. The types that `facet` generates for other languages read
//! this form, so the index of a variant must be its place in the type.

use flatmarkdown_ast::{AlertType, Alignment, ListDelimiter, ListType, Node, PropValue};
use std::collections::BTreeMap;

fn text(value: &str) -> Node {
    Node::Text {
        value: value.to_string(),
    }
}

/// One node of each kind, in the order of the variants of `Node`.
fn one_of_each() -> Vec<Node> {
    let kids = || vec![text("x")];
    vec![
        Node::Document { children: kids() },
        Node::Paragraph { children: kids() },
        Node::Heading {
            level: 2,
            children: kids(),
        },
        Node::CodeBlock {
            fenced: false,
            info: "rs".into(),
            literal: "x".into(),
        },
        Node::BlockQuote { children: kids() },
        Node::MultilineBlockQuote { children: kids() },
        Node::List {
            list_type: ListType::Ordered,
            start: 3,
            tight: true,
            delimiter: ListDelimiter::Paren,
            children: kids(),
        },
        Node::Item {
            list_type: ListType::Ordered,
            start: 3,
            children: kids(),
        },
        Node::TaskItem {
            symbol: Some("x".into()),
            children: kids(),
        },
        Node::Table {
            alignments: vec![Alignment::None, Alignment::Right],
            num_columns: 2,
            num_rows: 1,
            children: kids(),
        },
        Node::TableRow {
            header: true,
            children: kids(),
        },
        Node::TableCell { children: kids() },
        Node::ThematicBreak,
        Node::HtmlBlock {
            block_type: 6,
            literal: "<div>".into(),
        },
        Node::FootnoteDefinition {
            name: "n".into(),
            children: kids(),
        },
        Node::Alert {
            alert_type: AlertType::Caution,
            title: None,
            children: kids(),
        },
        Node::Subtext { children: kids() },
        text("t"),
        Node::Softbreak,
        Node::Linebreak,
        Node::Emph { children: kids() },
        Node::Strong { children: kids() },
        Node::Strikethrough { children: kids() },
        Node::Underline { children: kids() },
        Node::Highlight { children: kids() },
        Node::Superscript { children: kids() },
        Node::Subscript { children: kids() },
        Node::SpoileredText { children: kids() },
        Node::Code {
            literal: "c".into(),
        },
        Node::Link {
            url: "u".into(),
            title: "t".into(),
            children: kids(),
        },
        Node::Embed {
            url: "p.png".into(),
            title: String::new(),
            props: BTreeMap::from([
                ("w".to_string(), PropValue::Text("10".into())),
                ("inline".to_string(), PropValue::Flag(true)),
            ]),
            children: kids(),
        },
        Node::FootnoteReference {
            name: "n".into(),
            ref_num: 1,
            ix: 2,
        },
        Node::Shortcode {
            code: "smile".into(),
            emoji: "S".into(),
        },
        Node::Math {
            literal: "m".into(),
        },
        Node::HtmlInline {
            value: "<b>".into(),
        },
        Node::Wikilink {
            url: "P".into(),
            children: kids(),
        },
        Node::Hashtag { value: "h".into() },
        Node::Unknown {
            node_type: "insert".into(),
            children: kids(),
        },
    ]
}

// バイナリの形式では、種類の番号は、Nodeの種類を並べた順である。
#[test]
fn the_index_of_a_kind_in_a_binary_format_is_its_place_in_the_type() {
    for (place, node) in one_of_each().iter().enumerate() {
        let bytes = bincode::serialize(node).unwrap();
        let index = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        assert_eq!(index as usize, place, "{}", node.type_name());
    }
}

// どの種類のノードも、バイナリの形式で書いて読むと、同じノードに戻る。未知の種類は、種類名を保つ。
#[test]
fn every_kind_is_read_back_from_a_binary_format_as_the_same_node() {
    let document = Node::Document {
        children: one_of_each(),
    };
    let bytes = bincode::serialize(&document).unwrap();
    assert_eq!(bincode::deserialize::<Node>(&bytes).unwrap(), document);
}

// JSONの形は、バイナリの形式とは別である。空のchildrenを省き、種類をtypeの名前で書く。
#[test]
fn the_json_form_is_kept_beside_the_binary_form() {
    let node = Node::Alert {
        alert_type: AlertType::Tip,
        title: None,
        children: vec![],
    };
    assert_eq!(
        node.to_json(),
        r#"{"type":"alert","alert_type":"tip","title":null}"#
    );
}

// embedのpropsの値は、JSONでは文字列か真偽値である。数はエラーになる。
#[test]
fn a_prop_of_an_embed_in_json_is_a_string_or_a_boolean() {
    let embed =
        Node::from_json(r#"{"type":"embed","url":"p.png","props":{"w":"10","inline":true}}"#)
            .unwrap();
    let Node::Embed { props, .. } = embed else {
        panic!("not an embed");
    };
    assert_eq!(props["w"], PropValue::Text("10".into()));
    assert_eq!(props["inline"], PropValue::Flag(true));
    assert!(Node::from_json(r#"{"type":"embed","props":{"w":10}}"#).is_err());
}
