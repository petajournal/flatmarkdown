# flatmarkdown AST Specification

`markdown_to_ast(input)` parses Markdown into a JSON AST string.

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

#### `linebreak`

A hard line break (trailing \ or two spaces). In addition, in Flatmarkdown, newline characters are also always treated as a linebreak.

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

#### `image`

| Attribute | Type   | Description  |
|-----------|--------|--------------|
| `url`     | string | Image source |
| `title`   | string | Image title  |

Children are the alt text (inline nodes).

#### `wikilink`

A wikilink (`[[page]]` or `[[url|label]]`). Children are the label (inline nodes); when no label is given, the children contain the URL as a `text` node.

| Attribute | Type   | Description     |
|-----------|--------|-----------------|
| `url`     | string | Link destination |

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

Code-style math (`` ```math ``). `math_dollars` is disabled.

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

`#` must appear at the start of a string or immediately after whitespace (a `#` in the middle of a word is not recognized as a tag). The tag name consists of Unicode alphanumerics, `_`, `-`, and `/`; any other character (punctuation, whitespace, end of string, etc.) terminates the tag. `/` is not allowed as the first or last character of the tag name.

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
      "setext": false,
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
