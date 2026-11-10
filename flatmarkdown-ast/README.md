# flatmarkdown-ast

The Flat Markdown AST as Rust types.

The [flatmarkdown](https://crates.io/crates/flatmarkdown) crate parses the body of an item, or a whole page, into these types.
This crate also writes the types as JSON with serde, and reads the JSON back.
It has no Markdown parser, so a program that only reads the AST stays small.

The AST is defined in [SPEC_AST.md](https://github.com/petajournal/flatmarkdown/blob/main/SPEC_AST.md).
The types hold what the specification defines, and nothing more.
The same AST as TypeScript types is the npm package [flatmarkdown-ast](https://www.npmjs.com/package/flatmarkdown-ast). Its source is the [flatmarkdown-ast-ts](https://github.com/petajournal/flatmarkdown-ast-ts) repository.

## Usage

```rust
use flatmarkdown_ast::{Node, extract_text};

let json = r#"{"type":"document","children":[
    {"type":"paragraph","children":[{"type":"text","value":"Hello"}]}
]}"#;

let root = Node::from_json(json).unwrap();
assert_eq!(extract_text(&root), "Hello");

for child in root.children() {
    if let Node::Paragraph { children } = child {
        assert_eq!(children.len(), 1);
    }
}
```

- `Node::from_json` reads the JSON of a node. The node is normally the `Node::Document` of one item body.
- `Node::children` gives the child nodes of any node. A leaf has none.
- `Node::walk` visits a node and all its descendants.
- `extract_text` joins the text of a node and its descendants.
- `Page::from_json` reads the JSON of a whole page into a `Page`. Each `Item` of the page has `level`, `id`, `properties`, `body`, and `ast`.

## Writing

`Node::to_json` and `Page::to_json` write a `Node` or a `Page` as JSON.
The JSON follows the specification: a node without children has no `children`, and an `embed` without properties has no `props`.
`Node::from_json` and `Page::from_json` read that JSON back.

The functions `parse` and `parse_page` are the old names of `Node::from_json` and `Page::from_json`.
They still work, but they are deprecated.

## Unknown Nodes

Reading is tolerant.

- Unknown fields are ignored.
- A missing scalar field gets its default value.
- A node with an unknown `type` becomes `Node::Unknown`, and it keeps its type name and its children.
  It is written back with that type name, so nothing is lost on the way through.

So an old version of this crate can still read the AST of a newer parser.

## Versions

This crate, `flatmarkdown`, and `flatmarkdown-ast2html` always have the same version.
The AST made by `flatmarkdown` x.y.z can be read by `flatmarkdown-ast` x.y.z.
Use the same version of all three.

`Node` is not `#[non_exhaustive]`.
A `match` that lists every kind of node stops compiling when a new kind is added.
This is on purpose: the compiler tells you about a node that you do not handle yet.
A new kind of node is therefore a breaking change, and the version goes up, for example from 0.4 to 0.5.

## License

MIT
