# flatmarkdown

The Rust crates for Flat Markdown, an outline format with block-delimited items.

Flat Markdown is defined in the [Flat Markdown specification](https://github.com/petajournal/flatmarkdown-spec/).

## Crates

| Crate | What it does |
|---|---|
| [flatmarkdown](flatmarkdown/) | Parses a page or an item body into a JSON AST, and renders an item body to HTML |
| [flatmarkdown-ast](flatmarkdown-ast/) | The AST as Rust types. Reads and writes the JSON AST with serde. It has no Markdown parser |

`flatmarkdown` builds its AST with the types of `flatmarkdown-ast`, and writes the JSON from them.
So the types are the only place where the shape of the AST is written in Rust.
`flatmarkdown-ast` does not depend on `flatmarkdown`.

The AST is defined in [SPEC_AST.md](SPEC_AST.md) ([日本語](SPEC_AST_ja.md)).
The same AST as TypeScript types is the npm package [flatmarkdown-ast](https://www.npmjs.com/package/flatmarkdown-ast). Its source is the [flatmarkdown-ast-ts](https://github.com/petajournal/flatmarkdown-ast-ts) repository.

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

[SPEC_AST.md](SPEC_AST.md) is the source of truth for the AST.
The types of `flatmarkdown-ast` hold what the specification defines, and nothing more.
When the AST changes, change the specification first, and then the types and the parser.

GitHub Actions runs the same commands on every pull request and on `main`.
It also runs `cargo package --workspace`, to build each crate from its packaged files.
A change of only Markdown files or `docs/` does not start it.

The crates need Rust 1.88 or later. This is `rust-version` in the top `Cargo.toml`.
The CI also builds the workspace with Rust 1.88, so that the declared version stays true.
When you raise `rust-version`, change the version of the `msrv` job in `.github/workflows/ci.yml` too.

Clippy warnings do not fail the build.
Only the lints in `[workspace.lints]` of the top `Cargo.toml` are errors: `unsafe_code`, `unused_must_use`, `dbg_macro`, `todo`, and `unimplemented`.

## Release

1. Set `version` in the `Cargo.toml` at the top of the repository. Set the same version in `flatmarkdown-ast` of `[workspace.dependencies]` in that file. Merge the change into `main`.
2. Run `cargo publish --dry-run -p flatmarkdown-ast`.
3. Run `cargo publish -p flatmarkdown-ast`.
4. Run `cargo publish --dry-run -p flatmarkdown`, and then `cargo publish -p flatmarkdown`.

`flatmarkdown-ast` goes first, because `flatmarkdown` depends on it.

Publish both crates every time, even when only one of them changed.
A published version cannot be deleted or replaced.

## License

MIT
