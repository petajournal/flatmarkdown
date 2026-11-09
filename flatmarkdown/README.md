# flatmarkdown

Flat Markdown parser — outline format with block-delimited items.

Based on the Flat Markdown specification v0.2: https://github.com/petajournal/flatmarkdown-spec/

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

| Input | AST node | HTML output |
|---|---|---|
| `[label](<prefix>page.md)`, or with `#id` | `wikilink` | `<a href="page" data-wikilink="true">label</a>` |
| `[#tag](<prefix>tag.md)` | `hashtag` | `<a href="tag" class="hashtag">#tag</a>` |
| `#tag` in text | `hashtag` | `<a href="tag" class="hashtag">#tag</a>` |

Relative links are matched when the URL starts with one of `prefixes` (for example `../` or `./`) and ends with `.md`. When more than one prefix matches, the longest one is used. When `Off`, these links remain as `link` nodes and `#tag` text remains as plain text.

### Functions

| Function | Description |
|---|---|
| `page_to_ast(input)` | Split a page into items with metadata ([spec](https://github.com/petajournal/flatmarkdown/blob/main/SPEC_AST.md)) |
| `page_to_ast_with(input, opts)` | Same, with explicit `BodyOptions` |
| `body_to_ast(input)` | Parse an item body to JSON AST ([spec](https://github.com/petajournal/flatmarkdown/blob/main/SPEC_AST.md)) |
| `body_to_ast_with(input, opts)` | Same, with explicit `BodyOptions` |
| `body_to_html(input)` | Render an item body to HTML |
| `body_to_html_with(input, opts)` | Same, with explicit `BodyOptions` |

### HTML output is for trusted content

`body_to_html` is made for content that you trust, such as your own notes.

- Raw HTML in the body is written as it is. `<script>` and event attributes such as `onerror` are not removed.
- The URL of a link is not checked. A `javascript:` link stays a link.

This is on purpose. Flat Markdown lets you write HTML in a note.

Do not show the HTML of Markdown that you do not trust without a sanitizer.

Values that are text are escaped: the text itself, the info of a code fence, the title of an alert, the name of a hashtag, and the `alt` of an image.
The `alt` of an image holds only the text of the image description, without tags.

### Examples

```rust
use flatmarkdown::{body_to_ast, body_to_html_with, page_to_ast, BodyOptions, LinkResolution};

// Split a page into items. Each top-level block is one item.
// A `<!-- @@ ID key="value" -->` line above a block is its metadata.
let page = page_to_ast("<!-- @@ 2026-09-27-15-00-16-W3N8ASY plan=\"2026-09-28\" -->\n- [ ] Task1\n\nParagraph");

// Parse one item body (link resolution off)
let ast = body_to_ast("Hello **world**");

// With link resolution enabled
let opts = BodyOptions {
    link_resolution: LinkResolution::On { prefixes: vec!["../".into(), "./".into()] },
};
let html = body_to_html_with("#tag text", &opts);
```

