# Each part is a crate

## Context and Problem Statement

[The landscape](../../LANDSCAPE.md#the-parts-and-their-dependencies) draws which part uses which.
One of its invariants is "the diagram is the only wiring".
[A gate refuses each invariant a tool can see, from the first code it reads](a-gate-refuses-each-invariant-a-tool-can-see.md#decision-outcome) gives that invariant a gate once each part has a boundary that a tool reads.
[Every part of the skeleton is Rust, inside the extension module](every-part-of-the-skeleton-is-rust-inside-the-extension-module.md#decision-outcome) names no crate.

| fact | source |
|---|---|
| reporting's `use` of a crate that its manifest did not name stopped at `E0432: unresolved import` | [How do three layouts of a part's boundary compare when the walking skeleton and purba's own tasks run on them?](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6168468953) |
| inside one crate, a `use` between two part modules that the diagram does not draw compiled | [How do three layouts of a part's boundary compare when the walking skeleton and purba's own tasks run on them?](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6168468953) |
| a grep for `crate::<part>` missed `use crate::{execution, selection};` | [How do three layouts of a part's boundary compare when the walking skeleton and purba's own tasks run on them?](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6168468953) |
| `cargo metadata` read each arrow that a part's manifest named | [How do three layouts of a part's boundary compare when the walking skeleton and purba's own tasks run on them?](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6168468953) |
| a `pub use` of another part's crate carried an arrow that neither the compiler nor `cargo metadata` saw | [How do three layouts of a part's boundary compare when the walking skeleton and purba's own tasks run on them?](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6168468953) |
| on Linux, only a test binary whose test called Python needed `libpython` | [How do three layouts of a part's boundary compare when the walking skeleton and purba's own tasks run on them?](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6168468953) |
| crates.io holds a crate under seven of the eight part names | [`selection`](https://crates.io/crates/selection), [`parsing`](https://crates.io/crates/parsing), [`dispatch`](https://crates.io/crates/dispatch), [`fixtures`](https://crates.io/crates/fixtures), [`assertion`](https://crates.io/crates/assertion), [`reporting`](https://crates.io/crates/reporting), [`command`](https://crates.io/crates/command) |

## Considered Options

- **One crate, with a module for each part.** Rejected. An arrow the diagram does not draw compiles. Each test needs `libpython`.
- **A cli, a lib and a core, with command in cli and the other parts as modules of core.** Rejected. Inside core, an arrow the diagram does not draw compiles. A grep misses a grouped `use`. Each test of core needs `libpython`.
- **PyO3 only in the crate that builds the extension module.** Rejected. Each part that reaches Python grows inside that one crate, the bridge that [purba divides by responsibility and not by language](purba-divides-by-responsibility-and-not-by-language.md#considered-options) rejects.
- **A crate named for its part alone.** Rejected. A crate on crates.io already holds most of those names.
- **A crate for each part, with PyO3 in each part that crosses the seam.** Chosen. The compiler refuses an arrow that a part's manifest does not name.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**Each part of purba is a crate, and its manifest names what it uses.**

| what | where |
|---|---|
| a part | the crate `purba-<part>` |
| its interface | the `pub` items of the crate's `src/lib.rs` |
| the parts it uses | the `purba-*` entries under `[dependencies]` in its `Cargo.toml` |
| PyO3 | a dependency of each part that [crosses the seam](the-skeleton-crosses-the-seam-in-execution-and-assertion.md#decision-outcome) |
| the crate that builds the extension module | `purba`, which holds `#[pymodule]` and `_main`, and uses `purba-command` alone |

```mermaid
flowchart TD
  purba --> purba-command
  purba-command --> purba-selection
  purba-command --> purba-dispatch
  purba-command --> purba-reporting
  purba-selection --> purba-parsing
  purba-assertion --> purba-parsing
  purba-dispatch --> purba-selection
  purba-dispatch --> purba-execution
  purba-execution --> purba-fixtures
  purba-execution --> purba-assertion
  purba-reporting --> purba-execution
```

- **A check compares the arrows of the manifests with the diagram.** It reads the `purba-*` edges from `cargo metadata`, and the arrows from the fence in [the landscape](../../LANDSCAPE.md#the-parts-and-their-dependencies).
- **The pull request that adds the second part builds the check.** It runs the check green and red, as [a gate refuses each invariant a tool can see, from the first code it reads](a-gate-refuses-each-invariant-a-tool-can-see.md#decision-outcome) requires.
- **This record binds no Python file.** The design in depth that first moves a part to Python decides how the boundary of that part shows.

**Downside:**

- **A part that re-exports another part's crate with `pub use` carries an arrow that no check sees.** The reviewer looks for it.
- **A type that moves from one part to another edits two manifests.**

## Confirmation

No command checks it, because purba holds no product code.
The slices of [Wayfinder: purba's walking skeleton, running every part on main](https://github.com/kalonji-tools/purba/issues/533) add the crates.
The slice that adds the second part adds the check.
