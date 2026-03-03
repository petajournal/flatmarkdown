# flatmarkdown

Flat Markdown parser — outline format with blank-line-delimited blocks

## API

- `markdown_to_html(input: &str) -> String` — Convert Markdown to HTML
- `markdown_to_ast(input: &str) -> String (JSON)` — Convert Markdown to JSON AST ([spec](SPEC_AST.md))

## Build

```sh
cargo clean
cargo build
cargo test
```
