# flatmarkdown

The Rust crates for Flat Markdown, an outline format with block-delimited items.

Flat Markdown is defined in [SPEC.md](SPEC.md) ([日本語](SPEC_ja.md)).
[example.md](example.md) ([日本語](example_ja.md)) shows the syntax of each extension.

## Crates

| Crate | What it does |
|---|---|
| [flatmarkdown](flatmarkdown/) | Parses a page or an item body into the AST |
| [flatmarkdown-ast](flatmarkdown-ast/) | The AST as Rust types. Reads and writes the JSON AST with serde. It has no Markdown parser |
| [flatmarkdown-ast2html](flatmarkdown-ast2html/) | Renders the AST to HTML. It has no Markdown parser |
| [flatmarkdown-ast2html-wasm](flatmarkdown-ast2html-wasm/) | The WebAssembly build of `flatmarkdown-ast2html`. It is the source of the npm package [flatmarkdown-ast2html](https://www.npmjs.com/package/flatmarkdown-ast2html), and is not on crates.io |

`flatmarkdown` builds its AST with the types of `flatmarkdown-ast`, and returns them.
`flatmarkdown-ast` writes the JSON from them.
So the types are the only place where the shape of the AST is written in Rust.

`flatmarkdown` and `flatmarkdown-ast2html` both depend on `flatmarkdown-ast` only.
They do not depend on each other. Only the tests of `flatmarkdown-ast2html` use the parser.

``rust
let html = flatmarkdown_ast2html::ast_to_html(&flatmarkdown::parse_body("Hello **world**"));
``

The AST is defined in [SPEC_AST.md](SPEC_AST.md) ([日本語](SPEC_AST_ja.md)).
The same AST as TypeScript types is the npm package [flatmarkdown-ast](https://www.npmjs.com/package/flatmarkdown-ast). Its source is the [flatmarkdown-ast-ts](https://github.com/petajournal/flatmarkdown-ast-ts) repository.

## Versions

All crates always have the same version.
The version is written once, in the `Cargo.toml` at the top of the repository.
The npm package `flatmarkdown-ast2html` has this version too. Its `package.json` repeats the version, and a test compares the two.
The AST made by `flatmarkdown` x.y.z can be read by `flatmarkdown-ast` x.y.z, and rendered by `flatmarkdown-ast2html` x.y.z.

A change of the AST goes into one pull request: the parser, the specification, the types, and the renderer.

A new kind of node is a breaking change, because `Node` of `flatmarkdown-ast` is not `#[non_exhaustive]`.
The version then goes up, for example from 0.4 to 0.5.

## Development

Run these at the top of the repository. They cover all crates.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

[SPEC.md](SPEC.md) is the source of truth for the format.
[SPEC_AST.md](SPEC_AST.md) is the source of truth for the AST.
The types of `flatmarkdown-ast` hold what the specification defines, and nothing more.
When the AST changes, change the specification first, and then the types and the parser.

GitHub Actions runs the same commands on every pull request and on `main`.
It also runs `cargo package`, to build each published crate from its packaged files.
A third job builds the npm package and runs its tests. See the [README of flatmarkdown-ast2html-wasm](flatmarkdown-ast2html-wasm/README.md) to do the same on your machine.
A change of only Markdown files or `docs/` does not start it.

The crates need Rust 1.88 or later. This is `rust-version` in the top `Cargo.toml`.
The CI also builds the workspace with Rust 1.88, so that the declared version stays true.
When you raise `rust-version`, change the version of the `msrv` job in `.github/workflows/ci.yml` too.

Clippy warnings do not fail the build.
Only the lints in `[workspace.lints]` of the top `Cargo.toml` are errors: `unsafe_code`, `unused_must_use`, `dbg_macro`, `todo`, and `unimplemented`.

## Release

1. Set `version` in the `Cargo.toml` at the top of the repository. Set the same version in `flatmarkdown-ast` of `[workspace.dependencies]` in that file, in `flatmarkdown-ast2html` of `flatmarkdown-ast2html-wasm/Cargo.toml`, and in `flatmarkdown-ast2html-wasm/package.json`. Merge the change into `main`.
2. Run `cargo publish --dry-run -p flatmarkdown-ast`.
3. Run `cargo publish -p flatmarkdown-ast`.
4. Run `cargo publish --dry-run -p flatmarkdown`, and then `cargo publish -p flatmarkdown`.
5. Run `cargo publish --dry-run -p flatmarkdown-ast2html`, and then `cargo publish -p flatmarkdown-ast2html`.
6. In `flatmarkdown-ast2html-wasm`, run `npm publish --dry-run`, and then `npm publish`. Both build the package and run its tests first.

`flatmarkdown-ast` goes first, because the other two depend on it.

Publish all three crates and the npm package every time, even when only one of them changed.
A published version cannot be deleted or replaced.

`flatmarkdown-ast2html-wasm` itself is not published to crates.io. Its `Cargo.toml` has `publish = false`.

## License

MIT
