#!/usr/bin/env bash
# shellcheck disable=SC2312,SC2249,SC2181,SC2164,SC2028 # throwaway prototype
# PROTOTYPE for #664, question 1: where does the hand-off sit?
#
#   mise exec -- bash proto-664/probes.sh
#
# Applies each shape of the hand-off to a detached copy of this branch, then
# runs `cargo check` and the wiring gate (scripts/check-wiring.sh) on it.
#   callback        dispatch calls a closure once per test (this branch)
#   iterator        dispatch returns an iterator over the worker
#   trait+dep       dispatch declares a sink trait; command adds execution
#   trait+reexport  dispatch declares a sink trait and re-exports the types
#   reporting-sink  dispatch takes reporting's Report, the sink reporting defines
set -u

here=$(cd "$(dirname "$0")" && pwd)
top=$(git -C "${here}" rev-parse --show-toplevel)
work=${PROTO_WORK:-$(mktemp -d)}
export CARGO_TARGET_DIR=${work}/target

trait_in_dispatch() {
  cat >>crates/dispatch/src/lib.rs <<'EOF'

/// PROBE: a sink trait that dispatch declares.
pub trait Sink<T> {
    /// Takes one test's outcome or error.
    fn hand(&mut self, test: T, result: Result<Outcome, Error>) -> ControlFlow<()>;
}

/// PROBE: hands each test to `sink`.
pub fn hand_sink<T: Test>(modules: Vec<Module<T>>, sink: &mut impl Sink<T>) -> ControlFlow<()> {
    hand_each(modules, |test, result| sink.hand(test, result))
}
EOF
}

printer_in_command() { # $1 = the path command names the types by
  cat >>crates/command/src/lib.rs <<EOF

/// PROBE: command's sink.
pub struct Printer<W: Write>(pub purba_reporting::Report<W>);

impl<W: Write> purba_dispatch::Sink<TestId> for Printer<W> {
    fn hand(
        &mut self,
        test: TestId,
        result: Result<$1::Outcome, $1::Error>,
    ) -> ControlFlow<()> {
        match self.0.line(&test, &result) {
            Ok(()) => ControlFlow::Continue(()),
            Err(_) => ControlFlow::Break(()),
        }
    }
}
EOF
}

apply() {
  case $1 in
    callback) ;;
    iterator)
      cat >>crates/dispatch/src/lib.rs <<'EOF'

/// PROBE: an iterator over the worker.
pub fn hand_iter<T: Test>(
    modules: Vec<Module<T>>,
) -> impl Iterator<Item = (T, Result<Outcome, Error>)> {
    let mut queue = Vec::new();
    for module in modules {
        let executable = std::rc::Rc::new(purba_execution::Module::new(
            module.path,
            module.text,
            module.asserts,
        ));
        for test in module.tests {
            queue.push((std::rc::Rc::clone(&executable), test));
        }
    }
    let mut queue = queue.into_iter();
    purba_execution::attach(move |worker| {
        std::iter::from_fn(move || {
            let (executable, test) = queue.next()?;
            let result = worker.run(&executable, test.function());
            Some((test, result))
        })
    })
}
EOF
      ;;
    trait+dep)
      trait_in_dispatch
      printer_in_command purba_execution
      sed -i 's|^purba-dispatch = .*|&\npurba-execution = { path = "../execution" }|' \
        crates/command/Cargo.toml
      ;;
    trait+reexport)
      trait_in_dispatch
      echo 'pub use purba_execution::{Error as HandedError, Outcome as HandedOutcome};' \
        >>crates/dispatch/src/lib.rs
      printer_in_command purba_dispatch
      sed -i -e 's/purba_dispatch::Outcome/purba_dispatch::HandedOutcome/' \
        -e 's/purba_dispatch::Error/purba_dispatch::HandedError/' crates/command/src/lib.rs
      ;;
    reporting-sink)
      sed -i 's|^purba-execution = .*|&\npurba-reporting = { path = "../reporting" }|' \
        crates/dispatch/Cargo.toml
      cat >>crates/dispatch/src/lib.rs <<'EOF'

/// PROBE: the sink that reporting defines.
pub fn hand_report<T: Test + std::fmt::Display, W: std::io::Write>(
    modules: Vec<Module<T>>,
    report: &mut purba_reporting::Report<W>,
) -> ControlFlow<std::io::Error> {
    hand_each(modules, |test, result| match report.line(&test, &result) {
        Ok(()) => ControlFlow::Continue(()),
        Err(error) => ControlFlow::Break(error),
    })
}
EOF
      ;;
  esac
}

printf '%-15s %-10s %s\n' shape "cargo" "lint:wiring"
for shape in callback iterator trait+dep trait+reexport reporting-sink; do
  copy=${work}/copy-${shape}
  git -C "${top}" worktree add --quiet --detach "${copy}" HEAD
  # Carry this branch's uncommitted prototype too.
  git -C "${top}" diff HEAD | git -C "${copy}" apply --allow-empty
  (cd "${copy}" && apply "${shape}")
  (cd "${copy}" && cargo check --quiet --workspace >"${work}/${shape}.cargo.txt" 2>&1)
  cargo=$([[ $? == 0 ]] && echo builds || echo "refused")
  (cd "${copy}" && scripts/check-wiring.sh >"${work}/${shape}.wiring.txt" 2>&1)
  wiring=$?
  first=$(grep -m1 -v '^$' "${work}/${shape}.wiring.txt")
  wiring=$([[ ${wiring} == 0 ]] && echo passes || echo "refuses: ${first}")
  printf '%-15s %-10s %s\n' "${shape}" "${cargo}" "${wiring}"
  [[ ${cargo} == refused ]] && grep -m1 -E '^error' "${work}/${shape}.cargo.txt" |
    sed 's/^/                  rustc: /'
  git -C "${top}" worktree remove --force "${copy}"
done
echo "work dir: ${work}"
