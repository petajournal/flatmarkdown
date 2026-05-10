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

`resolve_links` (default: `false`) — when `true`, enables link and hashtag resolution:

| Input | AST node | HTML output |
|---|---|---|
| `[label](../page.md)`, `[label](./page.md)`, or with `#id` | `wikilink` | `<a href="page" data-wikilink="true">label</a>` |
| `[#tag](../tag.md)` or `[#tag](./tag.md)` | `hashtag` | `<a href="tag" class="hashtag">#tag</a>` |
| `#tag` in text | `hashtag` | `<a href="tag" class="hashtag">#tag</a>` |

Relative links are matched when the URL starts with `../` or `./` and ends with `.md`. When `false` (the default), these links remain as `link` nodes and `#tag` text remains as plain text.

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
