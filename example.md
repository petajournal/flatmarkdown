# Examples of Extended Syntax

This file explains the comrak extensions and the hashtag listed under "Available Syntax" in SPEC.md.
For each syntax, it shows how to write it and how Flat Markdown splits it into items.

In Flat Markdown, each top-level block becomes one item.
Inline syntax (used inside a sentence) does not affect item boundaries.
Block syntax (alert, subtext, multiline_block_quotes) becomes one item by itself.

## footnote

Write `[^name]` in the text. Define the footnote body with `[^name]: text`.

```markdown
Flat Markdown uses block boundaries[^1].

[^1]: A blank line is not required to separate items.
```

- The footnote reference is inline.
- The footnote definition is treated as a line that produces no block. It is appended to the end of the previous item's body.
- The example above becomes one item. The footnote still resolves when the body is parsed alone.
- The reference and the definition may also be in different items. Both are kept when a body is parsed alone, so a reference in one item links to a definition in a later item, such as a list of definitions at the end of the page.

An inline footnote is written in place, as `^[text]`. It needs no name and no definition line.

```markdown
Flat Markdown uses block boundaries^[A blank line is not required to separate items.].
```

- The text can hold inline markup.
- An inline footnote is inline. The example above is one item.
- An empty `^[]` is plain text.

## alert

Put `[!TYPE]` on the first line of a block quote. The five types are NOTE, TIP, IMPORTANT, WARNING, and CAUTION.

```markdown
> [!NOTE]
> A metadata line must fit on one line.

> [!WARNING]
> A metadata line must not start with whitespace.
```

- An alert is one block. The example above becomes two items.
- Several paragraphs inside one alert still form one item.
- Combined with multiline block quotes (`>>>`, see below), you can omit the leading `>` on each line.

```markdown
>>> [!TIP]
A long alert is easier to read as a multiline block quote.

- Lists work here too.
>>>
```

## subtext

Start a line with `-#` followed by a space. The whole paragraph becomes small supporting text.

```markdown
Paragraph 1

-# Information as of October 2026.
```

- Subtext is block syntax. The example above becomes two items.
- It looks like a heading `#`, but it starts with `-`.

## highlight

Wrap the text in `==`.

```markdown
A tag starts with ==a # that follows a space==.
```

- Inline syntax. It does not affect item boundaries.

## superscript

Wrap the text in `^`.

```markdown
The area is x^2^.
```

- Inline syntax.
- It looks like a footnote `[^1]`, but only the text between `^` marks becomes superscript.
- `^[text]`, with brackets right after the `^`, is an inline footnote and not superscript.

## subscript

Wrap the text in `~`.

```markdown
The formula for water is H~2~O.
```

- Inline syntax.
- It is distinct from GFM strikethrough `~~text~~`. One `~` is subscript. Two are strikethrough.

## spoilered_text

Wrap the text in `||`. The text stays hidden until clicked.

```markdown
The culprit was ||the butler||.
```

- Inline syntax.
- It is easy to confuse with the table cell separator `|`. Avoid it inside tables.
- A single `|` on each side is not a spoiler. `|text|` is plain text.

## shortcode

Write `:name:`. It is replaced with the emoji of the same name as on GitHub.

```markdown
Task completed :tada:
```

- Inline syntax.
- If the name does not exist, the text is left as is.

## math

Wrap the formula in `` $` `` and `` `$ ``. This is the code style of math.

```markdown
The quadratic formula is $`x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}`$.
```

- Inline syntax. It does not affect item boundaries.
- Math is always inline. There is no display math.
- `$x$` and `$$x$$`, without backticks, are not math. They are plain text, so `^` and `~` inside them work as superscript and subscript.
- `^`, `~`, and `\` inside math are not treated as markup. They are part of the math.

## hashtag

Write `#` followed by the tag name. A `#` starts a tag at the start of a line, right after a half-width space, or right after inline markup.

```markdown
#diary Wrote the spec today. #flat-markdown #spec/v0-2
```

- Inline syntax.
- This is not a comrak extension. The Flat Markdown parser finds the tags itself.
- Allowed characters are Unicode letters and digits, plus `-`, `_`, and `/`. Punctuation or whitespace ends the tag. In `#spec/v0.2`, the tag is `spec/v0`, because `.` ends it.
- A `#` right after another character of the text is not a tag. For example, `C#` is not a tag.
- A `#` right after inline markup is a tag, without a space. In `**Note**#todo`, `#todo` is a tag.
- Write `\#` to prevent it from being parsed as a tag.
- The parser makes tags only when link resolution is ON.
- A tag has the same meaning as a wikilink to a page. See "Tags" in SPEC.md for details.

## underline

Wrap the text in `__`.

```markdown
The only __required__ element is `@`.
```

- Inline syntax.
- In standard Markdown, `__text__` is bold. With this extension it becomes underline. Use `**text**` for bold.

## wikilink

Write `[[full path of the page]]`. To add a label, put it after `|`.

```markdown
See [[spec/flatmarkdown]] for details.

With a label: [[spec/flatmarkdown|Specification]]

Link to a block: [[spec/flatmarkdown#2026-09-26-18-14-47-32W15WT|Metadata example]]
```

- Inline syntax.
- Write the page name as a full path.
- An item ID after `#` links to that item.
- The whitespace around the page name is dropped. `[[ page ]]` is the same as `[[page]]`.
- A Wikilink without a page name, such as `[[]]` or `[[|label]]`, stays plain text.
- When written to a Markdown file, it is converted to a Markdown link. See "Wikilink Syntax" in SPEC.md for details.

## multiline_block_quotes

Surround the text with lines that contain only `>>>`. The lines inside become a block quote without a leading `>`.

```markdown
>>>
First paragraph of the quote.

Second paragraph of the quote.

- A list inside the quote
>>>
```

- A multiline block quote is one block. The example above becomes one item.
- It stays one item no matter how many paragraphs or lists it contains.
- The result is the same as a normal block quote with `>` on every line. It is handy for long quotes.

## Summary

| Extension | Syntax | Kind | Item boundary |
| --- | --- | --- | --- |
| footnote | `[^1]` / `[^1]: text` / `^[text]` | inline + definition line | definition joins the previous item |
| alert | `> [!NOTE]` | block | one item |
| subtext | `-# text` | block | one item |
| highlight | `==text==` | inline | no effect |
| superscript | `^text^` | inline | no effect |
| subscript | `~text~` | inline | no effect |
| spoilered_text | `\|\|text\|\|` | inline | no effect |
| shortcode | `:tada:` | inline | no effect |
| math | `` $`x`$ `` | inline | no effect |
| hashtag | `#tag` | inline | no effect |
| underline | `__text__` | inline | no effect |
| wikilink | `[[path\|label]]` | inline | no effect |
| multiline_block_quotes | `>>>` ... `>>>` | block | one item |
