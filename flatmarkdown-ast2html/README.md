# flatmarkdown-ast2html

Renders the Flat Markdown AST to HTML.

The AST is the `Node` of the [flatmarkdown-ast](https://crates.io/crates/flatmarkdown-ast) crate.
This crate has no Markdown parser, so a program that already has the AST stays small.
The [flatmarkdown](https://crates.io/crates/flatmarkdown) crate makes the AST from Markdown.

The AST is defined in [SPEC_AST.md](https://github.com/petajournal/flatmarkdown/blob/main/SPEC_AST.md).

## Usage

From the JSON of an AST, for example one that is stored in a database:

```rust
use flatmarkdown_ast::Node;
use flatmarkdown_ast2html::ast_to_html;

let json = r#"{"type":"document","children":[
    {"type":"paragraph","children":[{"type":"text","value":"Hello"}]}
]}"#;

let ast = Node::from_json(json).unwrap();
assert_eq!(ast_to_html(&ast), "<p>Hello</p>\n");
```

From Markdown, with the `flatmarkdown` crate:

```rust
use flatmarkdown::parse_body;
use flatmarkdown_ast2html::ast_to_html;

let html = ast_to_html(&parse_body("Hello **world**"));
assert_eq!(html, "<p>Hello <strong>world</strong></p>\n");
```

`ast_to_html` takes any node. The node is normally the `Node::Document` of an item body.

## HTML output

The HTML follows comrak where comrak has a form. These are the choices of this crate:

- A task item is a checkbox. Any character in the brackets counts as checked: `[x]` and `[-]` are both checked.
- A table cell has the `align` attribute of its column (`left`, `center`, or `right`), as in GFM. A column without an alignment has none.
- An `embed` is an `<img>` with `src`, `alt`, and, when it has one, `title`. The `props` of the URL fragment (`#w=100`) are not written: how an app uses them is up to the app.
- A `subtext` (`-# text`) is `<p><sub>text</sub></p>`, as in comrak.
- A hashtag is `<a href="<name>" class="hashtag">#<name></a>`. A wikilink is `<a href="<url>" data-wikilink="true">`.
- A node with an unknown `type` writes no tag of its own. Its children are rendered.

A footnote shows its name: `[^memo]` is shown as `[memo]`. An inline footnote (`^[content]`) has no name of its own, so it shows its number: `[1]` for the first one of the body.

A reference has the `id` `fnref-<name>`, and a definition has the `id` `fn-<name>`. The second reference to the same footnote has the `id` `fnref-<name>-2`, the third `fnref-<name>-3`, and so on. The label of the definition links back to the first reference.

## HTML output is for trusted content

`ast_to_html` is made for content that you trust, such as your own notes.

- Raw HTML in the AST is written as it is. `<script>` and event attributes such as `onerror` are not removed.
- The URL of a link is not checked. A `javascript:` link stays a link.

This is on purpose. Flat Markdown lets you write HTML in a note.

Do not show the HTML of an AST that you do not trust without a sanitizer.

Values that are text are escaped: the text itself, the info of a code fence, the title of an alert, the name of a hashtag, and the `alt` of an image.
The `alt` of an image holds only the text of the image description, without tags.

## Versions

This crate, `flatmarkdown`, and `flatmarkdown-ast` always have the same version.
Use the same version of all three.

## License

MIT