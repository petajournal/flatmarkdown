# flatmarkdown AST Specification

The basic specification is based on the Flat Markdown specification v0.2: https://github.com/petajournal/flatmarkdown-spec/

`page_to_ast(input)` parses a whole Flat Markdown page into a JSON string. `body_to_ast(input)` parses the body of one item into a JSON AST string.

## Page Structure

`page_to_ast(input)` returns the following object:

```json
{
  "type": "page",
  "front_matter": "---\nkey: value\n---\n\n",
  "items": [
    {
      "level": 1,
      "id": "2026-09-26-18-14-47-32W15WT",
      "properties": { "plan": "2026-09-28" },
      "body": "- [ ] Task1",
      "ast": { "type": "document", "children": [ ... ] }
    }
  ]
}
```

| Field          | Type           | Description |
|----------------|----------------|-------------|
| `front_matter` | string \| null | The raw YAML front matter at the top of the page, including the `---` delimiters. `null` when absent. |
| `items`        | array          | The outline items in source order. |

Each item has:

| Field        | Type           | Description |
|--------------|----------------|-------------|
| `level`      | integer        | Indent level. `1` is the shallowest level. The number of `@` characters in the metadata line. `1` when the metadata line is omitted. |
| `id`         | string \| null | The HMTID from the metadata line. `null` when absent. |
| `properties` | object         | `key: value` pairs from the metadata line. Values are always strings. Empty when absent. |
| `body`       | string         | The item's Markdown source. Leading and trailing blank lines are removed. |
| `ast`        | object         | The result of parsing `body` with `body_to_ast`. |

**Item delimiters**: every block at the shallowest level of the Markdown document is one item. Nested blocks belong to the body of the outer block. Block boundaries are decided by the Markdown parser, not by blank lines.

**Metadata line**: a single-line HTML comment of the form `<!-- @@ ID key1="value1" key2="value2" -->` at the shallowest level is the metadata of the next block. It starts at column 1 with `<!-- @` and ends with `-->`. Blank lines between the metadata line and its block are ignored. A metadata line that is followed only by blank lines (until the next metadata line or the end of the page) is an item with an empty `body`. Comments that do not follow this syntax exactly are ordinary body text.

**Lines without a node**: lines that the Markdown parser consumes without producing a block (link reference definitions, unreferenced footnote definitions) are kept. They are appended to the body of the previous item, because such definitions are normally written after the paragraph that uses them. Lines between a metadata line and its block are prepended to that item's body. Lines at the start of the page are prepended to the first block, or form their own item when a metadata line follows them.

## Body Preprocessing

**Markdown link → wikilink / hashtag conversion** *(LinkResolution::On only)*: After parsing, `link` nodes whose URL starts with one of the configured prefixes and ends with `.md` are converted as follows. The matching prefix and `.md` suffix are stripped to form the page path.

- If the link text is a single text node whose value equals `#page_path` (i.e. `[#tagname](<prefix>tagname.md)`), the node becomes a `hashtag` node with `value = tagname`. The `url`, `title`, and `children` fields are removed.
- Otherwise, the node becomes a `wikilink` node. The `url` is the page path (prefix and `.md` suffix stripped); a `#id` fragment is preserved. The `title` field is removed; `children` (link text) are kept as the label.

Links that do not match any prefix (external URLs, non-`.md` paths, or paths with non-matching prefixes) are kept as `link` nodes.

When *LinkResolution::Off* (the default), this conversion is skipped entirely, as is the `#tag` text-node extraction described below.

## Node Structure

Every node is a JSON object with at least a `type` field. Nodes with child elements include a `children` array. Additional attributes are flattened into the same object.

```json
{
  "type": "<node_type>",
  "<attr>": "<value>",
  "children": [ ... ]
}
```

- `type` (string) — always present
- `children` (array) — present only when the node has one or more child nodes

## Node Types

### Block Nodes

#### `document`

Root node. Always the top-level node of the AST.

#### `paragraph`

A paragraph block. Children are inline nodes.

#### `heading`

| Attribute | Type    | Description        |
|-----------|---------|--------------------|
| `level`   | integer | Heading level, 1–6 |

#### `code_block`

A fenced code block. Has no children; content is in `literal`. Flatmarkdown does not support indented code blocks.

| Attribute | Type    | Description                                     |
|-----------|---------|-------------------------------------------------|
| `info`    | string  | Info string after opening fence (e.g. `"rust"`) |
| `literal` | string  | The code content                                |

Note: when no info string is specified, `info` defaults to `"text"` (configured via `default_info_string`).

#### `block_quote`

A `>` blockquote. Children are block nodes.

#### `multiline_block_quote`

A blockquote fenced by `>>>` lines. Children are block nodes.

#### `front_matter`

YAML front matter. Only produced when parsing a whole page with `page_to_ast`. A body that starts with `---` is a thematic break, not front matter.

| Attribute | Type   | Description |
|-----------|--------|-------------|
| `value`   | string | The raw front matter including the `---` delimiters |

#### `list`

| Attribute   | Type    | Description                           |
|-------------|---------|---------------------------------------|
| `list_type` | string  | `"bullet"` or `"ordered"`             |
| `start`     | integer | Starting number (ordered lists)       |
| `tight`     | boolean | `true` if tight (no `<p>` wrapping)   |
| `delimiter` | string  | `"period"` (`.`) or `"paren"` (`)`)   |

#### `item`

A list item.

| Attribute   | Type    | Description                         |
|-------------|---------|-------------------------------------|
| `list_type` | string  | `"bullet"` or `"ordered"`           |
| `start`     | integer | Ordinal of this item                |
| `tight`     | boolean | `true` if parent list is tight      |

#### `task_item`

A task list item (checkbox).

| Attribute | Type           | Description                                      |
|-----------|----------------|--------------------------------------------------|
| `symbol`  | string \| null | The character in brackets (e.g. `"x"`), or `null` if unchecked |

#### `table`

| Attribute      | Type     | Description                                        |
|----------------|----------|----------------------------------------------------|
| `alignments`   | string[] | Per-column alignment: `"none"`, `"left"`, `"center"`, `"right"` |
| `num_columns`  | integer  | Number of columns                                  |
| `num_rows`     | integer  | Number of rows                                     |

#### `table_row`

| Attribute | Type    | Description                    |
|-----------|---------|--------------------------------|
| `header`  | boolean | `true` if this is the header row |

#### `table_cell`

A single table cell. Children are inline nodes.

#### `thematic_break`

A horizontal rule (`---`, `***`, `___`). No attributes, no children.

#### `html_block`

Raw HTML block.

| Attribute    | Type    | Description          |
|--------------|---------|----------------------|
| `block_type` | integer | HTML block type (1–7) |
| `literal`    | string  | Raw HTML content     |

#### `footnote_definition`

| Attribute | Type   | Description     |
|-----------|--------|-----------------|
| `name`    | string | Footnote label  |

#### `alert`

GitHub-style alert (e.g. `> [!NOTE]`).

| Attribute    | Type           | Description                                          |
|--------------|----------------|------------------------------------------------------|
| `alert_type` | string         | `"note"`, `"tip"`, `"important"`, `"warning"`, `"caution"` |
| `title`      | string \| null | Custom title, or `null` for the default              |

#### `subtext`

Block-level subscript text (`<sub>` block).

### Inline Nodes

#### `text`

Literal text content.

| Attribute | Type   | Description |
|-----------|--------|-------------|
| `value`   | string | Text content |

#### `softbreak`

A soft line break: a plain newline inside a paragraph. Line breaks follow standard GFM.

#### `linebreak`

A hard line break: GFM hard line break syntax (trailing two or more spaces, or a backslash before the newline).

#### `emph`

Emphasis (`*text*` or `_text_`). Children are inline nodes.

#### `strong`

Strong emphasis (`**text**`). Children are inline nodes.

#### `strikethrough`

Strikethrough (`~~text~~`). Children are inline nodes.

#### `underline`

Underline (`__text__`). Children are inline nodes.

#### `highlight`

Highlight (`==text==`). Children are inline nodes.

#### `superscript`

Superscript (`^text^`). Children are inline nodes.

#### `subscript`

Subscript (`~text~`). Children are inline nodes.

#### `spoilered_text`

Spoiler (`||text||`). Children are inline nodes.

#### `code`

Inline code span.

| Attribute | Type   | Description    |
|-----------|--------|----------------|
| `literal` | string | Code content   |

#### `link`

| Attribute | Type   | Description      |
|-----------|--------|------------------|
| `url`     | string | Link destination |
| `title`   | string | Link title       |

Children are the link text (inline nodes).

#### `embed`

All Markdown image syntax (`![alt](url)`) is interpreted as a media embed (image, video, audio, PDF, etc.).

| Attribute | Type   | Description  |
|-----------|--------|--------------|
| `url`     | string | Media source URL (without the `#` fragment) |
| `title`   | string | Title |
| `props`   | object | Key-value properties parsed from the URL fragment (`#key=value&flag`). Boolean flags without `=` are represented as `true`. Omitted when no properties are present. |

Children are the alt text (inline nodes).

#### `wikilink`

A wikilink. Children are the label (inline nodes); when no label is given, the children contain the URL as a `text` node.

| Syntax | Description |
|--------|-------------|
| `[[page]]` | Link to a page |
| `[[page\|label]]` | Link to a page with a display label |
| `[[page#id]]` | Link to a block within a page (`url` contains the `#id` fragment) |
| `[[page#id\|label]]` | Link to a block with a display label |

| Attribute | Type   | Description     |
|-----------|--------|-----------------|
| `url`     | string | Link destination. May contain a `#id` fragment for block links. |

#### `footnote_reference`

| Attribute | Type    | Description                           |
|-----------|---------|---------------------------------------|
| `name`    | string  | Footnote label                        |
| `ref_num` | integer | Index of this reference to the same footnote |
| `ix`      | integer | Index of the footnote in the document |

#### `shortcode`

Emoji shortcode (e.g. `:rabbit:` → 🐰).

| Attribute | Type   | Description                    |
|-----------|--------|--------------------------------|
| `code`    | string | Shortcode name (e.g. `"rabbit"`) |
| `emoji`   | string | Resolved emoji (e.g. `"🐰"`)    |

#### `math`

Inline math using code-style syntax: `` $`formula`$ `` (inline) or `` $$`formula`$$ `` (display). Block math via `math_dollars` (`$$`) is disabled.

| Attribute | Type   | Description  |
|-----------|--------|--------------|
| `literal` | string | Math content |

#### `html_inline`

Raw inline HTML.

| Attribute | Type   | Description      |
|-----------|--------|------------------|
| `value`   | string | Raw HTML content |

#### `raw`

Verbatim output content.

| Attribute | Type   | Description |
|-----------|--------|-------------|
| `value`   | string | Raw content |

#### `escaped`

An escaped character.

#### `escaped_tag`

An escaped HTML tag (from tagfilter).

| Attribute | Type   | Description    |
|-----------|--------|----------------|
| `value`   | string | The tag name   |

#### `hashtag`

A hashtag (e.g. `#tag`, `#diary`). Extracted from `text` nodes in AST post-processing.

`#` must appear at the start of a string or immediately after a half-width space (U+0020). A `#` preceded by any other character is not recognized as a tag. The tag name consists of Unicode alphanumerics, `_`, `-`, and `/`; any other character (punctuation, whitespace, end of string, etc.) terminates the tag. `/` is not allowed as the first or last character of the tag name.

`\#` is treated as a literal `#` and is never recognized as a hashtag.

| Attribute | Type   | Description                                    |
|-----------|--------|------------------------------------------------|
| `value`   | string | Tag name without the leading `#` (e.g. `"diary"`) |

## Example

Input:

```markdown
## Hello **world**
```

Output:

```json
{
  "type": "document",
  "children": [
    {
      "type": "heading",
      "level": 2,
      "children": [
        { "type": "text", "value": "Hello " },
        {
          "type": "strong",
          "children": [
            { "type": "text", "value": "world" }
          ]
        }
      ]
    }
  ]
}
```
