#!/usr/bin/env bash
# PROTOTYPE for issue 488. Never merge.
#
# Each scenario is a pull request against main, made in a scratch clone. For
# each one this prints whether Build reads what it changes (written by hand,
# from the trace in "Which tracked paths do the jobs of build.yml read?"), what
# the detector does, and what each rule says about its head.
#
#   run-scenarios.sh <scratch dir> <warm target dir> [scenario ...]
#
# A derived rule prints in brackets when the detector skipped Build, because
# in CI it runs inside Build and would never have run.
set -euo pipefail
here=$(dirname "$(realpath "$0")")
list="${here}/build-reads"
origin=$(git rev-parse --show-toplevel)
scratch=$1 warm=$2
shift 2

docs_edit() { echo "proto" >>docs/decisions/the-crate-carries-an-rlib.md; }
readme_edit() { echo "proto" >>README.md; }
build_rs() { printf 'fn main() {\n    println!("cargo:rerun-if-changed=build.rs");\n}\n' >build.rs; }
tests_dir() { mkdir tests && printf '//! proto\n\n#[test]\nfn proto() {}\n' >tests/proto.rs; }
shadow_pkg() { mkdir purba && echo 'raise SystemExit("shadow")' >purba/__init__.py; }
mise_confd() { mkdir -p .config/mise/conf.d && printf '[env]\nPROTO_488 = "1"\n' >.config/mise/conf.d/proto.toml; }
cargo_config() { mkdir .cargo && printf '[build]\nrustflags = ["--cfg", "proto488"]\n' >.cargo/config.toml; }
root_doc() { echo "# Security" >SECURITY.md; }
include_str_1() {
  echo "proto" >docs/agents/proto-488.md
  printf '\n/// proto\npub const PROTO_488: &str = include_str!("../docs/agents/proto-488.md");\n' >>src/lib.rs
}
include_str_2() { echo "proto again" >>docs/agents/proto-488.md; }
step_script_1() {
  printf '#!/usr/bin/env bash\necho proto\n' >.github/scripts/proto-488-step.sh
  chmod +x .github/scripts/proto-488-step.sh
  sed -i 's|^      - name: Run the Rust tests$|      - run: .github/scripts/proto-488-step.sh\n&|' .github/workflows/build.yml
}
step_script_2() { echo "echo again" >>.github/scripts/proto-488-step.sh; }

# name | the functions, one per pull request | does Build read what each changes
scenarios=(
  "docs-edit|docs_edit|no"
  "readme-edit|readme_edit|yes"
  "build-rs|build_rs|yes"
  "tests-dir|tests_dir|yes"
  "shadow-pkg|shadow_pkg|yes"
  "mise-confd|mise_confd|yes"
  "cargo-config|cargo_config|yes"
  "root-doc|root_doc|no"
  "include-str|include_str_1 include_str_2|yes yes"
  "step-script|step_script_1 step_script_2|yes yes"
)

printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' scenario pr reads detector classify readers depinfo strace
for row in "${scenarios[@]}"; do
  IFS='|' read -r name steps truths <<<"${row}"
  [[ $# -eq 0 ]] || [[ " $* " == *" ${name} "* ]] || continue
  dir="${scratch}/s-${name}"
  rm -rf "${dir}"
  git clone -q --shared --no-checkout "${origin}" "${dir}"
  git -C "${dir}" checkout -q --detach origin/main
  read -ra fns <<<"${steps}"
  read -ra truth <<<"${truths}"
  for i in "${!fns[@]}"; do
    base=$(git -C "${dir}" rev-parse HEAD)
    (cd "${dir}" && "${fns[$i]}" && git add -A && git -c user.name=p -c user.email=p@p commit -qm "proto ${fns[$i]}")
    detector=$(cd "${dir}" && "${here}/detect.sh" "${list}" "${base}" HEAD)
    classify=$(cd "${dir}" && "${here}/classify.sh" "${list}")
    readers=$(cd "${dir}" && scripts/check-readers.sh >/dev/null 2>&1 && echo pass || echo refuse)
    for mode in depinfo strace; do
      rm -rf "${scratch}/t-${mode}"
      cp -a --reflink=auto "${warm}/${mode}" "${scratch}/t-${mode}"
      verdict=$(cd "${dir}" && "${here}/derive.sh" "${mode}" "${list}" "${scratch}/t-${mode}" | head -1)
      verdict=${verdict%% (*}
      [[ "${detector}" = run ]] || verdict="[${verdict}]"
      declare "${mode}=${verdict}"
    done
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "${name}" "$((i + 1))" "${truth[$i]}" \
      "${detector}" "${classify}" "${readers}" "${depinfo}" "${strace}"
  done
  rm -rf "${dir}"
done
rm -rf "${scratch}/t-depinfo" "${scratch}/t-strace"
