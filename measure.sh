#!/usr/bin/env bash
# PROTOTYPE for purba #536: measure where PyO3 lives, in both workspaces.
# S is a scratch directory; PY312 is a 3.12 interpreter.
set -u
S=${S:?set S to a scratch directory}
PY312=${PY312:?}
ROOT=$(cd "$(dirname "$0")" && pwd)
LIBDIR=$("${PY312}" -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR"))')

summary() { # $1 label, rest: command; prints the test result lines and the exit code
  local label=$1
  shift
  local out
  out=$("$@" 2>&1)
  local code=$?
  echo "  ${label}: exit ${code}"
  grep -E '^test result|^test |cannot open shared object' <<<"${out}" | sed 's/^/      /' || true
}

for variant in pyo3-in-the-part pyo3-only-in-lib; do
  cd "${ROOT}/${variant}" || exit 1
  echo "################ ${variant}"
  echo "-- crates that depend on pyo3 directly"
  cargo tree -i pyo3 -e normal --depth 1 --offline 2>/dev/null |
    sed -n '2,$p' | sed 's/^/  /' || true
  echo "-- core's tests, with no libpython on the loader path"
  summary "unit" env -u LD_LIBRARY_PATH cargo test -q -p purba-core --lib
  summary "doc" env -u LD_LIBRARY_PATH cargo test -q -p purba-core --doc
  echo "-- core's and lib's tests, with libpython on the loader path"
  summary "core unit" env LD_LIBRARY_PATH="${LIBDIR}" cargo test -q -p purba-core --lib
  summary "lib unit" env LD_LIBRARY_PATH="${LIBDIR}" cargo test -q -p purba --lib
  echo "-- lines of Rust in each crate"
  for crate in cli core lib; do
    lines=$(awk 'END { print NR }' crates/"${crate}"/src/*.rs)
    echo "  ${crate}: ${lines}"
  done
  echo "-- the wheel, on the skeleton's test_a.py"
  rm -rf "${S:?}/${variant:?}"
  maturin build -q --release --interpreter "${PY312}" -o "${S}/${variant}/wheels" >/dev/null 2>&1
  "${PY312}" -m venv "${S}/${variant}/venv"
  "${S}/${variant}/venv/bin/pip" -q install --no-index "${S}/${variant}"/wheels/*.whl
  (cd "${ROOT}/tests/skeleton" && "${S}/${variant}/venv/bin/purba" test_a.py \
    >"${S}/${variant}/out.txt")
  code=$?
  sed "s/^/  /" "${S}/${variant}/out.txt"
  echo "  exit: ${code}"
  echo
done
