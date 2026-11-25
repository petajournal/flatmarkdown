# flatmarkdown-ast2html

Renders the Flat Markdown AST to HTML.

The AST is the `Node` of the [flatmarkdown-ast](https://crates.io/crates/flatmarkdown-ast) crate.
This crate has no Markdown parser, so a program that already has the AST stays small.
The [flatmarkdown](https://crates.io/crates/flatmarkdown) crate makes the AST from Markdown.

The AST is defined in [SPEC_AST.md](https://github.com/sosuisen/flatmarkdown/blob/main/SPEC_AST.md).

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

### Where links lead

`ast_to_html_with` takes `HtmlOptions`. They hold templates for the `href` of a wikilink and of a hashtag.
Every `{}` in a template is replaced by the name of the page or by the tag name:

```rust
use flatmarkdown_ast::Node;
use flatmarkdown_ast2html::{HtmlOptions, ast_to_html_with};

let options = HtmlOptions {
    hashtag_href: "/tags/{}.html".to_string(),
    ..HtmlOptions::default()
};
let tag = Node::Hashtag { value: "Rust".to_string() };
assert_eq!(
    ast_to_html_with(&tag, &options),
    "<a href=\"/tags/Rust.html\" class=\"hashtag\">#Rust</a>"
);
```

Both templates are `{}` by default, so `ast_to_html` is `ast_to_html_with` with `HtmlOptions::default()`.

The name is percent-encoded before it goes into the template. The text of the template is not.
Letters, digits, and `-_.!~*'();/:@&=+$,` stay as they are.
Every other byte of the UTF-8 name becomes `%XX`, with capital hex letters.
These are the characters that `encodeURI` of JavaScript encodes, and also `?` and `#`.
So `What?` becomes `What%3F`, `C#` becomes `C%23`, and `日記` becomes `%E6%97%A5%E8%A8%98`.
`/` stays, because it separates the folders in the name of a page.
The `href` is then escaped for HTML.

### The fragment of a wikilink

A `#` in a wikilink can be a part of the name, as in `[[C#]]`.
It can also start a place in the page, as in `[[Page#20260426035700_QZ998PX]]`.
`wikilink_fragment_pattern` tells the two apart. It is a regular expression for the text after the last `#`.

When the pattern matches the whole of that text, the text is the fragment, and the text before the `#` is the name.
The fragment goes after the filled template as `#<fragment>`, percent-encoded as a name is.
Otherwise the whole URL of the wikilink is the name.
The pattern is empty by default, so every `#` is a part of the name.

| Wikilink | `href` with the template `/pages/{}` and the pattern `[0-9]{14}[-_][0-9A-Z]{7}` |
|---|---|
| `[[C#]]` | `/pages/C%23` |
| `[[Page#20260426035700_QZ998PX]]` | `/pages/Page#20260426035700_QZ998PX` |
| `[[C##20260426035700_QZ998PX]]` | `/pages/C%23#20260426035700_QZ998PX` |
| `[[Page#item]]` | `/pages/Page%23item` |

The pattern is read by [regex-lite](https://crates.io/crates/regex-lite).
Write a pattern that `RegExp` of JavaScript reads in the same way, because the npm renderers read the same pattern.
Do not use look-around or back references.
A pattern that is not a regular expression matches nothing.

### URLs without rendering

An app can make the same URL as a rendered link, for example to link to a page from its own code.

- `href_template_from_prefix(prefix)` makes a template from a prefix. `/pages/` gives `/pages/{}`. The prefix is percent-encoded as a name is.
- `fill_href(template, name)` puts the percent-encoded name into the template. The result is a URL, not escaped for HTML.
- `split_wikilink(url, fragment_pattern)` splits the URL of a wikilink into the name and the fragment, by the rule above.

```rust
use flatmarkdown_ast2html::{fill_href, href_template_from_prefix, split_wikilink};

let template = href_template_from_prefix("/pages/");
assert_eq!(fill_href(&template, "What?"), "/pages/What%3F");
assert_eq!(split_wikilink("C##1", "[0-9]+"), ("C#", Some("1")));
```

### The table of hrefs

`tests/hrefs.json` lists names, templates, patterns, and the hrefs that they make.
The tests of this crate and of the npm package read it.
[flatmarkdown-ast2react](https://www.npmjs.com/package/flatmarkdown-ast2react) keeps a copy for its own tests, so that its TypeScript gives the same hrefs.
When the rule changes, change the table first.

## HTML output

The HTML follows comrak where comrak has a form. These are the choices of this crate:

- A task item is a checkbox. Any character in the brackets counts as checked: `[x]` and `[-]` are both checked.
- A table cell has the `align` attribute of its column (`left`, `center`, or `right`), as in GFM. A column without an alignment has none.
- An `embed` is an `<img>` with `src`, `alt`, and, when it has one, `title`. The `props` of the URL fragment (`#w=100`) are not written: how an app uses them is up to the app.
- A `subtext` (`-# text`) is `<p><sub>text</sub></p>`, as in comrak.
- A hashtag is `<a href="<name>" class="hashtag">#<name></a>`. A wikilink is `<a href="<url>" data-wikilink="true">`. The name in the `href` is percent-encoded. `HtmlOptions` can change the `href` of both (see "Where links lead").
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