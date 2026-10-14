#!/usr/bin/env bash
# PROTOTYPE for purba #537: how each layout shows a part's boundary, measured.
#
#   S=<scratch dir> PY312=<a 3.12 python> mise exec -- ./measure.sh [section...]
#
# Sections: arrow libpython repo wheel. With none, it runs all four. Every probe
# runs on a copy under $S, and the repo section on a detached worktree of
# origin/main under $S, which it removes.
set -uo pipefail
S=${S:?set S to a scratch directory}
PY312=${PY312:?set PY312 to a 3.12 interpreter}
ROOT=$(cd "$(dirname "$0")" && pwd)
LIBDIR=$("${PY312}" -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR"))')
export PYO3_PYTHON=${PY312} CARGO_TARGET_DIR=${S}/target
LAYOUTS=(one-crate cli-lib-core crate-per-part)
SECTIONS=("${@:-arrow libpython repo wheel}")
wants() { [[ " ${SECTIONS[*]} " == *" $1 "* ]]; }

fresh() { # $1 layout, $2 copy name: a clean copy of the layout under $S
  rm -rf "${S:?}/$2" && cp -r "${ROOT}/$1" "${S}/$2" && echo "${S}/$2"
}
reporting_file() { # $1 layout dir
  local f
  for f in src/reporting.rs src/core/src/reporting.rs src/reporting/src/lib.rs; do
    [[ -f "$1/${f}" ]] && echo "$1/${f}" && return
  done
}
build() { # $1 dir: exit code of `cargo build`, and its first error line
  local out code
  out=$(cargo build --offline --workspace --manifest-path "$1/Cargo.toml" 2>&1)
  code=$?
  echo "    cargo build: exit ${code} $(grep -m1 '^error' <<<"${out}")"
}
wiring() {
  "${ROOT}/check-wiring.sh" "${ROOT}/LANDSCAPE.md" "$1" | sed 's/^/    /'
  echo "    check-wiring: exit ${PIPESTATUS[0]}"
}

# An arrow the diagram does not draw: reporting calls selection.
PROBE='
/// PROBE: reporting → selection, an arrow the diagram does not draw.
#[must_use]
pub fn count(path: &str, source: &str) -> usize {
    selection::select(path, source).len()
}'

if wants arrow; then
  echo "################ ARROW: reporting → selection, which the diagram does not draw"
  for layout in "${LAYOUTS[@]}"; do
    echo "== ${layout}"
    dir=$(fresh "${layout}" "arrow-${layout}")
    file=$(reporting_file "${dir}")
    case ${layout} in
      crate-per-part) import='use purba_selection as selection;' ;;
      *) import='use crate::selection;' ;;
    esac
    sed -i "0,/^use /s//${import}\nuse /" "${file}"
    printf '%s\n' "${PROBE}" >>"${file}"
    echo "  -- the code alone"
    build "${dir}"
    if [[ ${layout} == crate-per-part ]]; then
      echo "  -- the code, and the dependency in reporting's Cargo.toml"
      sed -i 's|^\[dependencies\]$|[dependencies]\npurba-selection = { path = "../selection" }|' \
        "${dir}/src/reporting/Cargo.toml"
      build "${dir}"
    fi
    wiring "${dir}"

    echo "  -- a bypass of the wiring check"
    dir=$(fresh "${layout}" "bypass-${layout}")
    file=$(reporting_file "${dir}")
    case ${layout} in
      crate-per-part)
        echo "    execution re-exports fixtures, and reporting calls it through execution"
        sed -i '0,/^use purba_fixtures as fixtures;$/s//pub use purba_fixtures as fixtures;/' \
          "${dir}/src/execution/src/lib.rs"
        printf '\n/// PROBE: reporting → fixtures, through execution.\n#[must_use]\npub fn inputs() -> usize {\n    execution::fixtures::build("test_one").len()\n}\n' >>"${file}"
        ;;
      *)
        echo "    reporting imports selection in a grouped use"
        sed -i 's/^use crate::execution;$/use crate::{execution, selection};/' "${file}"
        printf '%s\n' "${PROBE}" >>"${file}"
        ;;
    esac
    build "${dir}"
    wiring "${dir}"
  done
  echo
fi

if wants libpython; then
  echo "################ LIBPYTHON: which test binaries load without it (Linux)"
  for layout in "${LAYOUTS[@]}"; do
    echo "== ${layout}"
    while read -r name exe; do
      needs=no
      ldd "${exe}" | grep -q libpython && needs=yes
      out=$(env -u LD_LIBRARY_PATH "${exe}" 2>&1)
      code=$?
      printf '    %-18s libpython: %-3s  run without it: exit %-3s %s\n' "${name}" "${needs}" "${code}" \
        "$(grep -m1 -oE 'test result: [a-z]+\. [0-9]+ passed|cannot open shared object file' <<<"${out}")"
    done < <(cargo test --offline --workspace --no-run --message-format=json \
      --manifest-path "${ROOT}/${layout}/Cargo.toml" 2>/dev/null |
      jq -r 'select(.executable != null and .profile.test) | "\(.target.name) \(.executable)"')
  done
  echo
fi

if wants repo; then
  echo "################ REPO: main's own tasks, with each layout in place of src/ and Cargo.toml"
  for layout in "${LAYOUTS[@]}" crate-per-part-in-crates; do
    echo "== ${layout}"
    wt="${S}/repo-${layout}"
    git -C "${ROOT}" worktree remove --force "${wt}" 2>/dev/null
    git -C "${ROOT}" worktree add -q --detach "${wt}" origin/main
    rm -rf "${wt:?}/src" "${wt}/Cargo.toml"
    cp -r "${ROOT}/${layout%-in-crates}/src" "${ROOT}/${layout%-in-crates}/Cargo.toml" "${wt}/"
    if [[ ${layout} == *-in-crates ]]; then
      mkdir "${wt}/crates"
      for part in "${wt}"/src/*/; do mv "${part}" "${wt}/crates/"; done
      sed -i 's|"src/|"crates/|g' "${wt}/Cargo.toml"
    fi
    (
      cd "${wt}" || exit 1
      mise trust -q . >/dev/null 2>&1
      cargo metadata --offline --format-version 1 >/dev/null 2>&1 # rewrites Cargo.lock
      git add -A && git -c user.name=probe -c user.email=probe@invalid -c commit.gpgsign=false \
        commit -qm 'probe: the layout' --no-verify
      for task in lint:rust test:rust lint:manifests lint:gates lint:licences; do
        out=$(mise run "${task}" 2>&1)
        code=$?
        tests=$(grep -oE '^test result: [a-z]+\. [0-9]+ passed' <<<"${out}" | awk '{ n += $4 } END { print n + 0 }')
        ran=$(sed -nE "s/.*Running .*\((.*)\)\$/\1/p; s/.*Doc-tests (.*)/doc:\1/p" <<<"${out}" | sed "s|.*/||; s/-[0-9a-f]*\$//" | sort -u | tr "\n" " ")
        printf '    %-15s exit %s' "${task}" "${code}"
        [[ ${task} == test:rust ]] && printf '  tests passed: %s  binaries: %s' "${tests}" "${ran}"
        [[ ${code} != 0 ]] && printf '  %s' "$(grep -m1 -E '^error|refuse|not marked|unmarked' <<<"${out}" | cut -c1-120)"
        echo
      done
      out=$(LD_LIBRARY_PATH="${LIBDIR}" cargo test --offline --workspace --all-targets 2>&1)
      code=$?
      tests=$(grep -oE '^test result: [a-z]+\. [0-9]+ passed' <<<"${out}" | awk '{ n += $4 } END { print n + 0 }')
      echo "    control: cargo test --workspace, libpython on the path: exit ${code}  tests passed: ${tests}"
      part_file=$(git ls-files '*selection*' | grep -m1 '\.rs$')
      echo "// probe" >>"${part_file}"
      git commit -qam 'probe: a change inside selection' --no-verify
      named=$(scripts/list-records.sh HEAD~1 2>/dev/null | awk "/^\*\*Named by this change:\*\*/ { on = 1; next } on && /^- / { n++ } on && !NF { exit } END { print n + 0 }")
      echo "    a change inside ${part_file}: list:records names ${named} records"
      printf '\n/// PROBE: pedantic `must_use_candidate` and nursery `missing_const_for_fn`.\npub fn probe() -> usize {\n    1\n}\n' >>"${part_file}"
      out=$(mise run lint:rust 2>&1)
      code=$?
      echo "    lint:rust after a clippy breach inside ${part_file}: exit ${code} $(grep -m1 -- '-->' <<<"${out}")"
      git checkout -q -- "${part_file}"
      printf '\npub const  PROBE: usize = 1;\n' >>"${part_file}"
      out=$(mise run lint:rust 2>&1)
      code=$?
      echo "    lint:rust after a format breach inside ${part_file}: exit ${code} $(grep -m1 '^Diff in' <<<"${out}")"
    )
    git -C "${ROOT}" worktree remove --force "${wt}"
  done
  echo
fi

if wants wheel; then
  echo "################ WHEEL: maturin builds it, and the tester runs test_a.py"
  for layout in "${LAYOUTS[@]}"; do
    echo "== ${layout}"
    out="${S}/wheel-${layout}"
    rm -rf "${out:?}"
    (cd "${ROOT}/${layout}" && maturin build -q --release \
      --interpreter "${PY312}" -o "${out}/wheels" >/dev/null 2>&1) || {
      echo "    maturin: failed"
      continue
    }
    "${PY312}" -m venv "${out}/venv"
    "${out}/venv/bin/pip" -q install --no-index "${out}"/wheels/*.whl
    sentinel="${out}/test_b-was-imported"
    (cd "${ROOT}/tests/skeleton" && PURBA_SENTINEL="${sentinel}" "${out}/venv/bin/purba" test_a.py >"${out}/stdout")
    code=$?
    sed 's/^/    /' "${out}/stdout"
    echo "    exit ${code}; test_b.py imported: $([[ -e ${sentinel} ]] && echo yes || echo no)"
  done
fi
