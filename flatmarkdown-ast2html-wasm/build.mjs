// Builds dist/index.global.js: one file that a page loads with <script>.
//
// The file holds the wasm as base64, so that a page needs no second request. The stickies app
// of peta-journal puts the file into a document that it hands over as a string, where a
// second file cannot be fetched (ADR 023 of peta-journal).

import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, '..', 'target', 'wasm-pkg');
const dist = join(here, 'dist');

// wasm-pack writes the wasm file and the JavaScript that talks to it.
// `no-modules` gives a plain script. The other targets give ES modules.
execFileSync(
    'wasm-pack',
    [
        'build',
        here,
        '--target',
        'no-modules',
        '--no-typescript',
        '--no-pack',
        '--profile',
        'wasm-release',
        '--out-dir',
        pkg,
        '--out-name',
        'fm_ast2html',
    ],
    { stdio: 'inherit' },
);

const glue = readFileSync(join(pkg, 'fm_ast2html.js'), 'utf8');
const wasm = readFileSync(join(pkg, 'fm_ast2html_bg.wasm')).toString('base64');
const { version } = JSON.parse(readFileSync(join(here, 'package.json'), 'utf8'));

// The function keeps `wasm_bindgen` of the glue out of the global scope. `initSync` compiles
// the wasm at once, so that `renderToHtml` works as soon as the script has run.
const bundle = `/*! flatmarkdown-ast2html ${version} | MIT License | https://github.com/petajournal/flatmarkdown */
var FmAst2Html = (function () {
${glue}
var text = atob('${wasm}');
var bytes = new Uint8Array(text.length);
for (var i = 0; i < text.length; i++) bytes[i] = text.charCodeAt(i);
wasm_bindgen.initSync({ module: bytes });
return {
    renderToHtml: function (ast) {
        return wasm_bindgen.renderToHtml(typeof ast === 'string' ? ast : JSON.stringify(ast));
    },
};
})();
`;

rmSync(dist, { recursive: true, force: true });
mkdirSync(dist);
writeFileSync(join(dist, 'index.global.js'), bundle);
copyFileSync(join(here, '..', 'LICENSE'), join(dist, 'LICENSE'));
console.log(`dist/index.global.js: ${Math.round(bundle.length / 1024)}KB`);