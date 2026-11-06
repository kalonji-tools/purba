# The zip alerts of check-prose are dismissed as unused

## Context and Problem Statement

Dependabot raises an alert on `zip` in `scripts/check-prose/Cargo.lock` for each advisory below.
Each one needs `zip` to read an archive from somebody else.
Only `zip` 9 fixes each one.

| fact | source |
|---|---|
| [GHSA-m665-v7pm-7grj](https://github.com/advisories/GHSA-m665-v7pm-7grj), critical, is fixed in 9.0.5 | the advisory |
| [GHSA-9hj8-73fh-9963](https://github.com/advisories/GHSA-9hj8-73fh-9963), [GHSA-hjqx-j4rw-g7xv](https://github.com/advisories/GHSA-hjqx-j4rw-g7xv) and [GHSA-q7v9-x93c-xqv3](https://github.com/advisories/GHSA-q7v9-x93c-xqv3), high, and [GHSA-8hxf-qr8p-62x4](https://github.com/advisories/GHSA-8hxf-qr8p-62x4), moderate, are fixed in 9.0.0 | the advisories |
| the lockfile holds `zip` 7.2.0 and 8.6.0, and `cargo tree -i zip@7.2.0 -e normal` prints nothing | the command of Confirmation, with `zip@7.2.0` |
| `zip` 7.2.0 comes from `candle-core`, whose `npy` module reads an archive | [`candle-core/src/npy.rs`](https://docs.rs/crate/candle-core/0.9.2/source/src/npy.rs) |
| `zip` 8.6.0 comes from `harper-core`, and no feature of `harper-core` drops it | [`harper-core/Cargo.toml`](https://github.com/Automattic/harper/blob/v2.3.0/harper-core/Cargo.toml#L37) |
| in `harper-core`, only the `weirpack` module calls `zip`, and only `harper-cli` and `harper-wasm` call that module | [`harper-core/src/weirpack/mod.rs`](https://github.com/Automattic/harper/blob/v2.3.0/harper-core/src/weirpack/mod.rs#L126) |
| `harper-core` 3.0.0 requires `zip` 8.6.0 | [`harper-core/Cargo.toml` at 3.0.0](https://github.com/Automattic/harper/blob/v3.0.0/harper-core/Cargo.toml) |
| a person reopens an alert that a person dismissed | [GitHub's page on Dependabot alerts](https://docs.github.com/en/code-security/how-tos/manage-security-alerts/manage-dependabot-alerts/view-dependabot-alerts) |

## Considered Options

- **Wait for a harper tag that requires `zip` 9.** No tag requires it, so the critical alert and the high ones stay open with no end.

## Decision Outcome

**Reach:** `scripts/check-prose/**`

**Area:** tooling

**A person dismisses the `zip` alerts on `scripts/check-prose/Cargo.lock` as `not_used`, while check-prose builds no `zip` 7.2.0 and calls no `weirpack`.**
check-prose never calls `weirpack`, the one `harper-core` module that reads an archive.

- **The dismissal waits for this record to merge.** Each dismissal comment names this record.
- **A new advisory on `zip` opens an alert that this record does not dismiss.**

A change in the reach runs this check:

1. For each `zip` in the lockfile, run the `cargo tree` command of Confirmation.
2. When the command prints a crate for `zip` 7.2.0, a person reopens the alerts.
3. Search the harper source at the tag of `check-prose.rs` for each caller of `weirpack`.
4. When a crate that check-prose builds calls `weirpack`, a person reopens the alerts.
5. When the lockfile holds no `zip` below 9.0.5, the same change deletes this record.

**Downside:**

- **Nothing refuses a change that breaks the condition.** The check rests on the person who reads this record.
- **The alerts page hides a dismissed alert.** It lists open alerts unless its filter changes.

## Confirmation

No gate checks this record.
`mise run list:records` names this record on each change to `scripts/check-prose/**`.
The Dependabot tab lists each alert in the table as dismissed.
Each dismissal comment names this record.

```bash
cargo -Zscript tree --locked --manifest-path scripts/check-prose/check-prose.rs \
  --config 'resolver.lockfile-path="scripts/check-prose/Cargo.lock"' -i zip@8.6.0 -e normal
```
