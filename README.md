# flatmarkdown

Flat Markdown parser — outline format with blank-line-delimited blocks

## API

### Options

```rust
#[derive(Default)]
pub struct BodyOptions {
    pub resolve_links: bool,
}
```

`resolve_links` (default: `false`) — when `true`, converts `../page.md` links to `wikilink` nodes,
`[#tag](../tag.md)` links to `hashtag` nodes, and extracts `#tag` text patterns as `hashtag` nodes.

### Functions

| Function | Description |
|---|---|
| `body_to_ast(input)` | Parse body to JSON AST ([spec](SPEC_AST.md)) |
| `body_to_ast_with(input, opts)` | Same, with explicit `BodyOptions` |
| `body_to_html(input)` | Render body to HTML |
| `body_to_html_with(input, opts)` | Same, with explicit `BodyOptions` |

### Examples

```rust
// Default (resolve_links = false)
let ast = body_to_ast("Hello **world**");
let html = body_to_html("Hello **world**");

// With link resolution enabled
let opts = BodyOptions { resolve_links: true };
let ast = body_to_ast_with("[[page]]", &opts);
let html = body_to_html_with("#tag text", &opts);
```

## Build

```sh
cargo clean
cargo build
cargo test
```
