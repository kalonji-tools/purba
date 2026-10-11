#!/usr/bin/env bash
# PROTOTYPE #482, thrown away. Which way of hosting CPython holds up when purba runs one
# test in a tester's virtual environment?
# Run from the repository root: mise exec -- PROTOTYPE-482-cpython-host/run.sh
set -uo pipefail

here=$(cd "$(dirname "$0")" && pwd)
out=${here}/out
suite=${here}/suite/test_one.py
mkdir -p "${out}"

py312_home=$(mise where python@3.12)
py314_home=$(mise where python@3.14)
py311_home=$(mise where python@3.11)
py312=${py312_home}/bin/python3.12
py314t=${py314_home}/bin/python3.14t
py311=${py311_home}/bin/python3.11
base_path=/run/current-system/sw/bin
pip_flags=(install --quiet --disable-pip-version-check)

# NixOS only: the Nix-built host has its own loader, which does not search nix-ld's
# directory, the stand-in for /usr/lib. Without it, numpy fails on libstdc++ in a host
# and not in python, and the prototype would measure this machine, not the arrangement.
nix_ld=/run/current-system/sw/share/nix-ld/lib

# What a launcher puts on LD_LIBRARY_PATH: the LIBDIR of the interpreter it found.
launcher_path() {
  local libdir
  libdir=$("$1" -c 'import sysconfig; print(sysconfig.get_config_var("LIBDIR"))')
  if [[ -d ${nix_ld} ]]; then
    printf '%s:%s' "${libdir}" "${nix_ld}"
  else
    printf '%s' "${libdir}"
  fi
}

step() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

# Run a command with nothing inherited but HOME, PATH and LD_LIBRARY_PATH.
# $1 is PATH, $2 is LD_LIBRARY_PATH ("" leaves it unset).
isolated() {
  local path=$1 ld=$2
  shift 2
  local environment=(HOME="${HOME}" PATH="${path}")
  [[ -n ${ld} ]] && environment+=(LD_LIBRARY_PATH="${ld}")
  env -i "${environment[@]}" "$@" 2>&1 | sed 's/^/  /'
  echo "  exit: ${PIPESTATUS[0]}"
}

step "build: extension module, abi3-py312 (the scaffold's shape today)"
cd "${here}/ext" || exit 1
maturin build --release --quiet -i "${py312}" -o "${out}/wheels-abi3" 2>&1 | tail -2
step "build: extension module, against 3.14t"
maturin build --release --quiet -i "${py314t}" -o "${out}/wheels-314t" 2>&1 | tail -4
for wheel in "${out}"/wheels-*/*.whl; do
  size=$(stat -c %s "${wheel}")
  echo "  ${size} bytes  ${wheel##*/}"
done

step "build: Rust host, against 3.12 and against 3.14t"
cd "${here}/host" || exit 1
for pair in "3.12:${py312}" "3.14t:${py314t}"; do
  tag=${pair%%:*} python=${pair#*:}
  PYO3_PYTHON=${python} cargo build --release --quiet --target-dir "${out}/target-${tag}"
  cp "${out}/target-${tag}/release/proto482_host" "${out}/host-${tag}"
  needs=$(ldd "${out}/host-${tag}" | awk '/libpython/ {print $1, $2, $3}')
  echo "  host-${tag} needs: ${needs}"
done

step "the tester's virtual environments, each with numpy"
for pair in "3.12:${py312}" "3.14t:${py314t}" "3.11:${py311}"; do
  tag=${pair%%:*} python=${pair#*:}
  venv=${out}/venv-${tag}
  [[ -d ${venv} ]] || "${python}" -m venv "${venv}"
  "${venv}/bin/pip" "${pip_flags[@]}" numpy 2>&1 | tail -1
  probe='import numpy, sys; print(sys.version.split()[0], numpy.__version__)'
  seen=$("${venv}/bin/python" -c "${probe}")
  echo "  venv-${tag}: ${seen}"
done

run_ext="import proto482_ext; proto482_ext.run('${suite}')"
v312=${out}/venv-3.12/bin
v314t=${out}/venv-3.14t/bin
v311=${out}/venv-3.11/bin
abi3_wheels=("${out}"/wheels-abi3/*.whl)
ft_wheels=("${out}"/wheels-314t/*.whl)
ld312=$(launcher_path "${py312}")
ld314t=$(launcher_path "${py314t}")

step "E1  extension module | abi3 wheel into the 3.12 venv, then run"
isolated "${v312}:${base_path}" "" "${v312}/pip" "${pip_flags[@]}" --force-reinstall \
  "${abi3_wheels[@]}"
isolated "${v312}:${base_path}" "" "${v312}/python" -c "${run_ext}"

step "E2  extension module | the same abi3 wheel into the 3.14t venv"
isolated "${v314t}:${base_path}" "" "${v314t}/pip" "${pip_flags[@]}" "${abi3_wheels[@]}"

step "E3  extension module | the wheel built for 3.14t into the 3.14t venv, then run"
isolated "${v314t}:${base_path}" "" "${v314t}/pip" "${pip_flags[@]}" --force-reinstall \
  "${ft_wheels[@]}"
isolated "${v314t}:${base_path}" "" "${v314t}/python" -c "${run_ext}"

step "E4  extension module | abi3 wheel into the 3.11 venv (below the floor)"
isolated "${v311}:${base_path}" "" "${v311}/pip" "${pip_flags[@]}" "${abi3_wheels[@]}"

step "H1  Rust host 3.12 | no launcher, 3.12 venv on PATH"
isolated "${v312}:${base_path}" "" "${out}/host-3.12" "${suite}"

step "H2  Rust host 3.12 | launcher sets LD_LIBRARY_PATH, 3.12 venv on PATH"
isolated "${v312}:${base_path}" "${ld312}" "${out}/host-3.12" "${suite}"

step "H3  Rust host 3.12 | launcher, no python on PATH"
isolated "${base_path}" "${ld312}" "${out}/host-3.12" "${suite}"

step "H4  Rust host 3.12 | launcher, the 3.14t venv on PATH"
isolated "${v314t}:${base_path}" "${ld312}" "${out}/host-3.12" "${suite}"

step "H5  Rust host 3.14t | launcher, the 3.14t venv on PATH"
isolated "${v314t}:${base_path}" "${ld314t}" "${out}/host-3.14t" "${suite}"

step "H6  Rust host 3.12 | launcher, a 3.12 python on PATH, the 3.12 venv not on PATH"
isolated "${py312_home}/bin:${base_path}" "${ld312}" "${out}/host-3.12" "${suite}"
