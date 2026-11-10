//! Typed serde model of the Flat Markdown AST.
//!
//! The AST is defined in `SPEC_AST.md` of the flatmarkdown repository, and the types hold
//! what that specification defines. The `flatmarkdown` crate builds these types
//! (`parse_body*` and `page_to_ast*`). In the JSON, every node is an object discriminated by
//! its `type` field (snake_case), with a `children` array when it has children.
//!
//! Deserialization is tolerant: unknown fields are ignored, scalar fields the producer
//! omits fall back to their defaults, and a node whose `type` is not modelled here becomes
//! [`Node::Unknown`] (keeping its type name and its children) instead of failing the whole
//! parse.

use std::collections::BTreeMap;

use serde::de::Error as _;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// One node of the Flat Markdown AST.
///
/// Variants follow the node types of `SPEC_AST.md`. Nodes that may contain other nodes carry
/// `children`; leaf nodes do not. An empty `children` is not written to the JSON.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", remote = "Self")]
pub enum Node {
    // ── Block nodes ──
    Document {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Paragraph {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Heading {
        #[serde(default)]
        level: u8,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    CodeBlock {
        /// True for a block written with fences, false for one written with an indent.
        ///
        /// An AST written before this attribute existed has no `fenced`. It reads as true: the
        /// earlier specification allowed only fenced blocks.
        #[serde(default = "default_fenced")]
        fenced: bool,
        #[serde(default)]
        info: String,
        #[serde(default)]
        literal: String,
    },
    BlockQuote {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    MultilineBlockQuote {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    List {
        /// `"bullet"` or `"ordered"`.
        #[serde(default)]
        list_type: String,
        #[serde(default)]
        start: u32,
        #[serde(default)]
        tight: bool,
        /// `"period"` or `"paren"`.
        #[serde(default)]
        delimiter: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Item {
        #[serde(default)]
        list_type: String,
        #[serde(default)]
        start: u32,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    TaskItem {
        /// The character inside the brackets (`"x"`), or `None` for an unchecked `[ ]`.
        #[serde(default)]
        symbol: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    TableRow {
        #[serde(default)]
        header: bool,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    TableCell {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Alert {
        #[serde(default)]
        alert_type: String,
        #[serde(default)]
        title: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Subtext {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Strong {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Strikethrough {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Underline {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Highlight {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Superscript {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Subscript {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    SpoileredText {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    /// Every Markdown image (`![alt](url#k=v)`): a media embed with the URL fragment split
    /// into `props`. Empty `props` are not written to the JSON.
    Embed {
        #[serde(default)]
        url: String,
        #[serde(default)]
        title: String,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        props: BTreeMap<String, Value>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
        literal: String,
    },
    HtmlInline {
        #[serde(default)]
        value: String,
    },
    Wikilink {
        #[serde(default)]
        url: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
    Hashtag {
        #[serde(default)]
        value: String,
    },

    /// A node whose `type` this crate does not model. Its children are still parsed.
    /// It is written back with its own `type`, so that nothing is lost on the way through.
    Unknown {
        /// The `type` as it was read. Not a serde field: the derived form would write `unknown`
        /// as the tag, so `Serialize` writes this node by hand.
        #[serde(skip)]
        node_type: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<Node>,
    },
}

/// The value of `fenced` for a code block that does not have the attribute.
fn default_fenced() -> bool {
    true
}

impl Serialize for Node {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Node::Unknown {
                node_type,
                children,
            } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", node_type)?;
                if !children.is_empty() {
                    map.serialize_entry("children", children)?;
                }
                map.end()
            }
            _ => Node::serialize(self, serializer),
        }
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
            | Node::Alert { children, .. }
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
            | Node::Embed { children, .. }
            | Node::Wikilink { children, .. }
            | Node::Unknown { children, .. } => children,
            Node::CodeBlock { .. }
            | Node::ThematicBreak
            | Node::HtmlBlock { .. }
            | Node::Text { .. }
            | Node::Softbreak
            | Node::Linebreak
            | Node::Code { .. }
            | Node::FootnoteReference { .. }
            | Node::Shortcode { .. }
            | Node::Math { .. }
            | Node::HtmlInline { .. }
            | Node::Hashtag { .. } => &[],
        }
    }

    /// The child nodes, to change them. `None` for a leaf node, which cannot have children.
    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
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
            | Node::Alert { children, .. }
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
            | Node::Embed { children, .. }
            | Node::Wikilink { children, .. }
            | Node::Unknown { children, .. } => Some(children),
            Node::CodeBlock { .. }
            | Node::ThematicBreak
            | Node::HtmlBlock { .. }
            | Node::Text { .. }
            | Node::Softbreak
            | Node::Linebreak
            | Node::Code { .. }
            | Node::FootnoteReference { .. }
            | Node::Shortcode { .. }
            | Node::Math { .. }
            | Node::HtmlInline { .. }
            | Node::Hashtag { .. } => None,
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
            Node::Alert { .. } => "alert",
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
            Node::Embed { .. } => "embed",
            Node::FootnoteReference { .. } => "footnote_reference",
            Node::Shortcode { .. } => "shortcode",
            Node::Math { .. } => "math",
            Node::HtmlInline { .. } => "html_inline",
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

    /// Writes this node and its descendants as the JSON of `SPEC_AST.md`.
    /// [`Node::from_json`] reads it back.
    pub fn to_json(&self) -> String {
        // Every key of the JSON is a string, so serde_json has nothing to refuse.
        serde_json::to_string(self).expect("a node is always valid JSON")
    }

    /// Reads a node and its descendants from the JSON of `SPEC_AST.md`
    /// (as written by [`Node::to_json`]). The root is normally a [`Node::Document`].
    pub fn from_json(json: &str) -> Result<Node, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// A whole page, as `flatmarkdown::page_to_ast` returns it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename = "page")]
pub struct Page {
    /// The raw YAML front matter with its `---` delimiters, or `None` when the page has none.
    pub front_matter: Option<String>,
    /// The outline items in source order.
    pub items: Vec<Item>,
}

/// One outline item of a page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// The indent level. `1` is the shallowest level.
    pub level: u32,
    /// The HMTID of the metadata line, or `None` when the item has none.
    #[serde(default)]
    pub id: Option<String>,
    /// The `key="value"` pairs of the metadata line.
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
    /// The Markdown source of the item.
    pub body: String,
    /// The AST of `body`. The root is normally a [`Node::Document`].
    pub ast: Node,
}

impl Page {
    /// Writes this page as the JSON of `SPEC_AST.md`. [`Page::from_json`] reads it back.
    pub fn to_json(&self) -> String {
        // Every key of the JSON is a string, so serde_json has nothing to refuse.
        serde_json::to_string(self).expect("a page is always valid JSON")
    }

    /// Reads a whole page from the JSON of `SPEC_AST.md` (as written by [`Page::to_json`]).
    pub fn from_json(json: &str) -> Result<Page, serde_json::Error> {
        serde_json::from_str(json)
    }
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
