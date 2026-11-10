# flatmarkdown

Flat Markdown parser — outline format with block-delimited items.

Based on the Flat Markdown specification v0.2: https://github.com/petajournal/flatmarkdown/blob/main/SPEC.md

## API

### Options

```rust
#[derive(Default)]
pub enum LinkResolution {
    #[default]
    Off,
    On { prefixes: Vec<String> },
}

#[derive(Default)]
pub struct BodyOptions {
    pub link_resolution: LinkResolution,
}
```

`link_resolution` (default: `Off`) — when `On`, enables link and hashtag resolution:

| Input | AST node |
|---|---|
| `[label](<prefix>page.md)`, or with `#id` | `wikilink` |
| `[#tag](<prefix>tag.md)` | `hashtag` |
| `#tag` in text | `hashtag` |

Relative links are matched when the URL starts with one of `prefixes` (for example `../` or `./`) and ends with `.md`. When more than one prefix matches, the longest one is used.

When `Off`, these links remain as `link` nodes and `#tag` text remains as plain text.

### Functions

| Function | Description |
|---|---|
| `page_to_ast(input)` | Split a page into items with metadata, as a `Page` ([spec](https://github.com/petajournal/flatmarkdown/blob/main/SPEC_AST.md)) |
| `page_to_ast_with(input, opts)` | Same, with explicit `BodyOptions` |
| `body_to_ast(input)` | Parse an item body to the AST, as a `Node` ([spec](https://github.com/petajournal/flatmarkdown/blob/main/SPEC_AST.md)) |
| `body_to_ast_with(input, opts)` | Same, with explicit `BodyOptions` |

### JSON and HTML

The functions return the types of the [flatmarkdown-ast](https://crates.io/crates/flatmarkdown-ast) crate: a `Node` for an item body, and a `Page` for a page.

- `to_json()` of a `Node` or a `Page` writes the JSON of the specification.
- `ast_to_html` of the [flatmarkdown-ast2html](https://crates.io/crates/flatmarkdown-ast2html) crate renders a `Node` to HTML. This crate does not make HTML.

### Examples

```rust
use flatmarkdown::{body_to_ast, body_to_ast_with, page_to_ast, BodyOptions, LinkResolution};

// Split a page into items. Each top-level block is one item.
// A `<!-- @@ ID key="value" -->` line above a block is its metadata.
let page = page_to_ast("<!-- @@ 2026-09-27-15-00-16-W3N8ASY plan=\"2026-09-28\" -->\n- [ ] Task1\n\nParagraph");
assert_eq!(page.items.len(), 2);

// Parse one item body (link resolution off)
let ast = body_to_ast("Hello **world**");
let json = ast.to_json();

// With link resolution enabled
let opts = BodyOptions {
    link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] },
};
let ast = body_to_ast_with("#tag text", &opts);
```

