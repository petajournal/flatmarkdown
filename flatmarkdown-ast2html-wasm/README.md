# flatmarkdown-ast2html

Renders the [Flat Markdown](https://github.com/petajournal/flatmarkdown) AST to HTML in a web page.

This npm package is the Rust crate [flatmarkdown-ast2html](https://crates.io/crates/flatmarkdown-ast2html), built as WebAssembly.
So the package and the crate give the same HTML.
The version of the package is the version of the crate.

The HTML is described in the [README of the crate](https://github.com/petajournal/flatmarkdown/tree/main/flatmarkdown-ast2html#readme).

## Use

The package is one JavaScript file, `dist/index.global.js`.
The WebAssembly code is inside that file, so a page needs no second file.
Load the file with a `<script>` tag.

```html
<script src="https://unpkg.com/flatmarkdown-ast2html@0.9.0"></script>
<script>
  const html = FmAst2Html.renderToHtml(
    '{"type":"paragraph","children":[{"type":"text","value":"Hello"}]}',
  );
  // "<p>Hello</p>\n"
</script>
```

The script adds one global object, `FmAst2Html`.

### `FmAst2Html.renderToHtml(ast, options)`

`ast` is the AST as a JSON string, or as an object.
The function returns the HTML as a string. It does not return a promise.

`options` can be left out. It is an object with these settings. A setting that is left out has its default value.

- `wikilinkHref`: the template of the `href` of a wikilink. The default is `{}`.
- `hashtagHref`: the template of the `href` of a hashtag. The default is `{}`.
- `wikilinkFragmentPattern`: the pattern of the fragment of a wikilink. The default is empty.

They are the settings of `HtmlOptions` of the crate, and give the same HTML.
See "Where links lead" in the [README of the crate](https://github.com/petajournal/flatmarkdown/tree/main/flatmarkdown-ast2html#readme).

```js
FmAst2Html.renderToHtml(ast, { wikilinkHref: '/pages/{}', hashtagHref: '/pages/tags/{}' });
```

The function throws when `ast` cannot be read as an AST.
For example, it throws for a string that is not JSON.
A node of an unknown `type` is not an error. Only its children are rendered.

The HTML holds the raw HTML of the Markdown as it is.
Do not render the AST of Markdown that you do not trust.

## Changes from 0.1

Versions 0.1.x were written in TypeScript. From 0.6.0, the package is built from Rust.

- The second argument of `renderToHtml`, the options for wikilinks and hashtags, was removed in 0.6.0. From 0.9.0, it is back with other settings.
- The ES module build, the CommonJS build, and the TypeScript types are removed.
- The types of the AST are not exported. They are in the npm package [flatmarkdown-ast](https://www.npmjs.com/package/flatmarkdown-ast).
- The file is about 340KB. It was about 4KB.

## Build

You need Rust with the target `wasm32-unknown-unknown`, [wasm-pack](https://rustwasm.github.io/wasm-pack/), and Node.js.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked
```

Run these in this folder.

```sh
npm run build
npm test
```

`npm run build` writes `dist/index.global.js`.
`npm test` runs that file in Node.js, as a page would run it. Build before you test.

The package has no npm dependencies. You do not need `npm install`.

## License

MIT