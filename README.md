# flatmarkdown

The Rust crates for Flat Markdown, an outline format with block-delimited items.

Flat Markdown is defined in the [Flat Markdown specification](https://github.com/petajournal/flatmarkdown-spec/).

## Crates

| Crate | What it does |
|---|---|
| [flatmarkdown](flatmarkdown/) | Parses a page or an item body into a JSON AST, and renders an item body to HTML |
| [flatmarkdown-ast](flatmarkdown-ast/) | Reads the JSON AST into Rust types with serde. It has no Markdown parser |

The AST is defined in [SPEC_AST.md](SPEC_AST.md) ([日本語](SPEC_AST_ja.md)).
The same AST as TypeScript types is the npm package [flatmarkdown-ast](https://www.npmjs.com/package/flatmarkdown-ast).

## Versions

Both crates always have the same version.
The version is written once, in the `Cargo.toml` at the top of the repository.
The AST made by `flatmarkdown` x.y.z can be read by `flatmarkdown-ast` x.y.z.

A change of the AST goes into one pull request: the parser, the specification, and the types.

A new kind of node is a breaking change, because `Node` of `flatmarkdown-ast` is not `#[non_exhaustive]`.
The version then goes up, for example from 0.4 to 0.5.

## Development

Run these at the top of the repository. They cover both crates.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

The round-trip tests of `flatmarkdown-ast` parse Markdown with the `flatmarkdown` crate of this repository.
So they always test the two crates of the same version together.

## Release

1. Set `version` in the `Cargo.toml` at the top of the repository, and merge the change into `main`.
2. Run `cargo publish --dry-run -p flatmarkdown` and `cargo publish --dry-run -p flatmarkdown-ast`.
3. Run `cargo publish -p flatmarkdown`, and then `cargo publish -p flatmarkdown-ast`.

Publish both crates every time, even when only one of them changed.
A published version cannot be deleted or replaced.

## License

MIT
