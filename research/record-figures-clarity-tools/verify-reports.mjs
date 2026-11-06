// Check the sweep reports: every record listed once,
// and each quoted span equals the record's lines L-M.
// Usage: node verify-reports.mjs <sweep-dir>
import { readFileSync } from 'node:fs';
import path from 'node:path';

const dir = process.argv[2];
const toread = readFileSync(path.join(dir, 'toread.txt'), 'utf8').trim().split('\n');
const seen = new Map();
let problems = 0;
for (let n = 1; n <= 6; n++) {
  const text = readFileSync(path.join(dir, `report-${n}.md`), 'utf8');
  const tsv = text.match(/```tsv\n([\s\S]*?)```/)[1].trim().split('\n');
  const spans = {};
  for (const row of tsv) {
    const [file, verdict, k, range] = row.split('\t');
    seen.set(file, (seen.get(file) || 0) + 1);
    if (verdict === 'candidate') spans[`${file} --${k}`] = range;
  }
  for (const section of text.split(/^### /m).slice(1)) {
    const head = section.split('\n')[0].trim();
    const range = spans[head];
    if (!range) { console.log(`NO TSV ROW for section ${head}`); problems++; continue; }
    const quoted = section.split('The replaced span, verbatim:')[1].split('The figure')[0]
      .split('\n').filter((line) => line.startsWith('>')).map((line) => line.replace(/^> ?/, ''))
      .filter((line) => line.trim() !== '');
    const [from, to] = range.split('-').map(Number);
    const file = head.split(' --')[0];
    const actual = readFileSync(path.join(dir, 'before', file), 'utf8').split('\n')
      .slice(from - 1, to).filter((line) => line.trim() !== '');
    const same = quoted.length === actual.length && quoted.every((line, i) => line === actual[i]);
    console.log(`${same ? 'span ok  ' : 'SPAN DIFF'} ${head} ${range}`);
    if (!same) { problems++; console.log({ quoted, actual }); }
  }
}
for (const file of toread) {
  if (seen.get(file) !== 1) {
    console.log(`COVER ${file} seen ${seen.get(file) || 0}`);
    problems++;
  }
}
for (const file of seen.keys()) {
  if (!toread.includes(file)) { console.log(`EXTRA ${file}`); problems++; }
}
console.log(`${seen.size} records listed, ${problems} problems`);
process.exit(problems ? 1 : 0);
