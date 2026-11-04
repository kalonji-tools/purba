// Count the words of a Markdown file, the words inside every fence included.
// A word is a run of letters or digits; an apostrophe inside a word joins it.
// A link destination `](...)` is dropped: the reader sees the link text, not the URL.
// Inside a fence every token counts, node ids and keywords too, so a figure is never undercounted.
// Usage: node count-words.mjs <file.md>...   prints "<count> <file>"
import { readFileSync } from 'node:fs';

const WORD = /[\p{L}\p{N}]+(?:['’][\p{L}]+)*/gu;

export function countWords(text) {
  let total = 0;
  for (const line of text.split('\n')) {
    if (/^\s*```/.test(line)) continue; // the fence markers themselves
    const visible = line.replace(/\]\([^)]*\)/g, ']');
    total += (visible.match(WORD) || []).length;
  }
  return total;
}

for (const file of process.argv.slice(2)) {
  console.log(`${countWords(readFileSync(file, 'utf8'))} ${file}`);
}
