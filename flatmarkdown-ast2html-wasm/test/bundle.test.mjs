// Tests of dist/index.global.js, the file that goes to npm. Run `npm run build` first.
//
// The file is run as a page would run it: as a plain script, with no module system.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import vm from 'node:vm';

const here = new URL('..', import.meta.url);
const source = readFileSync(new URL('dist/index.global.js', here), 'utf8');

// What a browser gives to a script, and nothing of Node.
const browserGlobals = () => ({ atob, TextDecoder, TextEncoder, WebAssembly, Uint8Array });

function loadBundle() {
    const page = vm.createContext(browserGlobals());
    vm.runInContext(source, page);
    return page;
}

const { FmAst2Html } = loadBundle();

const AST = {
    type: 'document',
    children: [
        {
            type: 'paragraph',
            children: [
                { type: 'text', value: 'a < b ' },
                { type: 'strong', children: [{ type: 'text', value: 'bold' }] },
            ],
        },
    ],
};

// JSONの文字列のASTは、HTMLになる。
test('an AST given as a JSON string is rendered to HTML', () => {
    const html = FmAst2Html.renderToHtml(JSON.stringify(AST));
    assert.equal(html, '<p>a &lt; b <strong>bold</strong></p>\n');
});

// オブジェクトのASTは、JSONの文字列のASTと同じHTMLになる。
test('an AST given as an object is rendered to the same HTML as its JSON string', () => {
    assert.equal(FmAst2Html.renderToHtml(AST), FmAst2Html.renderToHtml(JSON.stringify(AST)));
});

// 日本語と絵文字は、wasmとの間を通っても変わらない。
test('Japanese text and emoji pass through the wasm unchanged', () => {
    const html = FmAst2Html.renderToHtml({ type: 'text', value: '付箋紙📝' });
    assert.equal(html, '付箋紙📝');
});

// JSONでない文字列は、例外になる。
test('a string that is not JSON throws', () => {
    assert.throws(() => FmAst2Html.renderToHtml('not json'));
});

// ASTでないJSONは、例外になる。
test('JSON that is not an AST throws', () => {
    assert.throws(() => FmAst2Html.renderToHtml('{"children":[]}'));
});

// 例外のあとも、次の呼び出しはHTMLを返す。
test('a call after an exception still returns HTML', () => {
    assert.throws(() => FmAst2Html.renderToHtml('not json'));
    assert.equal(FmAst2Html.renderToHtml({ type: 'text', value: 'a' }), 'a');
});

// スクリプトがグローバルに足すのは、FmAst2Htmlだけである。
test('the script adds only FmAst2Html to the global scope', () => {
    const before = Object.keys(browserGlobals());
    const added = Object.keys(loadBundle()).filter((name) => !before.includes(name));
    assert.deepEqual(added, ['FmAst2Html']);
});

// URLのない文書（about:blank）でも、スクリプトは読み込める。
// WebView2のNavigateToStringに渡した文書が、この形である。
test('the script loads in a document that has no URL', () => {
    const page = vm.createContext({
        ...browserGlobals(),
        URL,
        document: { currentScript: { src: '' } },
        location: { href: 'about:blank' },
    });
    vm.runInContext(source, page);
    assert.equal(page.FmAst2Html.renderToHtml({ type: 'text', value: 'a' }), 'a');
});

// npmのパッケージのバージョンは、クレートのバージョンと同じである。
test('the version of the npm package is the version of the crates', () => {
    const cargo = readFileSync(new URL('../Cargo.toml', here), 'utf8');
    const crates = /\[workspace\.package\]\s+version = "([^"]+)"/.exec(cargo)[1];
    const npm = JSON.parse(readFileSync(new URL('package.json', here), 'utf8')).version;
    assert.equal(npm, crates);
});
const wikilink = (url) => ({ type: 'wikilink', url, children: [{ type: 'text', value: 'x' }] });
const hashtag = (value) => ({ type: 'hashtag', value });

// 2つ目の引数を省くと、wikilinkとハッシュタグのhrefは、符号化した名前だけである。
test('without the options the href of a wikilink and a hashtag is the encoded name', () => {
    assert.equal(
        FmAst2Html.renderToHtml({ type: 'paragraph', children: [wikilink('C#'), hashtag('a b')] }),
        '<p><a href="C%23" data-wikilink="true">x</a><a href="a%20b" class="hashtag">#a b</a></p>\n',
    );
});

// 2つ目の引数のテンプレートとフラグメントの形が、hrefを決める。
test('the templates and the fragment pattern of the options decide the hrefs', () => {
    const options = {
        wikilinkHref: '/pages/{}',
        hashtagHref: '/pages/tags/{}',
        wikilinkFragmentPattern: '[0-9]+',
    };
    assert.equal(
        FmAst2Html.renderToHtml({ type: 'paragraph', children: [wikilink('C##1'), hashtag('Rust')] }, options),
        '<p><a href="/pages/C%23#1" data-wikilink="true">x</a><a href="/pages/tags/Rust" class="hashtag">#Rust</a></p>\n',
    );
});

// 2つ目の引数で省いた設定は、既定の値になる。
test('a setting left out of the options has its default value', () => {
    const html = FmAst2Html.renderToHtml(hashtag('Rust'), { wikilinkHref: '/pages/{}' });
    assert.equal(html, '<a href="Rust" class="hashtag">#Rust</a>');
});

const hrefOf = (html) => /href="([^"]*)"/.exec(html)[1];
const escapeHtml = (s) => s.replaceAll('&', '&amp;').replaceAll('"', '&quot;');

// 表（hrefs.json）のどの行でも、描いたhrefは、表のhrefをHTMLのエスケープにかけたものである。
test('every href of the shared table is rendered escaped for HTML', () => {
    const table = JSON.parse(
        readFileSync(new URL('../flatmarkdown-ast2html/tests/hrefs.json', here), 'utf8'),
    );
    for (const c of table.cases) {
        const [ast, options] =
            c.kind === 'wikilink'
                ? [wikilink(c.name), { wikilinkHref: c.template, wikilinkFragmentPattern: c.fragmentPattern }]
                : [hashtag(c.name), { hashtagHref: c.template }];
        assert.equal(hrefOf(FmAst2Html.renderToHtml(ast, options)), escapeHtml(c.href), JSON.stringify(c));
    }
});
