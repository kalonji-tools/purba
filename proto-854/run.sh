#!/usr/bin/env bash
# Reproduces every measurement of RESULTS.md, from a clean worktree at e2ccac8.
#
#   proto-854/run.sh <interpreter dir> [stage ...]
#
# <interpreter dir> holds python-build-standalone builds, each as
# cpython-<version>[+flags]-linux-x86_64-gnu/bin/python. Stages: pairs, planted,
# abi3t, suite (default: all four). Each tree, target dir and venv goes under
# proto-854/work/; each result under proto-854/runs/.
set -uo pipefail
uvpy=$(cd "${1:?usage: run.sh <interpreter dir> [stage ...]}" && pwd)
shift
stages=("${@:-pairs planted abi3t suite}")
read -r -a stages <<<"${stages[*]}"
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "${here}/.." && pwd)
work="${here}/work"
runs="${here}/runs"
base=e2ccac8
mkdir -p "${work}" "${runs}/raw"

# The repo's pinned toolchain (rust nightly, maturin), run from the trees below.
eval "$(mise env -C "${repo}" -s bash)"
mise_py=$(mise exec -C "${repo}" -- python -c 'import sys; print(sys.executable)')
py() { echo "${uvpy}/cpython-$1-linux-x86_64-gnu/bin/python"; }

declare -A patch=(
  [w1]=w1-version-and-symbol [w2]=w2-compile-time-and-symbol
  [w3]=w3-version-alone [w4]=w4-read-at-report-time
  [w1x]=w1-version-and-symbol.in-execution
)
# W1 with its debug check in purba-execution takes the test changes of W1.
declare -A suite_patch=([w1]=w1 [w1x]=w1 [w2]=w2 [w3]=w3 [w4]=w4)
ways=(w0 w1 w2 w3 w4)

# label, interpreter, the wheel it runs (the 8 pairs of the research)
pairs=(
  "3.12|${mise_py}|abi3"
  "3.12d|$(py 3.12.15+debug)|abi3"
  "3.14|$(py 3.14.8)|abi3"
  "3.14d|$(py 3.14.8+debug)|abi3"
  "3.15d|$(py 3.15.0+debug)|abi3"
  "3.14t|$(py 3.14.8+freethreaded)|314t"
  "3.14td|$(py 3.14.8+freethreaded+debug)|314t"
  "3.15t|$(py 3.15.0+freethreaded)|315t"
)
declare -A builder=([abi3]="${mise_py}" [314t]="$(py 3.14.8+freethreaded)"
  [315t]="$(py 3.15.0+freethreaded)" [abi3t-on-3.15]="$(py 3.15.0)"
  [abi3t-on-3.15t]="$(py 3.15.0+freethreaded)")

# A fresh tree at e2ccac8 with the patches applied. Removes only <dir>.
tree() { # <dir> [patch ...]
  local dir=$1
  shift
  rm -rf "${dir}"
  git clone -q --shared --no-checkout "${repo}" "${dir}"
  git -C "${dir}" checkout -q --detach "${base}"
  local p
  for p in "$@"; do git -C "${dir}" apply "${p}" || {
    echo "cannot apply ${p}" >&2
    exit 2
  }; done
}
code_patches() { [[ $1 == w0 ]] || echo "${here}/ways/${patch[$1]}.patch"; }

# Builds the fault wheel of <tree> with <kind>'s interpreter, in a target dir
# of its own. Prints the wheel's path, or BUILD-FAILED.
build() { # <tree> <kind> <log>
  local tree=$1 kind=$2 log=$3 out="$1.wheels-$2"
  rm -rf "${out}"
  if (cd "${tree}" && CARGO_TARGET_DIR="${tree}.target-${kind}" maturin build --release \
    --interpreter "${builder[${kind}]}" --features extension-module,fault --out "${out}") \
    >"${log}" 2>&1 && ls "${out}"/*.whl >/dev/null 2>&1; then
    ls "${out}"/*.whl
  else
    echo BUILD-FAILED
  fi
}

fixtures="${work}/fixtures"
rm -rf "${fixtures}"
cp -r "${here}/fixtures" "${fixtures}"
export PYTHONDONTWRITEBYTECODE=1

# One row per test: way, pair, test, exit, what the report names, the truth,
# and the verdict. The truth is the interpreter's own sys.abiflags, read in a
# fresh process that no test touched.
measure() { # <way> <label> <interpreter> <wheel> <tests...>
  local way=$1 label=$2 interp=$3 wheel=$4
  shift 4
  local venv="${work}/venv/${way}-${label}"
  rm -rf "${venv}"
  "${interp}" -m venv "${venv}" >/dev/null
  if [[ ${wheel} == BUILD-FAILED ]] ||
    ! "${venv}/bin/python" -m pip install --quiet --no-index "${wheel}" >"${venv}.pip" 2>&1; then
    printf '%s\t%s\t-\t-\t-\t-\tnot installed: %s\n' "${way}" "${label}" \
      "$( [[ ${wheel} == BUILD-FAILED ]] && echo 'build failed' || grep -m1 ERROR "${venv}.pip")"
    return
  fi
  local truth
  truth=$("${venv}/bin/python" -c 'import platform, sys; print(platform.python_version() + sys.abiflags)')
  local t raw code named verdict env=()
  for t in "$@"; do
    raw="${runs}/raw/${way}-${label}-${t}.stderr"
    case ${t} in
      missing-at-start) env=(PYTHONPATH="${fixtures}/site") file=test_control.py ;;
      *) env=() file="test_${t}.py" ;;
    esac
    (cd "${fixtures}/tests" && env "${env[@]}" PURBA_FAULT=assert "${venv}/bin/purba" "${file}") \
      >/dev/null 2>"${raw}"
    code=$?
    named=$(sed -n 's/^purba: an internal error is a bug in purba [^,]*, on CPython \([^,]*\), .*/\1/p' "${raw}")
    if [[ -z ${named} ]]; then
      verdict="no report: $(tail -n 1 "${raw}")"
      named=-
    elif [[ ${named} == "${truth}" ]]; then
      verdict=right
    else
      verdict=WRONG
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "${way}" "${label}" "${t}" "${code}" "${named}" "${truth}" "${verdict}"
  done
}

stage_pairs() {
  local out="${runs}/pairs.tsv" way kind pair label interp
  printf 'way\tpair\ttest\texit\treport names\ttruth\tverdict\n' >"${out}"
  for way in "${ways[@]}"; do
    # shellcheck disable=SC2046 # one path or none
    tree "${work}/pairs-${way}" $(code_patches "${way}")
    declare -A wheel=()
    for kind in abi3 314t 315t; do
      wheel[${kind}]=$(build "${work}/pairs-${way}" "${kind}" "${runs}/raw/build-pairs-${way}-${kind}.log")
    done
    for pair in "${pairs[@]}"; do
      IFS='|' read -r label interp kind <<<"${pair}"
      measure "${way}" "${label}" "${interp}" "${wheel[${kind}]}" \
        control sets_abiflags deletes_abiflags missing-at-start >>"${out}"
    done
    unset wheel
  done
  column -t -s $'\t' "${out}" >"${runs}/pairs.txt"
}

# A planted bug: name, way, wheel kind, pairs, and the shell edit that plants it
# (run inside the tree, after the way's patch).
lib=crates/purba/src/lib.rs
planted=(
  "w1-d-misspelt|w1|abi3|3.12 3.12d 3.15d|perl -pi -e 's/_Py_NegativeRefcount/_Py_NegativeRefCount/' ${lib}"
  "w1-t-misspelt|w1|314t|3.14t 3.14td|perl -pi -e 's/ free-threading build / free-threaded build /' ${lib}"
  "w2-no-build-rs|w2|314t|3.14t|rm crates/purba/build.rs"
  "w2-no-build-rs-no-dep|w2|314t|3.14t|rm crates/purba/build.rs && perl -0pi -e 's/\\[build-dependencies\\]\\npyo3-build-config = \"0.29.2\"\\n\\n//' crates/purba/Cargo.toml"
  "w2-cfg-unset|w2|314t|3.14t 3.14td|perl -pi -e 's/use_pyo3_cfgs\\(\\)/print_expected_cfgs()/' crates/purba/build.rs"
  "w2-d-misspelt|w2|abi3|3.12d|perl -pi -e 's/_Py_NegativeRefcount/_Py_NegativeRefCount/' ${lib}"
  "w3-t-misspelt|w3|314t|3.14t|perl -pi -e 's/ free-threading build / free-threaded build /' ${lib}"
  "w4-attr-misspelt|w4|abi3|3.12 3.12d|perl -pi -e 's/getattr\\(\"abiflags\"\\)/getattr(\"abiflag\")/' ${lib}"
)

# The two cases of tests/fault.bats that compare the report line, on <venv>'s purba.
fault_bats() { # <tree> <venv>
  (cd "$1" && FAULT_PURBA="$2/bin/purba" \
    bats --filter '^the fault wheel panics in the assert hook|sys.abiflags' tests/fault.bats 2>&1 |
    grep -E '^(ok|not ok)')
}

stage_planted() {
  local out="${runs}/planted.tsv" row name way kind labels edit dir wheel pair label interp k
  printf 'plant\tpair\ttest\texit\treport names\ttruth\tverdict\n' >"${out}"
  : >"${runs}/planted-bats.txt"
  for row in "${planted[@]}"; do
    IFS='|' read -r name way kind labels edit <<<"${row}"
    dir="${work}/planted-${name}"
    tree "${dir}" "${here}/ways/${patch[${way}]}.patch"
    (cd "${dir}" && eval "${edit}")
    git -C "${dir}" diff --stat >"${runs}/raw/planted-${name}.diffstat"
    wheel=$(build "${dir}" "${kind}" "${runs}/raw/build-planted-${name}.log")
    if [[ ${wheel} == BUILD-FAILED ]]; then
      printf '%s\t-\t-\t-\t-\t-\tbuild failed: %s\n' "${name}" \
        "$(grep -E '^error' "${runs}/raw/build-planted-${name}.log" | head -n 2 | paste -sd ' ')" >>"${out}"
      continue
    fi
    for label in ${labels}; do
      for pair in "${pairs[@]}"; do
        IFS='|' read -r k interp _ <<<"${pair}"
        [[ ${k} == "${label}" ]] || continue
        measure "${name}" "${label}" "${interp}" "${wheel}" control sets_abiflags >>"${out}"
        echo "== ${name} on ${label}" >>"${runs}/planted-bats.txt"
        fault_bats "${dir}" "${work}/venv/${name}-${label}" >>"${runs}/planted-bats.txt"
      done
    done
  done
  # The control: each way unplanted, on the venvs of stage pairs.
  for way in w1 w2 w3 w4; do
    for label in 3.12 3.12d 3.14t 3.14td; do
      echo "== control: ${way} unplanted on ${label}" >>"${runs}/planted-bats.txt"
      fault_bats "${work}/pairs-${way}" "${work}/venv/${way}-${label}" >>"${runs}/planted-bats.txt"
    done
  done
  column -t -s $'\t' "${out}" >"${runs}/planted.txt"
}

# The research's M11 shape inside W1 and W2: one abi3t wheel, run on 3.15 and 3.15t.
stage_abi3t() {
  local out="${runs}/abi3t.tsv" way dir kind wheel label interp
  printf 'way\tpair\ttest\texit\treport names\ttruth\tverdict\n' >"${out}"
  for way in w1 w2; do
    dir="${work}/abi3t-${way}"
    tree "${dir}" "${here}/ways/${patch[${way}]}.patch"
    sed -i 's/features = \["abi3-py312"\]/features = ["abi3-py312", "abi3t-py315"]/' \
      "${dir}/crates/purba/Cargo.toml"
    for kind in abi3t-on-3.15 abi3t-on-3.15t; do
      wheel=$(build "${dir}" "${kind}" "${runs}/raw/build-abi3t-${way}-${kind}.log")
      for label in 3.15.0 3.15.0+freethreaded; do
        interp=$(py "${label}")
        measure "${way}-${kind}" "${label}" "${interp}" "${wheel}" control sets_abiflags >>"${out}"
      done
      echo "${way} ${kind}: ${wheel##*/}" >>"${runs}/abi3t-wheels.txt"
    done
  done
  column -t -s $'\t' "${out}" >"${runs}/abi3t.txt"
}

# The suite on the default mise interpreter: the way's code alone, then with its
# test and record changes. W0 runs once, as the baseline.
stage_suite() {
  local way dir
  : >"${runs}/suite-summary.txt"
  for way in "${ways[@]}" w1x; do
    dir="${work}/suite-${way}"
    # shellcheck disable=SC2046 # one path or none
    tree "${dir}" $(code_patches "${way}")
    if [[ ${way} != w0 ]]; then
      (cd "${dir}" && mise run test) >"${runs}/suite-${way}-code-alone.log" 2>&1
      echo "${way} code alone: mise run test exit $?" >>"${runs}/suite-summary.txt"
      grep -E 'not ok|test .* FAILED' "${runs}/suite-${way}-code-alone.log" \
        >>"${runs}/suite-summary.txt"
      git -C "${dir}" apply "${here}/ways/${patch[${suite_patch[${way}]}]}.suite.patch"
    fi
    (cd "${dir}" && mise run test) >"${runs}/suite-${way}-test.log" 2>&1
    echo "${way} with its test changes: mise run test exit $?" >>"${runs}/suite-summary.txt"
    grep -E 'not ok|test .* FAILED' "${runs}/suite-${way}-test.log" >>"${runs}/suite-summary.txt"
    (cd "${dir}" && mise run lint) >"${runs}/suite-${way}-lint.log" 2>&1
    echo "${way} with its test changes: mise run lint exit $?" >>"${runs}/suite-summary.txt"
    grep -E '^\[lint:[^]]*\] ERROR|^error' "${runs}/suite-${way}-lint.log" >>"${runs}/suite-summary.txt"
  done
  # W3's change to tests/internal_error.sh, on the debug venvs of stage pairs.
  for label in 3.12d 3.14td 3.14t; do
    echo "== w3 with its suite patch on ${label}"
    fault_bats "${work}/suite-w3" "${work}/venv/w3-${label}"
  done >"${runs}/w3-suite-patch-on-debug.txt"
}

for s in "${stages[@]}"; do "stage_${s}"; done
