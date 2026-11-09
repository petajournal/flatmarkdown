//! Typed serde model of the Flat Markdown AST.
//!
//! The JSON is produced by the `flatmarkdown` crate (`body_to_ast*`) and mirrors the
//! `flatmarkdown-ast` npm package: every node is an object discriminated by its `type`
//! field (snake_case), with an optional `children` array.
//!
//! Deserialization is tolerant: unknown fields are ignored, scalar fields the producer
//! omits fall back to their defaults, and a node whose `type` is not modelled here becomes
//! [`Node::Unknown`] (keeping its children) instead of failing the whole parse.

use std::collections::BTreeMap;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// One node of the Flat Markdown AST.
///
/// Variants follow the `flatmarkdown-ast` TypeScript definitions. Nodes that may contain
/// other nodes carry `children`; leaf nodes do not.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", remote = "Self")]
pub enum Node {
    // ── Block nodes ──
    Document {
        #[serde(default)]
        children: Vec<Node>,
    },
    Paragraph {
        #[serde(default)]
        children: Vec<Node>,
    },
    Heading {
        #[serde(default)]
        level: u8,
        #[serde(default)]
        setext: bool,
        #[serde(default)]
        children: Vec<Node>,
    },
    CodeBlock {
        #[serde(default)]
        fenced: bool,
        #[serde(default)]
        info: String,
        #[serde(default)]
        literal: String,
    },
    BlockQuote {
        #[serde(default)]
        children: Vec<Node>,
    },
    MultilineBlockQuote {
        #[serde(default)]
        children: Vec<Node>,
    },
    List {
        /// `"bullet"` or `"ordered"`.
        #[serde(default)]
        list_type: String,
        #[serde(default)]
        start: Option<u32>,
        #[serde(default)]
        tight: bool,
        /// `"period"` or `"paren"`.
        #[serde(default)]
        delimiter: String,
        #[serde(default)]
        children: Vec<Node>,
    },
    Item {
        #[serde(default)]
        list_type: String,
        #[serde(default)]
        start: u32,
        #[serde(default)]
        tight: bool,
        #[serde(default)]
        children: Vec<Node>,
    },
    TaskItem {
        /// The character inside the brackets (`"x"`), or `None` for an unchecked `[ ]`.
        #[serde(default)]
        symbol: Option<String>,
        #[serde(default)]
        children: Vec<Node>,
    },
    Table {
        /// One of `"none"`, `"left"`, `"center"`, `"right"` per column.
        #[serde(default)]
        alignments: Vec<String>,
        #[serde(default)]
        num_columns: usize,
        #[serde(default)]
        num_rows: usize,
        #[serde(default)]
        children: Vec<Node>,
    },
    TableRow {
        #[serde(default)]
        header: bool,
        #[serde(default)]
        children: Vec<Node>,
    },
    TableCell {
        #[serde(default)]
        children: Vec<Node>,
    },
    ThematicBreak,
    HtmlBlock {
        #[serde(default)]
        block_type: u8,
        #[serde(default)]
        literal: String,
    },
    FootnoteDefinition {
        #[serde(default)]
        name: String,
        #[serde(default)]
        children: Vec<Node>,
    },
    DescriptionList {
        #[serde(default)]
        children: Vec<Node>,
    },
    DescriptionItem {
        #[serde(default)]
        children: Vec<Node>,
    },
    DescriptionTerm {
        #[serde(default)]
        children: Vec<Node>,
    },
    DescriptionDetails {
        #[serde(default)]
        children: Vec<Node>,
    },
    Alert {
        #[serde(default)]
        alert_type: String,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        children: Vec<Node>,
    },
    FrontMatter {
        #[serde(default)]
        value: String,
    },
    HeexBlock {
        #[serde(default)]
        children: Vec<Node>,
    },
    Subtext {
        #[serde(default)]
        children: Vec<Node>,
    },

    // ── Inline nodes ──
    Text {
        #[serde(default)]
        value: String,
    },
    Softbreak,
    Linebreak,
    Emph {
        #[serde(default)]
        children: Vec<Node>,
    },
    Strong {
        #[serde(default)]
        children: Vec<Node>,
    },
    Strikethrough {
        #[serde(default)]
        children: Vec<Node>,
    },
    Underline {
        #[serde(default)]
        children: Vec<Node>,
    },
    Highlight {
        #[serde(default)]
        children: Vec<Node>,
    },
    Superscript {
        #[serde(default)]
        children: Vec<Node>,
    },
    Subscript {
        #[serde(default)]
        children: Vec<Node>,
    },
    SpoileredText {
        #[serde(default)]
        children: Vec<Node>,
    },
    Code {
        #[serde(default)]
        literal: String,
    },
    Link {
        #[serde(default)]
        url: String,
        #[serde(default)]
        title: String,
        #[serde(default)]
        children: Vec<Node>,
    },
    Image {
        #[serde(default)]
        url: String,
        #[serde(default)]
        title: String,
        #[serde(default)]
        children: Vec<Node>,
    },
    /// What the `flatmarkdown` crate emits for `![alt](url#k=v)`: an image with the URL
    /// fragment split into `props`. Not part of the npm type definitions.
    Embed {
        #[serde(default)]
        url: String,
        #[serde(default)]
        title: String,
        #[serde(default)]
        props: BTreeMap<String, Value>,
        #[serde(default)]
        children: Vec<Node>,
    },
    FootnoteReference {
        #[serde(default)]
        name: String,
        #[serde(default)]
        ref_num: u32,
        #[serde(default)]
        ix: u32,
    },
    Shortcode {
        #[serde(default)]
        code: String,
        #[serde(default)]
        emoji: String,
    },
    Math {
        #[serde(default)]
        dollar_math: bool,
        #[serde(default)]
        display_math: bool,
        #[serde(default)]
        literal: String,
    },
    HtmlInline {
        #[serde(default)]
        value: String,
    },
    HeexInline {
        #[serde(default)]
        value: String,
    },
    Raw {
        #[serde(default)]
        value: String,
    },
    Escaped,
    EscapedTag {
        #[serde(default)]
        value: String,
    },
    Wikilink {
        #[serde(default)]
        url: String,
        #[serde(default)]
        children: Vec<Node>,
    },
    Hashtag {
        #[serde(default)]
        value: String,
    },

    /// A node whose `type` this crate does not model. Its children are still parsed.
    /// Serializes as `{"type":"unknown","children":[...]}`.
    Unknown {
        #[serde(skip)]
        node_type: String,
        #[serde(default)]
        children: Vec<Node>,
    },
}

impl Serialize for Node {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Node::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        Node::from_value(&value).map_err(D::Error::custom)
    }
}

impl Node {
    /// Builds a node from a JSON value, mapping an unknown `type` to [`Node::Unknown`].
    fn from_value(value: &Value) -> Result<Node, serde_json::Error> {
        match Node::deserialize(value) {
            Ok(node) => Ok(node),
            // The derived deserializer reports an unmodelled tag as `unknown variant`;
            // every other failure (wrong field type, missing tag) is a real error.
            Err(err) if err.to_string().starts_with("unknown variant") => {
                let node_type = value
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                let children = match value.get("children") {
                    Some(Value::Array(items)) => items
                        .iter()
                        .map(Node::from_value)
                        .collect::<Result<Vec<_>, _>>()?,
                    _ => Vec::new(),
                };
                Ok(Node::Unknown {
                    node_type,
                    children,
                })
            }
            Err(err) => Err(err),
        }
    }

    /// The child nodes; an empty slice for leaf nodes.
    pub fn children(&self) -> &[Node] {
        match self {
            Node::Document { children }
            | Node::Paragraph { children }
            | Node::Heading { children, .. }
            | Node::BlockQuote { children }
            | Node::MultilineBlockQuote { children }
            | Node::List { children, .. }
            | Node::Item { children, .. }
            | Node::TaskItem { children, .. }
            | Node::Table { children, .. }
            | Node::TableRow { children, .. }
            | Node::TableCell { children }
            | Node::FootnoteDefinition { children, .. }
            | Node::DescriptionList { children }
            | Node::DescriptionItem { children }
            | Node::DescriptionTerm { children }
            | Node::DescriptionDetails { children }
            | Node::Alert { children, .. }
            | Node::HeexBlock { children }
            | Node::Subtext { children }
            | Node::Emph { children }
            | Node::Strong { children }
            | Node::Strikethrough { children }
            | Node::Underline { children }
            | Node::Highlight { children }
            | Node::Superscript { children }
            | Node::Subscript { children }
            | Node::SpoileredText { children }
            | Node::Link { children, .. }
            | Node::Image { children, .. }
            | Node::Embed { children, .. }
            | Node::Wikilink { children, .. }
            | Node::Unknown { children, .. } => children,
            Node::CodeBlock { .. }
            | Node::ThematicBreak
            | Node::HtmlBlock { .. }
            | Node::FrontMatter { .. }
            | Node::Text { .. }
            | Node::Softbreak
            | Node::Linebreak
            | Node::Code { .. }
            | Node::FootnoteReference { .. }
            | Node::Shortcode { .. }
            | Node::Math { .. }
            | Node::HtmlInline { .. }
            | Node::HeexInline { .. }
            | Node::Raw { .. }
            | Node::Escaped
            | Node::EscapedTag { .. }
            | Node::Hashtag { .. } => &[],
        }
    }

    /// The `type` tag as it appears in the JSON (`"code_block"`, `"task_item"`, ...).
    pub fn type_name(&self) -> &str {
        match self {
            Node::Document { .. } => "document",
            Node::Paragraph { .. } => "paragraph",
            Node::Heading { .. } => "heading",
            Node::CodeBlock { .. } => "code_block",
            Node::BlockQuote { .. } => "block_quote",
            Node::MultilineBlockQuote { .. } => "multiline_block_quote",
            Node::List { .. } => "list",
            Node::Item { .. } => "item",
            Node::TaskItem { .. } => "task_item",
            Node::Table { .. } => "table",
            Node::TableRow { .. } => "table_row",
            Node::TableCell { .. } => "table_cell",
            Node::ThematicBreak => "thematic_break",
            Node::HtmlBlock { .. } => "html_block",
            Node::FootnoteDefinition { .. } => "footnote_definition",
            Node::DescriptionList { .. } => "description_list",
            Node::DescriptionItem { .. } => "description_item",
            Node::DescriptionTerm { .. } => "description_term",
            Node::DescriptionDetails { .. } => "description_details",
            Node::Alert { .. } => "alert",
            Node::FrontMatter { .. } => "front_matter",
            Node::HeexBlock { .. } => "heex_block",
            Node::Subtext { .. } => "subtext",
            Node::Text { .. } => "text",
            Node::Softbreak => "softbreak",
            Node::Linebreak => "linebreak",
            Node::Emph { .. } => "emph",
            Node::Strong { .. } => "strong",
            Node::Strikethrough { .. } => "strikethrough",
            Node::Underline { .. } => "underline",
            Node::Highlight { .. } => "highlight",
            Node::Superscript { .. } => "superscript",
            Node::Subscript { .. } => "subscript",
            Node::SpoileredText { .. } => "spoilered_text",
            Node::Code { .. } => "code",
            Node::Link { .. } => "link",
            Node::Image { .. } => "image",
            Node::Embed { .. } => "embed",
            Node::FootnoteReference { .. } => "footnote_reference",
            Node::Shortcode { .. } => "shortcode",
            Node::Math { .. } => "math",
            Node::HtmlInline { .. } => "html_inline",
            Node::HeexInline { .. } => "heex_inline",
            Node::Raw { .. } => "raw",
            Node::Escaped => "escaped",
            Node::EscapedTag { .. } => "escaped_tag",
            Node::Wikilink { .. } => "wikilink",
            Node::Hashtag { .. } => "hashtag",
            Node::Unknown { node_type, .. } => node_type,
        }
    }

    /// Visits this node and every descendant, depth first.
    pub fn walk<'a>(&'a self, visit: &mut dyn FnMut(&'a Node)) {
        visit(self);
        for child in self.children() {
            child.walk(visit);
        }
    }
}

/// Parses an AST JSON string (as returned by `flatmarkdown::body_to_ast`).
/// The root is normally a [`Node::Document`].
pub fn parse(json: &str) -> Result<Node, serde_json::Error> {
    serde_json::from_str(json)
}

/// Concatenates the `text` values of a node and its descendants, in document order.
pub fn extract_text(node: &Node) -> String {
    fn collect(node: &Node, out: &mut String) {
        if let Node::Text { value } = node {
            out.push_str(value);
        } else {
            for child in node.children() {
                collect(child, out);
            }
        }
    }
    let mut out = String::new();
    collect(node, &mut out);
    out
}

/// The level of the first top-level heading among the root's children, if any.
pub fn heading_level(root: &Node) -> Option<u8> {
    root.children().iter().find_map(|child| match child {
        Node::Heading { level, .. } => Some(*level),
        _ => None,
    })
}

// Run the code example in the README as a doctest, so that it keeps working
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
