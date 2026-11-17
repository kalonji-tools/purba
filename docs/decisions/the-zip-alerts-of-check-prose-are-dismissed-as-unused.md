# The zip alerts of check-prose are dismissed as unused

## Context and Problem Statement

Dependabot raises an alert on `zip` in `scripts/check-prose/Cargo.lock` for each GitHub advisory below.
cargo-deny reports each RustSec advisory below.
Each one needs `zip` to read an archive from somebody else.
Only `zip` 9 fixes each one.

| fact | source |
|---|---|
| [GHSA-m665-v7pm-7grj](https://github.com/advisories/GHSA-m665-v7pm-7grj), critical, is fixed in 9.0.5 | the advisory |
| [GHSA-9hj8-73fh-9963](https://github.com/advisories/GHSA-9hj8-73fh-9963), [GHSA-hjqx-j4rw-g7xv](https://github.com/advisories/GHSA-hjqx-j4rw-g7xv) and [GHSA-q7v9-x93c-xqv3](https://github.com/advisories/GHSA-q7v9-x93c-xqv3), high, and [GHSA-8hxf-qr8p-62x4](https://github.com/advisories/GHSA-8hxf-qr8p-62x4), moderate, are fixed in 9.0.0 | the advisories |
| [RUSTSEC-2026-0367](https://github.com/rustsec/advisory-db/blob/4ef2df8/crates/zip/RUSTSEC-2026-0367.md), [RUSTSEC-2026-0368](https://github.com/rustsec/advisory-db/blob/4ef2df8/crates/zip/RUSTSEC-2026-0368.md), [RUSTSEC-2026-0369](https://github.com/rustsec/advisory-db/blob/4ef2df8/crates/zip/RUSTSEC-2026-0369.md) and [RUSTSEC-2026-0429](https://github.com/rustsec/advisory-db/blob/4ef2df8/crates/zip/RUSTSEC-2026-0429.md) are each in `extract`, and are fixed in 9.0.0 | the advisories |
| RUSTSEC-2026-0429 is GHSA-q7v9-x93c-xqv3; GitHub's advisory database holds no alias of the other three, so Dependabot raises no alert for them | the advisories, and the GitHub API |
| [RUSTSEC-2026-0545](https://github.com/rustsec/advisory-db/blob/233155b/crates/zip/RUSTSEC-2026-0545.md) is GHSA-8hxf-qr8p-62x4, [RUSTSEC-2026-0546](https://github.com/rustsec/advisory-db/blob/233155b/crates/zip/RUSTSEC-2026-0546.md) is GHSA-hjqx-j4rw-g7xv, [RUSTSEC-2026-0547](https://github.com/rustsec/advisory-db/blob/233155b/crates/zip/RUSTSEC-2026-0547.md) is GHSA-9hj8-73fh-9963, and [RUSTSEC-2026-0548](https://github.com/rustsec/advisory-db/blob/233155b/crates/zip/RUSTSEC-2026-0548.md) is GHSA-m665-v7pm-7grj | the advisories |
| the lockfile holds `zip` 7.2.0 and 8.6.0, and `cargo tree -i zip@7.2.0 -e normal` prints nothing | the command of Confirmation, with `zip@7.2.0` |
| `zip` 7.2.0 comes from `candle-core`, whose `npy` module reads an archive | [`candle-core/src/npy.rs`](https://docs.rs/crate/candle-core/0.9.2/source/src/npy.rs) |
| `zip` 8.6.0 comes from `harper-core`, and no feature of `harper-core` drops it | [`harper-core/Cargo.toml`](https://github.com/Automattic/harper/blob/v3.2.0/harper-core/Cargo.toml#L38) |
| in `harper-core`, only the `weirpack` module calls `zip`, and only `harper-cli` and `harper-wasm` call that module | [`harper-core/src/weirpack/mod.rs`](https://github.com/Automattic/harper/blob/v3.2.0/harper-core/src/weirpack/mod.rs#L126) |
| an alert dismissed by hand is reopened by hand | [GitHub's page on Dependabot alerts](https://docs.github.com/en/code-security/how-tos/manage-security-alerts/manage-dependabot-alerts/view-dependabot-alerts) |

## Considered Options

- **Wait for a harper version that requires `zip` 9.** No version requires it, so the critical alert and the high ones stay open with no end.

## Decision Outcome

**Reach:** `scripts/check-prose/**` `.cargo/check-prose.deny.toml`

**Area:** tooling

**The `zip` alerts and advisories of check-prose are dismissed as unused, while check-prose builds no `zip` 7.2.0 and calls no `weirpack`.**
check-prose never calls `weirpack`, the one `harper-core` module that reads an archive.
A person dismisses each alert by hand as `not_used`, and `.cargo/check-prose.deny.toml` ignores each advisory.

- **The dismissal waits for this record to merge.** Each dismissal comment names this record.
- **Each ignore states the condition in its reason.** cargo-deny refuses an ignore in `.cargo/check-prose.deny.toml` that matches no advisory.
- **This record dismisses no new advisory on `zip`.** Dependabot opens an alert for it, and `lint:crates` refuses it.

A change in the reach runs this check:

1. For each `zip` in the lockfile, run the `cargo tree` command of Confirmation.
2. When the command prints a crate for `zip` 7.2.0, reopen the alerts, and remove the `zip` ignores.
3. Search the harper source for each caller of `weirpack`, at the `harper-core` version in the lockfile.
4. When a crate that check-prose builds calls `weirpack`, reopen the alerts, and remove the `zip` ignores.
5. When the lockfile holds no `zip` below 9.0.5, the same change deletes this record and the `zip` ignores.

**Downside:**

- **Nothing refuses a change that breaks the condition.** The check rests on the reader of this record.
- **The alerts page hides a dismissed alert.** It lists open alerts unless its filter changes.

## Confirmation

No gate checks the condition of this record.
`mise run list:records` names this record on each change to `scripts/check-prose/**` or `.cargo/check-prose.deny.toml`.
The Dependabot tab lists each alert in the table as dismissed.
Each dismissal comment names this record.
`mise run lint:crates` refuses a `zip` advisory that no ignore names, and an ignore that matches no advisory.

```bash
cargo -Zscript tree --locked --manifest-path scripts/check-prose/check-prose.rs \
  --config 'resolver.lockfile-path="scripts/check-prose/Cargo.lock"' -i zip@8.6.0 -e normal
```
