#!/usr/bin/env bash
# TypeScript's file set: `include` and `exclude` globs, an explicit `files` list,
# and each file that an included file imports.
# usage: nix-shell -p nodejs --run "bash run.sh <dir with node_modules/typescript@5.9.3> <empty dir>"
set -u
DEPS=$1
D=$2
TSC="$DEPS/node_modules/.bin/tsc"
cd "$D" || exit 1
echo "== tsc --version"
"$TSC" --version

mkdir -p p/src/skip p/other && cd p || exit 1
printf 'import { b } from "./skip/b";\nexport const a = b;\n' >src/a.ts
printf 'export const b = 1;\n' >src/skip/b.ts
printf 'export const c = 1;\n' >src/skip/c.ts
printf 'export const o = 1;\n' >other/o.ts
list() { "$TSC" -p tsconfig.json --listFilesOnly 2>&1 | grep -v node_modules | sed "s#$D/p/##"; }

echo "== 1. no include: the default is every file under the folder"
printf '{"compilerOptions": {"noEmit": true, "types": []}}\n' >tsconfig.json
list
echo "== 2. include src, exclude src/skip: b.ts stays, since a.ts imports it"
printf '{"compilerOptions": {"noEmit": true, "types": []}, "include": ["src"], "exclude": ["src/skip"]}\n' >tsconfig.json
list
echo "== 3. files lists one file that does not exist"
printf '{"compilerOptions": {"noEmit": true, "types": []}, "files": ["src/a.ts", "src/gone.ts"]}\n' >tsconfig.json
list
echo "== 4. include matches no file"
printf '{"compilerOptions": {"noEmit": true, "types": []}, "include": ["nothing"]}\n' >tsconfig.json
list
