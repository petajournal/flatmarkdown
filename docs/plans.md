2026-11-09 ギア5速：テストもコードもAIが書く。人間は最初の設計に関わるだけで、コードレビューはしない

# plans

## ユーザータスク

- [ ] `fm-ast`を、`flatmarkdown-ast`クレートとして公開する
  - 決定と全体のタスクは、peta-journalのリポジトリにある（`docs/adr/adr-012-flatmarkdown-ast-as-a-published-crate.md`と`docs/plans.md`）。ここには、このリポジトリの分だけを書く
  - [x] リポジトリをCargoのワークスペースにし、既存のクレートを`flatmarkdown/`へ移す
  - [x] `flatmarkdown-ast/`を足し、peta-journalの`fm-ast`のコードとテストを移す
  - [x] 2つのクレートのREADMEと、リポジトリのREADMEを書く
  - [ ] 公開する版を決める（ユーザーが決める）
  - [ ] `cargo publish --dry-run`で確かめてから、`flatmarkdown`、`flatmarkdown-ast`の順に公開する（ユーザーが行う）
- [ ] `body_to_html`が、文字として扱う値をエスケープせずにHTMLへ入れる不具合を直す
  - [x] コードフェンスの言語名は、`class`属性の中でエスケープされる
  - [ ] アラートの題名は、文字としてエスケープされる
  - [ ] 画像の`alt`には、子の文字だけが入る。子のタグは入らない
  - [ ] 本文の生のHTMLと`javascript:`のリンクをそのまま出すのは、意図した動きである。READMEに書く
