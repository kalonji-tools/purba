// Parse every ```mermaid fence in the given Markdown files with the Mermaid GitHub runs.
// Usage: node parse-fences.mjs <deps-dir> <file.md>...
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import path from 'node:path';

const [depsDir, ...files] = process.argv.slice(2);
const require = createRequire(path.join(path.resolve(depsDir), 'package.json'));
const { JSDOM } = require('jsdom');
const dom = new JSDOM('<!doctype html><html><body></body></html>');
globalThis.window = dom.window;
globalThis.document = dom.window.document;
const mermaid = (await import(require.resolve('mermaid'))).default;
mermaid.initialize({ startOnLoad: false });

let failures = 0;
let total = 0;
for (const file of files) {
  const lines = readFileSync(file === '-' ? 0 : file, 'utf8').split('\n');
  for (let index = 0; index < lines.length; index++) {
    if (lines[index].trim() !== '```mermaid') continue;
    const start = index + 1;
    const body = [];
    index++;
    while (index < lines.length && lines[index].trim() !== '```') body.push(lines[index++]);
    total++;
    try {
      await mermaid.parse(body.join('\n'));
      console.log(`ok   ${file}:${start}`);
    } catch (error) {
      failures++;
      const message = String(error.message || error).split('\n').slice(0, 4).join(' / ');
      console.log(`FAIL ${file}:${start} ${message}`);
    }
  }
}
console.log(`${total} fences, ${failures} failed`);
process.exit(failures ? 1 : 0);
