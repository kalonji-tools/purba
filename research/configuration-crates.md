# Which crate resolves each value across the levels of configuration, and keeps its level

Research for [Which crate resolves each value across the levels of configuration, and keeps the level it came from?](https://github.com/kalonji-tools/purba/issues/1218)
Measured on 2026-11-23, against purba `main` at `33e89251f08c2d4170252ccefe21821a418cd309`, on one Linux machine, with cargo 1.99.0.
This file decides nothing. It vets the crates; the design of configuration scores relief.

## Summary

The forecast found 48 crates, not 41: the issue's table has 48 rows. Five of them do the need: they resolve one value across sources in an order, and they keep the source of the value. These are `config`, `figment`, `figment2`, `tier` and `star-toml`. Nine more crates merge layers but keep no level. Three keep a level, but only for the keys of one application: cargo, git or an agent host. The other 31 are keyword noise: INI and dotenv parsers, macOS bindings, graph layout, XDG paths.

`star-toml` fails the advisory check, so it is out. The other four pass the advisory and source checks. `tier` fails the licence gate on its own licence, `Apache-2.0`, so it stays on the list marked ⚖, as [#962](https://github.com/kalonji-tools/purba/issues/962) did for such crates.

I ran the four crates against the same six levels. Each crate meets the order and keeps each value's level, with some code from purba. No crate closes a level, applies the cap's lower value, or collects every error. purba writes those three rules in every case. The probe's shared file [`rules.rs`](configuration-crates/probes/rules.rs) does it in 56 lines, from the keys that each level sets. Only `tier` names the nearest dial, and it collects every unknown key across files. But it loses that check when one struct flattens the structs of two parts.

So a crate replaces the merge and the record of each value's level. In the prototype of #1213, that is about 35 lines. With `tier`, it also replaces the search for the nearest dial, which is about 20 more lines. The design scores relief. My judgement is that the code saved is below 100 lines for each crate.

## Method

| what | how |
|---|---|
| the crates and their levels | the table of the issue, measured by `mise run list:needs` with the cut at 2026-11-23T20:41:33Z |
| the fit of each crate | its crates.io description, and for each crate that merges layers, its source at the published version (the commit of its `.cargo_vcs_info.json`) |
| the rules | [`probes/run.sh`](configuration-crates/probes/run.sh), output in [`probes/run.out`](configuration-crates/probes/run.out) |
| the checks | [`probes/deny.sh`](configuration-crates/probes/deny.sh) runs purba's `.cargo/deny.toml` on the lock of each probe; output in [`probes/deny.out`](configuration-crates/probes/deny.out) |
| the trust facts | `https://crates.io/api/v1/crates/<name>/<version>` and `.../owners`, on 2026-11-23 |

Each probe reads the same levels, from [`probes/levels/`](configuration-crates/probes/levels/):

| level | sets |
|---|---|
| code | `timeout = 3`, `workers = 3` |
| the command line | `timeout = 1`, `cap = 50`, `workers = 1` |
| the environment | `PURBA_TIMEOUT=2`, `PURBA_WORKERS=2` |
| the project | `timeout = 4`, `cap = 0`, `workers = 4` |
| the user | `workers = 5`; in `user-closed.toml` also `timeout = 5` |
| the system | `workers = 6` |

`timeout` and `cap` change an outcome. `workers` does not. Under purba's rules, the result is `timeout = 3` from code, `cap = 0` from the project, `workers = 3` from code. The user's `timeout = 5` is an error. `typo.toml` holds `tmeout = 4`. `bad-project.toml` holds `tmeout = 4` and `cap = "many"`, and `bad-user.toml` holds `workrs = 5`: three faults in two files.

The probes use a flat project file. Reading `[tool.purba]` from `pyproject.toml` is unmeasured for each crate.

## 1. Fit, for each of the 48 crates

A crate fits when it resolves one value across several sources in an order, and keeps the source of that value.

### Fits: 5

| crate | version | why it fits | decides it |
|---|---|---|---|
| `config` | 0.15.27 | layered sources; each `Value` holds an `origin` | [`value.rs` L181-L219](https://github.com/rust-cli/config-rs/blob/f14a0442345bd63e84638e37aa13daa383f0ba54/src/value.rs#L181-L219) |
| `figment` | 0.10.19 | providers merged in order; each value has a `Tag` to its provider's `Metadata` | [`lib.rs` L163-L180](https://github.com/SergioBenitez/Figment/blob/dae213f0f31d764a0c00aeb39f145222050bc657/src/lib.rs#L163-L180) |
| `figment2` | 0.11.5 | a fork of `figment`; `diff -r` of `src/` shows only new array orders (`Zipjoin`, `Zipmerge`) and formatting | [`coalesce.rs` L10-L32](https://github.com/lmmx/figment2/blob/eb98729868dbb9a7c0d0081aafe9570e9f520aec/src/coalesce.rs#L10-L32) |
| `tier` | 0.1.21 | a fixed pipeline of layers; a trace of every step for each path | [`collect.rs` L16-L24](https://github.com/lvillis/tier-rs/blob/3b7e1b7b4ae805ae0102b6577edcbabeb56174da/crates/tier/src/loader/load/session/collect.rs#L16-L24), [`source.rs` L8-L21](https://github.com/lvillis/tier-rs/blob/3b7e1b7b4ae805ae0102b6577edcbabeb56174da/crates/tier/src/loader/source.rs#L8-L21) |
| `star-toml` | 26.7.3 | TOML layers merged in order; a `LayerReport` names the layer that wrote each leaf | [`reports.rs` L77-L106](https://github.com/seanchatmangpt/star-toml/blob/94d8a1b6406316e6d3c8fed4a56848ed498be9a1/src/reports.rs#L77-L106) |

### Merges layers, keeps no level: 9

| crate | version | why it does not fit | decides it |
|---|---|---|---|
| `confique` | 0.5.0 | joins partial structs with `Option::or`; keeps no source | [#1217's probe](https://github.com/kalonji-tools/purba/blob/82fadc8c4800dd3f8b79b30df0e1bfae1f6e944c/research/configuration-patterns.md?plain=1#L122) |
| `compote` | 0.3.1 | the merged state is one `Result<Value>`, with no source | [`compote.rs` L35-L37](https://docs.rs/crate/compote/0.3.1/source/src/compote.rs#35-37) (its forge is not GitHub) |
| `stratify` | 1.1.0 | `Store` caches one merged `Value`; getters return the value only | [`store.rs` L30-L58](https://github.com/LogicOcean/stratify/blob/d520fddbeada4b7bce85655fdd56e2dcaaa30c09/src/config/store.rs#L30-L58) |
| `envstack` | 1.0.1 | `merge` deep-merges layers into one `serde_json::Value` | [`lib.rs` L304-L325](https://github.com/WyattAu/envstack/blob/2313098f7d4b7c290eecac802a733b67cd1e20d0/src/lib.rs#L304-L325) |
| `partial_config` | 0.7.4 | `override_with` joins partial structs; `source` only logs the name | [`lib.rs` L110-L127](https://github.com/appetrosyan/partial_config/blob/1d4e36fac9f663460c08fa8e1083a7e4637413ce/src/lib.rs#L110-L127) |
| `rtb-config` | 0.6.6 | wraps `figment` and hides it; fixed levels: embedded default, user files, environment | [`config.rs` L232-L264](https://docs.rs/crate/rtb-config/0.6.6/source/src/config.rs#232-264) |
| `clap-config-fallback` | 0.3.0 | two levels: one file turned back into arguments, which clap parses again | [`lib.rs` L102-L113](https://github.com/alekece/clap-config-fallback/blob/175039db8900fed4cfcd7ec75c9b0cce4e04d84f/src/lib.rs#L102-L113) |
| `clap-config-fallback-derive` | 0.3.0 | the derive of the crate above | [docs.rs](https://docs.rs/clap-config-fallback-derive/0.3.0) |
| `optional_struct` | 0.5.2 | derives an `Option` struct and `apply`; no sources, no levels | [`lib.rs` L42-L54](https://github.com/lesurp/OptionalStruct/blob/9a60142787dc800bd925da5111b942981aa055f4/src/lib.rs#L42-L54) |

### Keeps a level, only for one application's keys: 3

These crates do the need for keys that they declare themselves. purba cannot declare its dials to them. That is a judgement.

| crate | version | the application | decides it |
|---|---|---|---|
| `cargo-config2` | 0.1.45 | cargo: `Value<T>` holds a `Definition`, on a fixed `Config` of cargo's keys | [`value.rs` L21-L27](https://github.com/taiki-e/cargo-config2/blob/239f3d74f1949baaa78f046d76ef0afd36e18685/src/value.rs#L21-L27), [`de.rs` L45](https://github.com/taiki-e/cargo-config2/blob/239f3d74f1949baaa78f046d76ef0afd36e18685/src/de.rs#L45) |
| `rightkit-agent-config` | 0.1.3 | an agent host: `merge_with_report` takes two `HostConfig` | [docs.rs](https://docs.rs/rightkit-agent-config/0.1.3/rightkit_agent_config/merge/fn.merge_with_report.html) (no VCS commit in the archive) |
| `gix-config` | 0.63.0 | git: git's syntax and git's own `Source` levels; it cannot read TOML | [`types.rs` L17](https://github.com/GitoxideLabs/gitoxide/blob/541b4db0a59644b617a8b706b83af23e0b17f7f3/gix-config/src/types.rs#L17) |

### Off-topic: 31

Each reason is the crate's crates.io description at its version.

| crate | version | what it does | link |
|---|---|---|---|
| `system-configuration` | 0.8.0 | bindings to macOS SystemConfiguration | [docs.rs](https://docs.rs/system-configuration/0.8.0) |
| `system-configuration-sys` | 0.6.0 | low-level bindings to the same | [docs.rs](https://docs.rs/system-configuration-sys/0.6.0) |
| `rust-ini` | 0.21.3 | INI parser | [docs.rs](https://docs.rs/rust-ini/0.21.3) |
| `xdg` | 3.0.0 | XDG base directory paths | [docs.rs](https://docs.rs/xdg/3.0.0) |
| `envconfig` | 0.11.1 | one struct from the environment only | [docs.rs](https://docs.rs/envconfig/0.11.1) |
| `envconfig_derive` | 0.11.1 | the derive of `envconfig` | [docs.rs](https://docs.rs/envconfig_derive/0.11.1) |
| `configparser` | 3.3.0 | INI parser | [docs.rs](https://docs.rs/configparser/3.3.0) |
| `zombienet-configuration` | 0.7.2 | builds a Polkadot test network's configuration | [docs.rs](https://docs.rs/zombienet-configuration/0.7.2) |
| `ini` | 2.0.0 | INI macro over `configparser` | [docs.rs](https://docs.rs/ini/2.0.0) |
| `docker-generate` | 0.1.3 | reads and writes Dockerfiles | [docs.rs](https://docs.rs/docker-generate/0.1.3) |
| `ini_core` | 0.2.0 | streaming INI parser | [docs.rs](https://docs.rs/ini_core/0.2.0) |
| `questdb-confstr` | 0.1.1 | QuestDB's configuration-string parser | [docs.rs](https://docs.rs/questdb-confstr/0.1.1) |
| `tini` | 1.3.0 | INI parser | [docs.rs](https://docs.rs/tini/1.3.0) |
| `cini` | 1.0.0 | callback INI parser | [docs.rs](https://docs.rs/cini/1.0.0) |
| `dotenv-parser` | 0.1.3 | `.env` parser | [docs.rs](https://docs.rs/dotenv-parser/0.1.3) |
| `dotenvy` | 0.15.7 | loads `.env` into the environment | [docs.rs](https://docs.rs/dotenvy/0.15.7) |
| `serde-saphyr` | 2.2.0 | YAML for serde | [docs.rs](https://docs.rs/serde-saphyr/2.2.0) |
| `dotenv` | 0.15.0 | loads `.env`; RUSTSEC-2021-0141 | [docs.rs](https://docs.rs/dotenv/0.15.0) |
| `kdl` | 6.7.1 | KDL document parser | [docs.rs](https://docs.rs/kdl/6.7.1) |
| `env-flags` | 0.1.1 | a macro that declares environment variables | [docs.rs](https://docs.rs/env-flags/0.1.1) |
| `dotenvs` | 0.2.2 | `.env` loader | [docs.rs](https://docs.rs/dotenvs/0.2.2) |
| `seatbelt` | 0.12.1 | retries and other recovery for fallible operations | [docs.rs](https://docs.rs/seatbelt/0.12.1) |
| `merman-elk-layered` | 0.11.0 | ELK layered graph layout | [docs.rs](https://docs.rs/merman-elk-layered/0.11.0) |
| `elkrs` | 0.1.1 | ELK graph layout | [docs.rs](https://docs.rs/elkrs/0.1.1) |
| `conmig` | 0.3.0 | renders ordered YAML layers into one file for another program; keeps no level for a value | [docs.rs](https://docs.rs/conmig/0.3.0) |
| `conmig_env` | 0.3.0 | an env-file schema for `conmig` | [docs.rs](https://docs.rs/conmig_env/0.3.0) |
| `conmig_caddyfile` | 0.3.0 | a Caddyfile schema for `conmig` | [docs.rs](https://docs.rs/conmig_caddyfile/0.3.0) |
| `phlee-spec` | 1.1.0 | constants of the `.phlee` format | [docs.rs](https://docs.rs/phlee-spec/1.1.0) |
| `phlee-core` | 1.1.0 | the `.phlee` document model | [docs.rs](https://docs.rs/phlee-core/1.1.0) |
| `phlee-render` | 1.1.0 | renders `.phlee` layers to HTML | [docs.rs](https://docs.rs/phlee-render/1.1.0) |
| `layered-crate` | 0.4.7 | checks the dependencies between modules of a crate | [docs.rs](https://docs.rs/layered-crate/0.4.7) |

### Found by #1217, not by the forecast

| crate | version | fit | decides it |
|---|---|---|---|
| `schematic` | 0.21.2 | merges partials; returns each partial with its `Source`, but no level for each value. Its environment wins over code and the command line | [#1217's probe](https://github.com/kalonji-tools/purba/blob/82fadc8c4800dd3f8b79b30df0e1bfae1f6e944c/research/configuration-patterns.md?plain=1#L123), [L18](https://github.com/kalonji-tools/purba/blob/82fadc8c4800dd3f8b79b30df0e1bfae1f6e944c/research/configuration-patterns.md?plain=1#L18). Not on the forecast's list, so not on the shortlist |

## 2. The rules, for each crate that fits

✓ out of the box. ⚠ with code that purba writes. ✗ impossible. "unmeasured" for no probe.

| rule | `config` 0.15.27 | `figment` 0.10.19 | `figment2` 0.11.5 | `tier` 0.1.21 | `star-toml` 26.7.3 |
|---|:-:|:-:|:-:|:-:|:-:|
| (a) six levels, code above the command line | ⚠ | ✓ | ✓ | ⚠ | unmeasured |
| (b) a closed level refuses an outcome dial | ⚠ | ⚠ | ⚠ | ⚠ | unmeasured |
| (c) the cap: the lower value applies | ⚠ | ⚠ | ⚠ | ⚠ | unmeasured |
| (d) each value keeps its level, readable | ⚠ | ✓ | ✓ | ✓ | unmeasured |
| (e) an unknown key names the nearest dial | ⚠ | ⚠ | ⚠ | ✓ | unmeasured |
| (f) every error is named before purba stops | ⚠ | ⚠ | ⚠ | ⚠ | unmeasured |
| (g) each part declares its dials; `purba-config` names no part | ⚠ | ⚠ | ⚠ | ⚠ | unmeasured |

No cell is ✗. Each ⚠ is code purba writes; the evidence follows. Each line quoted below is from [`probes/run.out`](configuration-crates/probes/run.out).

`star-toml` fails the advisory check (section 3), so no probe could move its verdict. Its source applies the environment "on top of everything" ([`loader.rs` L299](https://github.com/seanchatmangpt/star-toml/blob/94d8a1b6406316e6d3c8fed4a56848ed498be9a1/src/loader.rs#L299)), so rule (a) would need purba's code. That is read, not run.

### (a) The order

- **`config` ⚠, about 20 lines.** An override loses its `origin`, and every override beats every source ([#1217](https://github.com/kalonji-tools/purba/blob/82fadc8c4800dd3f8b79b30df0e1bfae1f6e944c/research/configuration-patterns.md?plain=1#L105)). So purba writes a `Source` for the command line and for code. Then the order of `add_source` holds: `up to code: workers = 3 from Some("code")` down to `up to the system: workers = 6`.
- **`figment` and `figment2` ✓.** The later `merge` wins. The same six lines come out: 3, 1, 2, 4, 5, 6.
- **`tier` ⚠, about 10 lines.** The order is fixed: defaults, files, custom layers, environment, arguments, typed arguments ([`collect.rs` L16-L24](https://github.com/lvillis/tier-rs/blob/3b7e1b7b4ae805ae0102b6577edcbabeb56174da/crates/tier/src/loader/load/session/collect.rs#L16-L24)). Its environment source reads `PURBA__TIMEOUT`, with two underscores, unless metadata names each variable. So the probe reads `PURBA_*` itself and hands the environment, the command line and code in as custom layers, in order. Then the six lines read 3, 1, 2, 4, 5, 6. Code as a typed patch after the arguments is unmeasured.

### (b) A closed level

No crate refuses the user's `timeout = 5`. Each one resolves `timeout = 3` from code, and says nothing. The user's value lost, so the winner's level cannot show it. purba must read the keys of each level apart.

- **`config`, `figment`, `figment2` ⚠.** Each source or provider can be asked for its own map: `Source::collect`, `Provider::data`. With that, `rules::refusals` gives `the user: `timeout` changes an outcome, and this level may not set it`. The rule is about 10 lines of `rules.rs`; reading each level apart is about 15 lines for each crate.
- **`tier` ⚠.** Its trace keeps every step, not only the winner, so the same rule reads the trace. `tier` also has a source policy for each path, but by the kind of source, and the user's file, the system's file and the project's file are all `File`. The probe made the policy work only by handing the project in as a custom layer: `refused: source file(levels/user-closed.toml) is not allowed to set `timeout``. That is a workaround, and the policy stops at the first violation.

### (c) The cap

Each crate gave `cap = 50` from the command line. None has a rule for the lower value: `tier`'s `MergeStrategy` is `Merge`, `Replace` or `Append` ([`merge.rs` L3-L11](https://github.com/lvillis/tier-rs/blob/3b7e1b7b4ae805ae0102b6577edcbabeb56174da/crates/tier/src/metadata/merge.rs#L3-L11)). purba's `rules::lower`, 7 lines, gave `Some((0, "the project"))` for each crate. #1217 found the one crate with such a rule, `schematic`, through a merge function.

### (d) The level of each value

- **`config` ⚠.** `get::<Value>` returns `origin: None` for every value in the probe. The public field `cache` keeps it: `timeout = 3 from Some("code")`. A file's origin is its path, so purba maps a path to a level.
- **`figment`, `figment2` ✓.** `find_metadata` names the provider. With purba's `Named` wrapper, 10 lines, the name is the level: `cap = "50" from "the command line"`. Without it, a file reads `TOML file` with its path ([#1217](https://github.com/kalonji-tools/purba/blob/82fadc8c4800dd3f8b79b30df0e1bfae1f6e944c/research/configuration-patterns.md?plain=1#L121)).
- **`tier` ✓.** `report().explain(path)` gives the winner and every step: `cap = Some(Number(50)) from Some("the command line"); every step: [("defaults", "null"), ("levels/project.toml", "0"), ("the command line", "50")]`.

### (e) An unknown key

- **`config`, `figment`, `figment2` ⚠, about 20 lines.** A struct with `deny_unknown_fields` refuses `tmeout`, and lists every field. purba's edit distance names `timeout`.
- **`tier` ✓.** `unknown field `tmeout` from file(levels/typo.toml); did you mean `timeout`?` ([`model.rs` L42-L54](https://github.com/lvillis/tier-rs/blob/3b7e1b7b4ae805ae0102b6577edcbabeb56174da/crates/tier/src/error/model.rs#L42-L54)). It holds only for a type that names each dial without `#[serde(flatten)]`; see (g).

### (f) Every error

Three faults in two files: `tmeout`, `cap = "many"`, `workrs`.

- **`config` ⚠.** One error. It names `tmeout` in some runs and `workrs` in others: in five runs of the same binary, three named `tmeout` and two named `workrs`. The order of its table is not fixed.
- **`figment`, `figment2` ⚠.** `three faults in two files gave 1 error(s)`, though `figment::Error` can chain several.
- **`tier` ⚠.** Both unknown keys, from both files, with a suggestion each. It did not reach `cap = "many"`. With a closed-level fault and an unknown key, it named only the closed-level fault.

`rules::refusals` collects every unknown key and closed level in one list. A type error is still one at a time in each crate. purba could read each dial alone, to collect those too; that is unmeasured.

### (g) Each part declares its dials

The probe has two parts: dispatch declares `timeout` and `workers`, reporting declares `cap`.

- **`config`, `figment`, `figment2` ⚠.** Each part takes its own struct from the shared tree: `Ok(DispatchLoose { timeout: Some(3), workers: Some(3) })`. A strict part struct refuses the other part's key: `unknown field `cap`, expected `timeout` or `workers``. So the refusal of an unknown key needs every part's names together. In the probe, command gathers the declarations, and `rules.rs` names no part.
- **`tier` ⚠.** A load takes one type. A JSON tree with each part's metadata accepted `tmeout`. A struct that flattens both parts' structs also accepted it: `composed struct with flatten, typo: accepted`. So `tier`'s check needs one struct that names every dial, in command, or purba's own check over the trace.

### What a crate replaces

The prototype of #1213 holds each value with its level in a `Set`, and resolves by the rank of the level: `Level`, its `rank` and `name`, and `resolve` ([`config.rs` L40-L76, L253-L266](https://github.com/kalonji-tools/purba/blob/3d6e12f45b79e2a8952c4c6eaab72e4dc456331e/crates/command/src/config.rs#L40-L76)). That is about 35 lines. Because it keeps every level's value, the closed levels and the cap read the same list.

Each crate replaces that merge, and keeps only what it needs for the winner. Rules (b), (c) and (f) then need each level's keys again: from `collect`, `data` or `tier`'s trace. The 56 lines of `rules.rs` stay with any crate. `tier` also replaces the edit distance, 13 lines in `rules.rs` and the prototype's `distance` ([L366](https://github.com/kalonji-tools/purba/blob/3d6e12f45b79e2a8952c4c6eaab72e4dc456331e/crates/command/src/config.rs#L366)).

## 3. Checks

`deny.sh` ran purba's `.cargo/deny.toml` on the lock of each probe. The `license-exception-not-encountered` findings are about purba's own exceptions and name no crate of the probe; the forecast drops such findings too.

| crate | advisory | licence | source | source of the result |
|---|---|---|---|---|
| `config` | pass | pass with the `toml` feature only; 3 rejected with default features: `encoding_rs`, `foldhash`, `tiny-keccak` | pass | [`deny.out`](configuration-crates/probes/deny.out); default features in the scratch run below |
| `figment` | pass | pass | pass | `deny.out` |
| `figment2` | pass | pass | pass | `deny.out` |
| `tier` | pass | 1 rejected: `tier` itself, `Apache-2.0` | pass | `deny.out` |
| `star-toml` | RUSTSEC-2026-0543, `chrono` 0.4.45 through `wasm4pm-compat` 26.8.7 | 3 rejected: `blake3`, `constant_time_eq`, `foldhash` | pass | `deny.out` |

The issue's table gives `config` "3 rejected"; the forecast built it with default features. A manifest with `config = "=0.15.27"` and default features gave the same three rejections. The command was `cargo-deny --config .cargo/deny.toml --manifest-path <scratch>/Cargo.toml check licenses`; that manifest is not kept.

The features also move blast area. `cargo tree -q -e normal -p <crate> --prefix none`, with `(*)` removed, each name once, and the crate itself left out, counts the crates each brings with the probe's features: `config` 7 (47 with default features), `figment` 22, `figment2` 16, `tier` 24. The issue's table gives `config` 3 and `figment` 1. The entries below keep the issue's levels; the design measures again with the features it picks.

**Trust is a person's check.** These are the facts to confirm, from crates.io on 2026-11-23.

| crate | version | repository | owners on crates.io | published by | published |
|---|---|---|---|---|---|
| `config` | 0.15.27 | https://github.com/rust-cli/config-rs | `filipegoncalves`, `matthiasbeyer`, `mehcode`, `epage` (users), `github:rust-cli:maintainers` (team) | `epage` | 2026-09-30 |
| `figment` | 0.10.19 | https://github.com/SergioBenitez/Figment | `SergioBenitez` | `SergioBenitez` | 2024-05-17 |
| `figment2` | 0.11.5 | https://github.com/lmmx/figment2 | `lmmx` | `lmmx` | 2026-04-20 |
| `tier` | 0.1.21 | https://github.com/lvillis/tier-rs | `forust` | `forust` | 2026-09-22 |

The `tier` publisher, `forust`, differs from the owner of its repository, `lvillis`. A person should confirm that both are the same project.

## 4. Conclusion

Four crates pass the advisory and source checks and do the need: `config`, `figment`, `figment2`, `tier` (⚖). `star-toml` is out on RUSTSEC-2026-0543. The other 43 do not do the need.

Two ways to close the issue follow. Each is a judgement for the person.

**Keep the four on the shortlist ★.** This follows #962: the issue vets the checks and the fit, and the design scores relief. The pull request writes these entries in `.config/shortlists.toml`, with the issue's levels:

```toml
[resolve-levels.config]
version = "0.15.27"
measured = "2026-11-23"
blast-area = 3
upkeep = 1
maintainers = 1
upstream-review = 3
tests = 1
unsafe = 1
api-churn = 2
owner-in-purba = 3
unmeasured = []
needs = 3

[resolve-levels.figment]
version = "0.10.19"
measured = "2026-11-23"
blast-area = 1
upkeep = 3
maintainers = 3
upstream-review = 3
tests = 2
unsafe = 1
api-churn = 2
owner-in-purba = 3
unmeasured = []
needs = 2

[resolve-levels.figment2]
version = "0.11.5"
measured = "2026-11-23"
blast-area = 1
upkeep = 1
maintainers = 3
upstream-review = 3
tests = 2
unsafe = 1
api-churn = 2
owner-in-purba = 3
unmeasured = []
needs = 2

[resolve-levels.tier]
version = "0.1.21"
measured = "2026-11-23"
blast-area = 2
upkeep = 1
maintainers = 3
upstream-review = 3
tests = 2
unsafe = 1
api-churn = 2
owner-in-purba = 3
unmeasured = ["upstream-review"]
needs = 3
```

Pain from the formula: `config` 2.36, `figment` 1.71, `figment2` 1.57, `tier` 2.07. The line in `.config/needs.toml` drops `issue`, and gets no `declined`, because a shortlisted line has entries and no `issue`.

| crate | the code it would replace | needs relief |
|---|---|---|
| `config` | the prototype's merge and level, about 35 lines; purba adds about 20 lines for the in-memory levels | 3 |
| `figment` | the same 35 lines; purba adds about 10 lines to name each level | 2 |
| `figment2` | the same as `figment` | 2 |
| `tier` ⚖ | the same 35 lines, and the nearest-dial search, about 20 lines; purba adds about 10 lines for the environment | 3 |

**Decline every crate.** This would follow from the judgement that relief cannot pass 1: each crate saves fewer than 100 lines, and serves no quality that the prototype misses. Relief 1 never joins. The line would then read, in `.config/needs.toml`:

```toml
declined = ["system-configuration", "system-configuration-sys", "rust-ini", "config", "figment", "xdg", "envconfig", "envconfig_derive", "configparser", "zombienet-configuration", "optional_struct", "ini", "docker-generate", "confique", "ini_core", "questdb-confstr", "tini", "figment2", "cini", "dotenv-parser", "dotenvy", "serde-saphyr", "gix-config", "dotenv", "cargo-config2", "kdl", "env-flags", "dotenvs", "clap-config-fallback-derive", "clap-config-fallback", "seatbelt", "merman-elk-layered", "star-toml", "envstack", "stratify", "compote", "elkrs", "rtb-config", "partial_config", "conmig", "tier", "rightkit-agent-config", "phlee-spec", "phlee-core", "phlee-render", "layered-crate", "conmig_env", "conmig_caddyfile"]
```

The record of the six levels says "A crate that passes its vet can replace that code". This research measured how little of that code a crate replaces. Whether that is enough is relief, which is the design's score.
