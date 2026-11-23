# How Rust tools shape the code that reads and resolves configuration

Research for [How do Rust tools that read configuration from several levels shape the code that reads and resolves it, and which shape fits purba's configuration?](https://github.com/kalonji-tools/purba/issues/1217)
Measured on 2026-11-23, against purba `main` at `9e221a74`, on one Linux machine.
This file decides nothing. It weighs patterns, and the issue that vets the need line weighs crates.

## Summary

I read the configuration code of four tools, ruff, uv, cargo and cargo-nextest, and of four crates, `config`, `figment`, `confique` and `schematic`. I ran each tool and each crate against the same keys. I also ran purba's own `lint:wiring` check on seven shapes of a configuration crate.

Two shapes appear. In ruff and uv, the code that reads the files is a crate of its own, and it depends on the crates that declare the typed values. The crates that do the work never see it. The binary resolves each value and hands each consumer a typed struct. In cargo and cargo-nextest, one module holds the mechanism and nearly every value, and a consumer asks it for a value. In cargo-nextest, that module also imports the types of its consumers, so the two depend on each other. That is legal inside one crate, and Cargo refuses it between two crates.

The four crates hold only the mechanism. The application declares every value. Each crate states the order of the levels as the order in which code adds each source. None of them meets all six agreed rules without code of purba's own. The gaps are these:

- **No tool and no crate names the nearest key** when it refuses an unknown one (rule 6). Each one that refuses lists every known key, or names none. purba's prototype already holds an edit distance.
- **Only `schematic` has a rule of a value's own** (rule 2's cap) out of the box. A merge function on one field gave the lower value, 0, over the command line's 50. uv reaches the same with a `Combine` impl for each type.
- **No crate closes a level to a value** (rule 3). uv, cargo and cargo-nextest each do it in code written by hand.
- **`schematic` puts the environment above every other source.** In the probe, `PURBA_TIMEOUT=2` won over the command line's 1 and code's 3. That breaks rule 5 unless purba reads the environment itself.
- **`config` puts every override above every source.** Code added as a source lost to the command line, so rule 4 needs code as an override too.

Every mechanism I read states one total order. The issue first listed the environment above code, which formed a cycle with rules 4 and 5 for a dial that code sets. The issue now lists code, then the command line, then the environment, as #1213's prototype ranked them under `code-wins`, and the cycle is gone.

`lint:wiring` decides much of the choice of home. A crate of the concern may use no `purba-*` crate at all, purba-errors included. So a configuration crate cannot use selection's parse to read the code level. It cannot return purba-errors' invocation error either. A part may use it only when the concern's line names that part, and execution is not named. Each of these is a run of the check, in section 9.

## How to read this file

- **[run]** means I ran it in this session, on the version named. **[source]** means I read it in the code or the documentation at the pinned commit, and did not run it.
- Every link to code is pinned to a full commit. The Method section names each one.
- A **level** is one place a value comes from: the command line, the environment, code, the project's file, the user's file or the system's file.
- A **source** is the unit a crate reads: a file, the environment, or a value set in code.
- A **consumer** is the code that uses a resolved value. In purba, it is a part.
- A **home** is one of the four places that the issue asks me to sketch for purba's configuration.
- A **mod stands for a crate** in each sketch under [`configuration-patterns/sketches/`](configuration-patterns/sketches/).

## Method

### Versions and sources

| tool or crate | version | what I ran | commit |
|---|---|---|---|
| ruff | 0.17.6 | `uvx ruff@0.17.6` | [`a7ba68b4919e56b6af949a42b2da38fe4eec67a9`](https://github.com/astral-sh/ruff/tree/a7ba68b4919e56b6af949a42b2da38fe4eec67a9) (tag `0.17.6`) |
| uv | 0.13.7 | on `PATH` | [`316c827e058c16b71d5c507a20729c2995a7bcce`](https://github.com/astral-sh/uv/tree/316c827e058c16b71d5c507a20729c2995a7bcce) (tag `0.13.7`) |
| cargo | 1.99.0 (`5f94df478`) | on `PATH` | [`5f94df4789f005f9a352888e8355ffc645b7ed0e`](https://github.com/rust-lang/cargo/tree/5f94df4789f005f9a352888e8355ffc645b7ed0e) (tag `0.100.0`) |
| cargo-nextest | 0.9.149 | on `PATH` | [`3fe4ef31972f24e0c94cfe291718eab0fe678260`](https://github.com/nextest-rs/nextest/tree/3fe4ef31972f24e0c94cfe291718eab0fe678260) (tag `cargo-nextest-0.9.149`) |
| `config` | 0.15.27 | a probe binary | [`f14a0442345bd63e84638e37aa13daa383f0ba54`](https://github.com/rust-cli/config-rs/tree/f14a0442345bd63e84638e37aa13daa383f0ba54) (tag `v0.15.27`) |
| `figment` | 0.10.19 | a probe binary | [`dae213f0f31d764a0c00aeb39f145222050bc657`](https://github.com/SergioBenitez/Figment/tree/dae213f0f31d764a0c00aeb39f145222050bc657) (no tag; the commit that `.cargo_vcs_info.json` of the published crate names) |
| `confique` | 0.5.0 | a probe binary | [`39179571813e6aa42b010b82b2bee433b5f6446b`](https://github.com/LukasKalbertodt/confique/tree/39179571813e6aa42b010b82b2bee433b5f6446b) (tag `v0.5.0`) |
| `schematic` | 0.21.2 | a probe binary | [`5b426874cea1b4569089040ea4553f6e27871b0b`](https://github.com/moonrepo/schematic/tree/5b426874cea1b4569089040ea4553f6e27871b0b) (tag `schematic-v0.21.2`) |

Each crate's version is the newest stable one on crates.io on 2026-11-23. Figment has no tag for 0.10.19. I fetched the commit that the published crate names, and `diff -r` found no difference in `src/` between that commit and the crate's archive.

The `ruff` on `PATH` was 0.17.4, so I ran 0.17.6 through `uvx` instead.

For purba, I read `main` at [`9e221a7452f9f8ec97a35706434599f676b664ab`](https://github.com/kalonji-tools/purba/tree/9e221a7452f9f8ec97a35706434599f676b664ab), the prototype at [`fedc6c127d5890d4a1168faf869f2cf880bcccc8`](https://github.com/kalonji-tools/purba/blob/fedc6c127d5890d4a1168faf869f2cf880bcccc8/crates/command/src/config.rs), and its results at [`3d6e12f45b79e2a8952c4c6eaab72e4dc456331e`](https://github.com/kalonji-tools/purba/blob/3d6e12f45b79e2a8952c4c6eaab72e4dc456331e/proto/1213/RESULTS.md).

### Probes

Each probe and its output sit under [`configuration-patterns/probes/`](configuration-patterns/probes/).

- [`tools.sh`](configuration-patterns/probes/tools.sh) runs ruff, uv, cargo and cargo-nextest on an unknown key and on a closed level. Its output is [`tools.out`](configuration-patterns/probes/tools.out).
- [`crates/`](configuration-patterns/probes/crates/) holds one binary for each crate. Each one reads the same six levels: the command line sets `timeout = 1` and `cap = 50`, `PURBA_TIMEOUT=2`, code sets `timeout = 3`, the project sets `timeout = 4` and `cap = 0`, the user sets `timeout = 5` and `workers = 8`, and the system sets `timeout = 6`. A seventh file holds `tmeout = 4`. The output is [`crates.out`](configuration-patterns/probes/crates.out). I built them with cargo 1.99.0.
- [`wiring.sh`](configuration-patterns/probes/wiring.sh) runs purba's `scripts/check-wiring.sh` on seven shapes, in a scratch clone of `main` at `9e221a74`. The output is [`wiring.out`](configuration-patterns/probes/wiring.out).
- The four sketches under [`sketches/`](configuration-patterns/sketches/) each compile alone with `rustc 1.99.0 --edition 2024 --crate-type lib -D warnings` [run]. They compile as sketches only. They are not a build of purba.

The dial names in the probes are short (`cap`, not `suppression-cap`), so that one key reads the same in a file and in a variable.

## 1. Where the code lives, and what it depends on

| tool or crate | where the code that reads and resolves lives | what it depends on | who depends on it |
|---|---|---|---|
| ruff | crate `ruff_workspace`: `options.rs` → `configuration.rs` → `settings.rs` | the crates whose settings it builds: `ruff_linter`, `ruff_python_formatter`, `ruff_graph` ([`Cargo.toml` L19-L28](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_workspace/Cargo.toml#L19-L28)) | the binary `ruff` and `ruff_server`. `ruff_linter` does not depend on it. `LinterSettings` lives in `ruff_linter` ([`settings/mod.rs` L213](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_linter/src/settings/mod.rs#L213)) [source] |
| uv | crate `uv-settings` for the files and their merge; `uv-cli/src/settings.rs` for the resolve with the command line and the environment | the crates that declare the value types: `uv-configuration`, `uv-distribution-types`, `uv-install-wheel`, `uv-torch`, `uv-workspace` and 17 more | `uv`, `uv-cli` and the `uv-*-commands` crates. `uv-resolver`, `uv-installer`, `uv-client`, `uv-build-frontend`, `uv-workspace` and `uv-configuration` do not depend on it [source] |
| cargo | module `context` of the one crate `cargo`: `GlobalContext`, `ConfigValue`, `Definition`, and `schema.rs` | the rest of the crate | every module that calls `gctx.get` [source] |
| cargo-nextest | module `config` of the crate `nextest-runner`; the crate `config` reads each file | the crate `config` 0.15.27, `serde_ignored`, and modules of its own consumers: `reporter::TestOutputDisplay`, `list::TestList` ([`core/imp.rs` L35-L38](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/nextest-runner/src/config/core/imp.rs#L35-L38)) | `runner` and the other modules of `nextest-runner`, and the binary crate `cargo-nextest`, which applies the command line [source] |
| `config` | the crate | serde and a format parser for each feature | the application |
| `figment` | the crate | serde, `uncased`, and `toml` 0.8 for the TOML feature | the application, or a library that declares its own `Config` |
| `confique` | the crate and its derive | serde, and `toml` 1.2 for the TOML feature | the application |
| `schematic` | the crate `schematic`, its derive in `schematic_macros`, and `schematic_core` | serde, `garde` for validation, and a format parser for each feature | the application |

ruff and uv cross a crate boundary between the reader and the work. In both, the reader points at the types of the work, never the reverse [source]. cargo-nextest points both ways, inside one crate [source]. A Cargo manifest cannot point both ways between two crates, so that shape does not move into separate crates as it is.

## 2. The stages a value passes through

| tool or crate | stages |
|---|---|
| ruff | `Options`, the raw table of one file → `Configuration`, every field an `Option`, merged along `extend` → the command line applied by a `ConfigurationTransformer` → `Settings`, which holds `LinterSettings`, `FormatterSettings` and `AnalyzeSettings` ([`configuration.rs` L175-L214](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_workspace/src/configuration.rs#L175-L214)) [source] |
| uv | `Options`, the raw table → `FilesystemOptions`, the project's file combined with the user's and the system's ([`uv/src/lib.rs` L337-L347](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv/src/lib.rs#L337-L347)) → a `*Settings` struct for each command, each field resolved from the flag, the environment and the files ([`uv-cli/src/settings.rs` L100-L156](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-cli/src/settings.rs#L100-L156)) [source] |
| cargo | each file → one tree of `ConfigValue`, each node with its `Definition` → the command line's `--config` merged with force → a typed struct per key, deserialized on demand, with the environment read at that moment ([`mod.rs` L2099-L2106](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/context/mod.rs#L2099-L2106)) [source] |
| cargo-nextest | each file deserialized alone, for its warnings → every file merged by the crate `config` → `NextestConfig` → an `EvaluatableProfile` for the chosen profile → `TestSettings` for one test ([`core/imp.rs` L396-L428](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/nextest-runner/src/config/core/imp.rs#L396-L428)) [source] |
| `config` | each source → one tree of `Value`, each with an `origin` → a typed struct, or one value by key [source] |
| `figment` | each provider → one `Dict` of `Value`, each with a `Tag` that names its `Metadata` → a typed struct, or one value by key [source] |
| `confique` | each source → a derived partial struct, every field an `Option` → the partials joined by `with_fallback` → the typed struct ([`lib.rs` L442-L469](https://github.com/LukasKalbertodt/confique/blob/39179571813e6aa42b010b82b2bee433b5f6446b/src/lib.rs#L442-L469)) [source] |
| `schematic` | each source → a derived `Partial*` struct → the partials merged, field by field → the typed struct, with the list of partials and their sources ([`loader.rs` L23-L33](https://github.com/moonrepo/schematic/blob/5b426874cea1b4569089040ea4553f6e27871b0b/crates/schematic/src/config/loader.rs#L23-L33)) [source] |

Each one has a partial stage and a final typed stage. The tools add a third stage: the settings that one consumer reads. #1213's prototype has two stages: a list of `Set`s, then a lookup by dial name.

## 3. How the order of the levels is stated

| tool or crate | how the order is stated |
|---|---|
| ruff | by hand. `resolve` tries a file named on the command line, then the nearest project file, then the user's file, and takes the first it finds ([`resolve.rs` L19-L98](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff/src/resolve.rs#L19-L98)). The derive `CombineOptions` writes `self.x.or(other.x)` for each field ([`combine_options.rs` L53](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_macros/src/combine_options.rs#L53)). The command line wins through `transform` ([`args.rs` L780-L784](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff/src/args.rs#L780-L784)) [source] |
| uv | a trait for each type, `Combine`, "preferring the values in `self`" ([`combine.rs` L23-L31](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-settings/src/combine.rs#L23-L31)), a derive for structs, and a chain written by hand at each field: `flag(…).combine(env(…)).combine(workspace…)` ([`settings.rs` L148-L156](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-cli/src/settings.rs#L148-L156)) [source] |
| cargo | an `Ord` on `Definition`: command line over environment over file over built-in ([`value.rs` L118-L128](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/context/value.rs#L118-L128)). Between two files, the order of the walk decides [source] |
| cargo-nextest | the order of the `config` builder's sources for the files; a macro, `profile_field!`, for the chain of profiles ([`core/imp.rs` L923-L931](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/nextest-runner/src/config/core/imp.rs#L923-L931)); and by hand for the command line ([`run.rs` L251-L252](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/cargo-nextest/src/dispatch/core/run.rs#L251-L252)) [source] |
| `config` | the order of `add_source`, the later one winning. Defaults always go first and overrides always go last ([`builder.rs` L13-L15](https://github.com/rust-cli/config-rs/blob/f14a0442345bd63e84638e37aa13daa383f0ba54/src/builder.rs#L13-L15)) [source]. Code added as a source after the command line's override lost: `timeout = 1` [run] |
| `figment` | the order of `merge` (the later one wins) or `join` (the earlier one wins) ([`figment.rs` L210](https://github.com/SergioBenitez/Figment/blob/dae213f0f31d764a0c00aeb39f145222050bc657/src/figment.rs#L210), [L292](https://github.com/SergioBenitez/Figment/blob/dae213f0f31d764a0c00aeb39f145222050bc657/src/figment.rs#L292)) [source]. Code merged last won: `timeout = 3` [run] |
| `confique` | the order of the builder's sources: "Sources specified earlier have a higher priority" ([`lib.rs` L466](https://github.com/LukasKalbertodt/confique/blob/39179571813e6aa42b010b82b2bee433b5f6446b/src/lib.rs#L466)) [source]. Code added first won: `timeout = 3` [run] |
| `schematic` | the order of the loader's sources, the later one winning. Defaults go first and the environment goes last ([`loader.rs` L219-L235](https://github.com/moonrepo/schematic/blob/5b426874cea1b4569089040ea4553f6e27871b0b/crates/schematic/src/config/loader.rs#L219-L235)) [source]. `PURBA_TIMEOUT=2` won over code and the command line [run] |

Every one of them states a total order. None states an order for each dial. ruff and uv come closest: each field's line of code can chain its own levels.

## 4. How a value keeps the level it came from

| tool or crate | what it keeps |
|---|---|
| ruff | the level of a rule selector only, as a `RangedValue` with a `ValueSource` ([`rule_selector.rs` L23](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_linter/src/rule_selector.rs#L23), [`ruff_ranged_value` L19-L44](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_ranged_value/src/lib.rs#L19-L44)). No other setting keeps one [source] |
| uv | the level of an index only, as an `Origin`: `Cli`, `User`, `System`, `Project` or `RequirementsTxt` ([`with_origin`, `settings.rs` L213](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-settings/src/settings.rs#L213)) [source] |
| cargo | every value, as a `Definition`: `BuiltIn`, `Path(file)`, `Environment(variable)` or `Cli` ([`value.rs` L74-L83](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/context/value.rs#L74-L83)). A consumer can ask for `Value<T>`, which carries it ([`value.rs` L55-L61](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/context/value.rs#L55-L61)). A user's file and a project's file differ only by path [source] |
| cargo-nextest | for each test's setting, whether it came from a default, the profile or an override, as a type parameter `Source` that "isn't used by any public APIs at the moment" ([`overrides/imp.rs` L97-L120](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/nextest-runner/src/config/overrides/imp.rs#L97-L120)) [source] |
| `config` | an `origin` string for each value: a file's path, or "the environment" ([`value.rs` L181-L219](https://github.com/rust-cli/config-rs/blob/f14a0442345bd63e84638e37aa13daa383f0ba54/src/value.rs#L181-L219), [`env.rs` L240](https://github.com/rust-cli/config-rs/blob/f14a0442345bd63e84638e37aa13daa383f0ba54/src/env.rs#L240)) [source]. A value from the user's file read `Some("levels/user.toml")`. An override read `None` [run] |
| `figment` | a `Tag` on each value, which names its provider's `Metadata`: a name, a source path, and the line of code that added the provider ([`lib.rs` L163-L180](https://github.com/SergioBenitez/Figment/blob/dae213f0f31d764a0c00aeb39f145222050bc657/src/lib.rs#L163-L180)). A consumer field `Tagged<T>` keeps it [source]. `find_metadata("timeout")` named the probe's line 34, where code set it [run] |
| `confique` | nothing [source, run] |
| `schematic` | the list of partials, each with its `Source`, returned beside the config. It holds no level for each value, but a reader can find the last partial that sets one [source, run] |

Only cargo, `config` and `figment` keep a level for every value. purba's prototype keeps one for every value, in `Set::level`.

## 5. How a consumer gets its values, and who declares them

| tool or crate | how a consumer gets a value | who declares its name and type |
|---|---|---|
| ruff | a typed struct handed to it: `LinterSettings` reaches the linter as an argument [source] | the names: `ruff_workspace`'s `Options`. The resolved types: the consumer's crate (`LinterSettings` in `ruff_linter`) [source] |
| uv | a typed struct handed to it: `ResolverSettings`, `InstallerSettingsRef` [source] | the names: `uv-settings`. The types: the crates that do the work (`IndexStrategy` in `uv-configuration`) [source] |
| cargo | a lookup by key, deserialized into a type: `gctx.get::<CargoNewConfig>("cargo-new")` ([`cargo_new.rs` L778](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/ops/cargo_new.rs#L778)), or a cached getter such as `build_config()` ([`mod.rs` L1964-L1967](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/context/mod.rs#L1964-L1967)) [source] | mostly `context/schema.rs` (`CargoBuildConfig`, [L200](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/context/schema.rs#L200)). Some consumers declare their own: `CargoNewConfig` lives in `ops/cargo_new.rs` ([L136](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/ops/cargo_new.rs#L136)) [source] |
| cargo-nextest | a typed getter on the profile: `profile.retries()` ([`core/imp.rs` L1133-L1135](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/nextest-runner/src/config/core/imp.rs#L1133-L1135)) [source] | the module `config`, in `config/elements/` [source] |
| `config` | either: `try_deserialize::<T>()` or `get::<T>("key")` [source, run] | the application |
| `figment` | either: `extract::<T>()` or `extract_inner::<T>("key")` [source, run] | the application, or each library for itself ("For Library Authors", [`lib.rs` L349-L415](https://github.com/SergioBenitez/Figment/blob/dae213f0f31d764a0c00aeb39f145222050bc657/src/lib.rs#L349-L415)) [source] |
| `confique` | a typed struct only [source] | the application, with `#[derive(Config)]` |
| `schematic` | a typed struct only [source] | the application, with `#[derive(Config)]`; a struct may nest another with `#[setting(nested)]` |

figment's guide is the one place where each library declares its own values to a shared mechanism. Each library extracts its own `Config` from the one `Figment` that the framework exposes [source].

## 6. How a key that no value owns is refused

| tool or crate | an unknown key in a file | names the nearest key |
|---|---|---|
| ruff | an error: ``unknown field `line-lnegth` ``, exit 2 [run] | no; it names no other key [run] |
| uv | in `uv.toml`, an error that lists every field, exit 2. In `[tool.uv]` of a `pyproject.toml`, a warning, and the run goes on with exit 0 ([`lib.rs` L106-L127](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-settings/src/lib.rs#L106-L127)) [run] | no; it lists all 79 [run] |
| cargo | a warning, ``unused config key `build.jbos` ``, only for a table that some code deserializes into a struct ([`de.rs` L291-L307](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/context/de.rs#L291-L307)). An unknown table, `[bluid]`, is silent. Exit 0 [run] | no [run] |
| cargo-nextest | a warning, `ignoring unknown configuration key: profile.default.retires`, through `serde_ignored` ([`core/imp.rs` L864-L866](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/nextest-runner/src/config/core/imp.rs#L864-L866)). Exit 0 [run] | no [run] |
| `config` | an error from serde, only when the target struct has `#[serde(deny_unknown_fields)]`. A lookup by key cannot see an unknown key [run] | no; it lists every field [run] |
| `figment` | the same, with the file in the error: `for key "default.tmeout" in levels/typo.toml TOML file` [run] | no; it lists every field [run] |
| `confique` | accepted in silence by default. Refused only with `#[config(layer_attr(serde(deny_unknown_fields)))]` ([`gen/mod.rs` L110-L116](https://github.com/LukasKalbertodt/confique/blob/39179571813e6aa42b010b82b2bee433b5f6446b/macro/src/gen/mod.rs#L110-L116)) [run] | no; it lists every field [run] |
| `schematic` | refused by default; `#[config(allow_unknown_fields)]` accepts it ([`container.rs` L371-L421](https://github.com/moonrepo/schematic/blob/5b426874cea1b4569089040ea4553f6e27871b0b/crates/core/src/container.rs#L371-L421)) [source, run] | no; it lists every field [run] |

Rule 6 asks for the nearest dial. Every one of the eight needs purba's own code for that.

## 7. A rule of a value's own, and a level closed to a value

| tool or crate | a rule of one value's own (rule 2's cap) | a level closed to a value (rule 3) |
|---|---|---|
| ruff | possible: `Configuration::combine` is written by hand for each field ([`configuration.rs` L618](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_workspace/src/configuration.rs#L618)) [source] | none found [source] |
| uv | a `Combine` impl for one type; a value with its own rule needs a type of its own. Most types use `impl_combine_or!`, which is `self.or(other)` ([`combine.rs` L79-L87](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-settings/src/combine.rs#L79-L87)) [source] | by hand: `validate_uv_toml` refuses 12 fields of a project in any `uv.toml` ([`lib.rs` L296](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-settings/src/lib.rs#L296)). `sources` in the user's file stopped the run with exit 2 [run] |
| cargo | none in the merge; a consumer can read each level's value apart [source] | by hand: `registry.token cannot be set through --config` ([`mod.rs` L1579](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/context/mod.rs#L1579)), exit 101 [run] |
| cargo-nextest | none [source] | by type: the user's file deserializes into `DeserializedUserConfig`, which has no `profile` ([`user_config/imp.rs` L192-L208](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/nextest-runner/src/user_config/imp.rs#L192-L208)). `retries` there is ignored with a warning, exit 0 [run] |
| `config` | none [source] | none; a reader can compare each value's `origin` [source] |
| `figment` | none. `join` keeps the earlier level, not the lower value: the probe gave `cap = 50` [run] | none; a reader can ask `find_metadata` for the winner's source [source] |
| `confique` | none; a field's `validate` judges one value, not two [source] | none, and no level is kept to check it by [source] |
| `schematic` | a field's `merge` function: `fn(prev, next, &context)` ([`merge.rs` L8-L20](https://github.com/moonrepo/schematic/blob/5b426874cea1b4569089040ea4553f6e27871b0b/crates/schematic/src/merge.rs#L8-L20)). `merge = lower` gave `cap = 0` over the command line's 50 [run] | none; the merge function is not told the level. A reader can scan the list of partials for the user's and the system's file [source] |

## 8. The six rules, tool by tool

"Out of the box" means that the tool or crate does it with no code of the application's own. "Own code" means that purba writes the rule. Each pointer leads to a row above.

**Rule 1. A dial that loosens a check is refused. A switch that touches no check is allowed.**

| meets it out of the box | needs own code |
|---|---|
| none | all eight. No tool or crate has a notion of a check. A validator on one value can refuse a value that loosens one, such as a timeout of 0: `confique` (`validate`) and `schematic` (`validate`) offer one for each field [source]. A type can do it too: `NonZeroU32` in the sketch of home 1 |

**Rule 2. The higher level overwrites the lower one. The cap keeps its own rule: the lower value applies.**

| meets it out of the box | needs own code |
|---|---|
| `schematic`, with a `merge` function on the cap's field (section 7) [run] | ruff (a line in `combine`), uv (a type and its `Combine` impl), cargo, cargo-nextest, `config`, `figment`, `confique` (read the value of each level apart, then take the lower). Every one meets the first sentence [source, run] |

**Rule 3. The files of the user and of the system take only the dials that change no outcome.**

| meets it out of the box | needs own code |
|---|---|
| none of the four crates | uv, cargo and cargo-nextest each wrote it by hand (section 7) [run]. For the crates, purba writes it: compare `origin` (`config`), ask `find_metadata` (`figment`), scan the partials (`schematic`), or derive a second, narrower struct for those files (`confique`, as cargo-nextest does) |

**Rule 4. A value in code beats the command line.**

| meets it out of the box | needs own code |
|---|---|
| `figment` and `confique`: code goes on top of the order [run] | `config`: code must be an override, set after the command line's [run]. `schematic`: the environment still wins over code [run]. ruff, uv, cargo and cargo-nextest read no configuration from code. ruff's `# noqa` is a suppression, which its configuration code does not read [source] |

**Rule 5. The environment is a level below the command line.**

| meets it out of the box | needs own code |
|---|---|
| uv, cargo (`Definition`), cargo-nextest (clap's `env =`, [`run.rs` L158](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/cargo-nextest/src/dispatch/core/run.rs#L158)), `config`, `figment`, `confique` [source]. ruff reads only four `RUFF_*` variables, for output and cache [source] | `schematic`: its environment goes above every source [run]. purba would leave `#[setting(env)]` out, and add the environment as a source of its own |

**Rule 6. A key that no dial owns is an invocation error, which names the nearest dial.**

| meets it out of the box | needs own code |
|---|---|
| none | all eight, for the nearest dial (section 6). For the error itself: ruff, `figment` and `config` with `deny_unknown_fields`, `confique` with `layer_attr`, and `schematic` refuse [run]; uv refuses in one file and warns in another [run]; cargo and cargo-nextest only warn [run] |

**The order, and rules 4 and 5, read together.** The issue first listed the order as the command line, the environment, code, the project, the user, the system. With rule 4 and rule 5, that gave code > command line > environment > code for a dial that code may set. The issue now lists code, the command line, the environment, the project, the user, the system, which agrees with both rules. Each mechanism above needs one total order. #1213's prototype ranked the same order under `code-wins` ([`config.rs` L53-L55](https://github.com/kalonji-tools/purba/blob/fedc6c127d5890d4a1168faf869f2cf880bcccc8/crates/command/src/config.rs#L53-L55)).

## 9. What `lint:wiring` allows a crate of configuration

I ran `scripts/check-wiring.sh` of `main` on seven shapes, in a scratch clone at `9e221a74` [run]. The script is [`wiring.sh`](configuration-patterns/probes/wiring.sh) and the output is [`wiring.out`](configuration-patterns/probes/wiring.out).

| shape | exit | what the check says |
|---|---|---|
| G0 `main`, unchanged | 0 | — |
| G1 a crate `purba-configuration`, and the line of configuration names no crate | 1 | "A crate named purba-<part> is refused when <part> is no part of the diagram" |
| G2 the same crate, and the line ends with "Its crate is `purba-configuration`." | 0 | — |
| G3 G2, and the crate uses purba-errors | 1 | "The crate of a concern uses no part … purba-configuration uses errors" |
| G4 G2, and the crate uses purba-selection | 1 | "The crate of a concern uses no part … purba-configuration uses selection" |
| G5 G2, and dispatch uses the crate | 0 | — |
| G6 G2, and execution uses the crate | 1 | "A part may use the crate of a concern only when the concern's line names the part" |

So a crate of configuration, as purba's records stand:

- **cannot use purba-errors** (G3). The check counts every `purba-*` dependency, not only a part ([`check-wiring.sh` L135-L136](https://github.com/kalonji-tools/purba/blob/9e221a7452f9f8ec97a35706434599f676b664ab/scripts/check-wiring.sh#L135-L136)). The crate must return its own refusal, and a part turns it into an invocation error.
- **cannot read the code level itself** (G4). The prototype read it through `purba_selection::settings(facts)` ([`config.rs` L214](https://github.com/kalonji-tools/purba/blob/fedc6c127d5890d4a1168faf869f2cf880bcccc8/crates/command/src/config.rs#L214)). A part must hand it in as data.
- **serves command, selection, dispatch and reporting only** (G5, G6). The line names those four ([`LANDSCAPE.md` L156](https://github.com/kalonji-tools/purba/blob/9e221a7452f9f8ec97a35706434599f676b664ab/LANDSCAPE.md#L156)). The prototype armed the timeout in execution ([`zygote.rs` L1125](https://github.com/kalonji-tools/purba/blob/fedc6c127d5890d4a1168faf869f2cf880bcccc8/crates/execution/src/zygote.rs#L1125)), through command's drawn arrow to execution.
- **draws no arrow.** The record of errors names the same downside for purba-errors ([L48](https://github.com/kalonji-tools/purba/blob/9e221a7452f9f8ec97a35706434599f676b664ab/docs/decisions/an-outcome-and-an-error-live-in-a-crate-that-links-no-python.md?plain=1#L48)).

ruff's `ruff_workspace` and uv's `uv-settings` depend on the crates of their consumers' types. As a crate of a concern, either shape is G4: refused.

## 10. The four homes for purba

Each home lists its types, its crates and arrows, what `lint:wiring` says, how each rule is met, and its cost. Each sketch compiles alone [run]. "Own code" means purba writes it in that home.

### 10.1 Command reads and resolves, and hands each part its typed values

[`sketches/command_resolves.rs`](configuration-patterns/sketches/command_resolves.rs). It copies ruff and uv, and the prototype is one sketch of it.

```rust
// purba-dispatch: the part declares what it reads.
pub struct Seconds(pub NonZeroU32);
pub struct Dials { pub timeout: Option<Seconds>, pub timeouts: HashMap<String, Seconds>, pub workers: Option<u16> }
// purba-reporting
pub struct Dials { pub suppression_cap: Option<u32> }
// purba-command: the one table, the levels, and the resolve.
pub struct Dial { pub name: &'static str, pub rule: Rule, pub changes_an_outcome: bool }
pub const DIALS: [Dial; 3] = [/* timeout, workers, suppression-cap */];
impl Levels { pub fn dispatch(&self) -> Result<purba_dispatch::Dials, Refusal> { … } }
```

- **Crates and arrows.** No new crate. Command already has an arrow to selection, dispatch, reporting and execution. Command reads the code level from selection's answer, along the drawn arrow.
- **`lint:wiring`.** G0: it passes. A `toml` dependency of command is not a `purba-*` crate, so the check does not read it [source].
- **The rules.** Command's table meets rules 2, 3 and 6, and the order. A part's type meets rule 1, as `NonZeroU32` does for a timeout. Rule 4 needs selection's comments, which command already receives.
- **Cost.**
  - ✓ No new crate, no new arrow, no new line in the landscape.
  - ✓ The part declares the type of each value, so "the part that reads a value owns what it means" holds ([record](https://github.com/kalonji-tools/purba/blob/9e221a7452f9f8ec97a35706434599f676b664ab/docs/decisions/a-concern-is-what-no-part-can-own.md?plain=1#L42)).
  - ✓ One table holds every name, so rule 6's nearest dial is one search.
  - ✓ Command may use purba-errors, so a refusal is an invocation error directly.
  - ✓ #1213 ran this shape on a wheel.
  - ✗ Command grows. The prototype's reader is 453 lines ([`git diff --stat 27e8f559 fedc6c12`](https://github.com/kalonji-tools/purba/blob/fedc6c127d5890d4a1168faf869f2cf880bcccc8/crates/command/src/config.rs)).
  - ✗ A new dial edits two crates: the part's type, and command's table.
  - ✗ The name and rule of a dial sit in command, and its type in the part. Only a test ties them, as `crates/dispatch/tests/checks.rs` ties the names of the strict checks.
  - ✗ No crate of the concern holds it. The concern then lives in a part, the shape that [a concern's shared names may live in a part](https://github.com/kalonji-tools/purba/blob/9e221a7452f9f8ec97a35706434599f676b664ab/docs/decisions/a-concerns-shared-names-may-live-in-a-part.md#decision-outcome) allows for names, but not for a mechanism.

### 10.2 A crate of the concern holds the mechanism and every dial

[`sketches/concern_holds_every_dial.rs`](configuration-patterns/sketches/concern_holds_every_dial.rs). It copies purba-errors, and cargo's `context` with `schema.rs`.

```rust
// purba-configuration: uses no purba-* crate.
pub enum Level { CommandLine, Environment(String), Code { line: usize }, Project(PathBuf), User(PathBuf), System(PathBuf) }
pub struct Resolved<T> { pub value: T, pub level: Level }
pub struct Dials { pub timeout: Option<Resolved<u32>>, pub workers: Option<Resolved<u16>>, pub suppression_cap: Option<Resolved<u32>>, … }
pub struct CodeSetting { pub function: String, pub key: String, pub value: String, pub line: usize }
pub struct Refusal { pub words: String, pub nearest: Option<&'static str> }
pub fn resolve(sources: &Sources<'_>) -> Result<Dials, Refusal>;
```

- **Crates and arrows.** A new crate `purba-configuration`. The line of configuration gains "Its crate is `purba-configuration`.", as the line of errors names `purba-errors`. No arrow joins the parts diagram.
- **`lint:wiring`.** G2 and G5: it passes, while the crate uses no `purba-*` crate and only the four named parts use it. G3: purba-errors is refused. G6: execution is refused.
- **The rules.** One table in the crate meets rules 2, 3, 5 and 6, and the order. Rule 4 needs selection to hand `CodeSetting`s in, as data. Rule 1 lives in the crate's types.
- **Cost.**
  - ✓ The errors precedent: names and mechanism in a crate of the concern, which uses no part.
  - ✓ Every value keeps its level in one type, `Resolved<T>`.
  - ✓ Rules 2, 3 and 6 are written once.
  - ✗ The type of each value leaves the part that reads it. That sits against "the part that reads a value owns what it means".
  - ✗ The crate cannot use purba-errors (G3). It needs its own `Refusal`, and command turns it into an invocation error.
  - ✗ Each new dial edits the crate, and each part recompiles.
  - ✗ A use draws no arrow, the downside the record of errors names.
  - ✗ cargo-nextest's form of it imports its consumers' types. Across crates, that is G4: refused.

### 10.3 A crate of the concern holds the mechanism, and each part declares its own dials to it

[`sketches/each_part_declares_its_dials.rs`](configuration-patterns/sketches/each_part_declares_its_dials.rs). It copies figment's guide for libraries, `schematic`'s merge function for one field, and cargo's `CargoNewConfig`.

```rust
// purba-configuration: names and mechanism, no dial.
pub struct Dial { pub name: &'static str, pub rule: Rule, pub changes_an_outcome: bool, pub in_code: bool }
pub trait Dials: Sized {
    const DIALS: &'static [Dial];
    fn from_resolved(resolved: &Resolved) -> Result<Self, Refusal>;
}
pub fn read(declared: &[&'static [Dial]], levels: &[(Level, Vec<(String, String)>)]) -> Result<Resolved, Refusal>;
// purba-reporting
impl Dials for reporting::Dials { const DIALS: &'static [Dial] = &[Dial { name: "suppression-cap", rule: Rule::Lower, … }]; … }
// purba-command: it uses every part, so it gathers every declaration.
let resolved = configuration::read(&[purba_dispatch::Dials::DIALS, purba_reporting::Dials::DIALS], levels)?;
```

- **Crates and arrows.** A new crate `purba-configuration`, named in the line as in 10.2. No arrow joins the parts diagram. Command gathers each part's declaration along its drawn arrows.
- **`lint:wiring`.** As 10.2: G2 and G5 pass, G3, G4 and G6 are refused.
- **The rules.** Each part states rule 2 and rule 3 for its own dial, in its `Dial`. The mechanism applies them once. Rule 6 needs every declaration before the first key, so command gathers them. Rule 4 needs selection to hand code in as data.
- **Cost.**
  - ✓ The part keeps the type, the meaning and the rule of each value.
  - ✓ The mechanism is written once, and uses no part.
  - ✓ The concern's duty stays with each part that follows it, as the record of shared names says.
  - ✗ A part reads a value by its name as a string. A misspelled name inside a part goes unseen unless a test reads it.
  - ✗ Rule 6 depends on command listing every part's declaration. A part that command forgets makes its keys unknown.
  - ✗ Whether a dial changes an outcome is judged in each part. The record of concerns calls that line thin.
  - ✗ The crate cannot use purba-errors (G3), as in 10.2.

### 10.4 Each part reads its own values

[`sketches/each_part_reads.rs`](configuration-patterns/sketches/each_part_reads.rs). No surveyed tool does this. cargo-nextest comes closest: it has two readers, one for the project's files and one for the user's, each with its own type ([`user_config/imp.rs` L192](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/nextest-runner/src/user_config/imp.rs#L192)).

```rust
// purba-dispatch: its own copy of the levels, the walk and the order.
pub enum Level { … }
pub struct Dials { pub timeout: Option<(u32, Level)>, pub workers: Option<(u16, Level)> }
pub const KNOWN: [&str; 3] = ["timeout", "workers", "suppression-cap"];
pub fn read(arguments: &[String], environment: &[(String, String)], project: &[(String, String)]) -> Result<Dials, String>;
// purba-reporting: the same, again.
```

- **Crates and arrows.** No new crate, and no new arrow. Each part needs the arguments and the environment, which only command reads today.
- **`lint:wiring`.** G0: it passes.
- **The rules.** Each part writes the order and rules 2, 3 and 5 again. Rule 6 needs every key in every part: a copy in each, tied by a test.
- **Cost.**
  - ✓ No crate of the concern, and no part holds another part's values.
  - ✗ The mechanism is written three or four times, and the copies can drift.
  - ✗ Each part reads the same files again in one run.
  - ✗ Rule 6 needs a list of every key in each part.
  - ✗ The code level is read by selection and used by dispatch, so it crosses parts anyway.
  - ✗ One place must still print where each value came from, as the prototype did.

### Which home copies which pattern

| home | ruff | uv | cargo | cargo-nextest | `config` | `figment` | `confique` | `schematic` | purba-errors |
|---|---|---|---|---|---|---|---|---|---|
| 10.1 command resolves | ✓ | ✓ | | the command line only | as a dependency of command | as a dependency of command | as a dependency of command | as a dependency of command | |
| 10.2 the crate holds every dial | | | ✓ | ✓, with a cycle | inside the crate | inside the crate | inside the crate | inside the crate | ✓ |
| 10.3 each part declares its dials | | | `CargoNewConfig` | | | ✓ (its guide) | | the merge function of one field | ✓ |
| 10.4 each part reads | | | | two readers, by kind of file | | | | | |

`confique` and `schematic` derive one struct for the whole configuration. A part's struct can nest in it, but the top struct names every part. That top struct can live in command (10.1). It cannot live in a crate of the concern (G4).

## What was not checked

- I did not build purba with any home. The sketches compile alone, with std only.
- I did not run ruff, uv, cargo or cargo-nextest on a rule of a value's own. Those cells are [source].
- I did not time any mechanism, or measure its compile cost in purba.
- I did not probe whether an unknown `PURBA_*` variable should be refused. The prototype reads only the variable of each known dial, and ignores the others.
- I did not read the `ty` crates in ruff's repository beyond `ty_combine`'s first lines. They keep a level for each value, with `RangedValue`, unlike ruff's linter.
- Scoring the crates against purba's criteria belongs to the issue that vets the need line. I name only what each crate's design implies.

## What the design can take from this

This is input for the design in depth of configuration. It decides nothing.

- **The order needs one total order.** Every mechanism I read states one. The issue's corrected order, code > command line > environment > project > user > system, is one, and it agrees with rules 4 and 5.
- **Home 10.1 has the most precedent.** ruff and uv both resolve beside the binary, and hand each consumer a typed struct that the consumer's crate declares. #1213 ran it.
- **Home 10.3 is the one home with a crate of the concern that keeps each value's meaning in its part.** figment's guide recommends that shape to library authors. It follows the errors precedent: a crate of a concern that uses no part.
- **Home 10.2 moves each value's type out of its part.** cargo and cargo-nextest hold it inside one crate. cargo-nextest's form needs a dependency both ways, which `lint:wiring` and Cargo both refuse across crates.
- **Home 10.4 has no precedent** among the eight, and writes the order and rule 6 again in each part.
- **A crate of the concern needs its own refusal**, because it may not use purba-errors (G3). The line of configuration would also have to name execution, if execution reads a value itself (G6).
- **Rule 6's nearest dial is purba's own code in every pattern.** The prototype's edit distance already does it.
- **A crate from the vetting issue fits homes 10.1 to 10.3 as a dependency.** `schematic` needs its environment turned off to meet rule 5. `config` needs code as an override to meet rule 4. `confique` keeps no level, which the prototype's report of each value's level needs.

## Appendix A: each tool and crate

### A.1 ruff 0.17.6

- **Crate.** `ruff_workspace` holds `options.rs` (raw), `configuration.rs` (merged), `settings.rs` (resolved), `resolver.rs` and `pyproject.rs`. It depends on `ruff_linter`, `ruff_python_formatter` and `ruff_graph`, whose settings it builds ([`Cargo.toml` L19-L28](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_workspace/Cargo.toml#L19-L28)). Only `ruff`, `ruff_server` and tools of the repository depend on it [source].
- **Raw options.** `Options` has `#[serde(deny_unknown_fields, rename_all = "kebab-case")]` ([`options.rs` L49-L52](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_workspace/src/options.rs#L49-L52)) [source].
- **Merge.** `Configuration::combine` takes `self.x.or(config.x)` for each field, and chains `extend-*` lists ([`configuration.rs` L618](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_workspace/src/configuration.rs#L618)). The configurations along `extend` merge in order ([`resolver.rs` L396-L401](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_workspace/src/resolver.rs#L396-L401)) [source].
- **The command line.** `ConfigurationTransformer::transform` ([`resolver.rs` L285-L287](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_workspace/src/resolver.rs#L285-L287)). `ConfigArguments` combines `--config` pairs over the file, then applies each flag by hand in `ExplicitConfigOverrides` ([`args.rs` L780-L784](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff/src/args.rs#L780-L784), [L1411](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff/src/args.rs#L1411)) [source].
- **The user's file.** Read only when no project file is found ([`resolve.rs` L79](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff/src/resolve.rs#L79), [`pyproject.rs` L159](https://github.com/astral-sh/ruff/blob/a7ba68b4919e56b6af949a42b2da38fe4eec67a9/crates/ruff_workspace/src/pyproject.rs#L159)) [source].
- **Probe.** `line-lnegth = 80` in `[tool.ruff]`: ``unknown field `line-lnegth` ``, exit 2 [run].

### A.2 uv 0.13.7

- **Crates.** `uv-settings` reads and combines the files. `uv-cli/src/settings.rs` (5,240 lines) resolves each command's settings from the flags, the environment and the files [source].
- **Combine.** The trait `Combine`, "preferring the values in `self`", follows cargo's merge for lists ([`combine.rs` L23-L31](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-settings/src/combine.rs#L23-L31)). The derive `CombineOptions` calls `combine` on each field ([`uv-macros/src/lib.rs` L166](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-macros/src/lib.rs#L166)) [source].
- **The files.** `FilesystemOptions::user`, `system` and `find` ([`lib.rs` L59](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-settings/src/lib.rs#L59), [L87](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-settings/src/lib.rs#L87), [L106](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-settings/src/lib.rs#L106)). The binary takes `project.combine(user).combine(system)` ([`uv/src/lib.rs` L337-L347](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv/src/lib.rs#L337-L347)) [source].
- **Unknown keys.** `OptionsWire` has `deny_unknown_fields` ([`settings.rs` L2642-L2644](https://github.com/astral-sh/uv/blob/316c827e058c16b71d5c507a20729c2995a7bcce/crates/uv-settings/src/settings.rs#L2642-L2644)) [source].
- **Probe.** `index-urll` in `uv.toml`: error, exit 2. The same in `[tool.uv]`: a warning, exit 0. `[sources]` in the user's `uv.toml`: "The `sources` field is not allowed in a `uv.toml` file", exit 2 [run].

### A.3 cargo 1.99.0

- **Module.** `src/context/` of the crate `cargo`: `mod.rs` (2,638 lines), `value.rs`, `config_value.rs`, `de.rs`, `schema.rs` and five more [source].
- **Values.** `ConfigValue` is an integer, string, list, table or boolean, each with a `Definition` ([`config_value.rs` L43-L49](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/context/config_value.rs#L43-L49)). `merge` swaps in a value of higher priority, and joins lists ([L148](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/context/config_value.rs#L148)) [source].
- **The command line.** `merge_cli_args` merges `--config` with force ([`mod.rs` L1625](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/context/mod.rs#L1625)) [source].
- **Probe.** `[build] jbos = 4`: ``warning: unused config key `build.jbos` ``. `[bluid] jobs = 4`: nothing. Exit 0. `--config 'registry.token="x"'`: refused, exit 101 [run].

### A.4 cargo-nextest 0.9.149

- **Module.** `nextest-runner/src/config/`: `core/` (the read and the profiles), `elements/` (one type for each setting), `overrides/`, `scripts/`. The user's file has its own module, `user_config/` [source].
- **The read.** Each file goes through the crate `config` alone, for its warnings, then into one builder ([`core/imp.rs` L396-L428](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/nextest-runner/src/config/core/imp.rs#L396-L428)) [source].
- **Unknown keys.** `serde_ignored` collects them, and a `ConfigWarnings` impl prints them ([`core/imp.rs` L61-L62](https://github.com/nextest-rs/nextest/blob/3fe4ef31972f24e0c94cfe291718eab0fe678260/nextest-runner/src/config/core/imp.rs#L61-L62)) [source].
- **Probe.** `retires = 2` in `.config/nextest.toml`: "ignoring unknown configuration key: profile.default.retires". `[profile.default] retries = 2` in the user's file: "ignoring unknown configuration key: profile". The test passed, exit 0 [run].

### A.5 `config` 0.15.27

- **API.** `Config::builder()`, then `set_default`, `add_source` and `set_override` ([`builder.rs` L137](https://github.com/rust-cli/config-rs/blob/f14a0442345bd63e84638e37aa13daa383f0ba54/src/builder.rs#L137), [L154](https://github.com/rust-cli/config-rs/blob/f14a0442345bd63e84638e37aa13daa383f0ba54/src/builder.rs#L154), [L190](https://github.com/rust-cli/config-rs/blob/f14a0442345bd63e84638e37aa13daa383f0ba54/src/builder.rs#L190)). `get::<T>(key)` and `try_deserialize::<T>()` ([`config.rs` L107](https://github.com/rust-cli/config-rs/blob/f14a0442345bd63e84638e37aa13daa383f0ba54/src/config.rs#L107), [L145](https://github.com/rust-cli/config-rs/blob/f14a0442345bd63e84638e37aa13daa383f0ba54/src/config.rs#L145)) [source].
- **Probe.** `timeout = 1, origin = None` (the command line's override won over code); `cap = 50, origin = None`; `workers = 8, origin = Some("levels/user.toml")`. `get::<Option<u32>>("timeout")` on the misspelled file gave a missing field, not an unknown key [run].

### A.6 `figment` 0.10.19

- **API.** `merge`, `join`, `admerge`, `adjoin`; `extract`, `extract_inner`, `find_value`, `find_metadata` ([`figment.rs` L210-L333](https://github.com/SergioBenitez/Figment/blob/dae213f0f31d764a0c00aeb39f145222050bc657/src/figment.rs#L210-L333), [L819](https://github.com/SergioBenitez/Figment/blob/dae213f0f31d764a0c00aeb39f145222050bc657/src/figment.rs#L819)). `Tagged<T>` ([`magic.rs` L557](https://github.com/SergioBenitez/Figment/blob/dae213f0f31d764a0c00aeb39f145222050bc657/src/value/magic.rs#L557)). A `Serialized` provider names its type and the line that added it ([`serialized.rs` L188-L190](https://github.com/SergioBenitez/Figment/blob/dae213f0f31d764a0c00aeb39f145222050bc657/src/providers/serialized.rs#L188-L190)) [source].
- **Release.** crates.io dates 0.10.19 to 2024-05-17. The probe's lock held `toml` 0.8.23 for figment beside `toml` 1.2.2 for `confique` [run].
- **Probe.** `timeout = 3`, from the probe's line 34; `cap = 50`, from line 32; `workers = 8`, from `levels/user.toml`. `join` gave `cap = 50` [run].

### A.7 `confique` 0.5.0

- **API.** `#[derive(Config)]` makes a partial struct of `Option`s. `Config::builder()` takes `.env()`, `.file(path)` and a preloaded partial, earliest first; `load()` joins them with `with_fallback`, which is `Option::or` ([`builder.rs` L44-L54](https://github.com/LukasKalbertodt/confique/blob/39179571813e6aa42b010b82b2bee433b5f6446b/src/builder.rs#L44-L54), [`lib.rs` L557](https://github.com/LukasKalbertodt/confique/blob/39179571813e6aa42b010b82b2bee433b5f6446b/src/lib.rs#L557)). A field's attributes: `env`, `default`, `nested`, `validate`, `deserialize_with`, `parse_env` ([`parse.rs` L116-L148](https://github.com/LukasKalbertodt/confique/blob/39179571813e6aa42b010b82b2bee433b5f6446b/macro/src/parse.rs#L116-L148)) [source].
- **Probe.** `timeout = 3, cap = 50, workers = 8`, with no level. `tmeout = 4` gave `timeout: None` and no error, until the derive carried `deny_unknown_fields` [run].

### A.8 `schematic` 0.21.2

- **API.** `#[derive(Config)]` makes `Partial*`. `ConfigLoader::new()` takes `.file`, `.code`, `.url` and `.partial(name, partial)` ([`loader.rs` L168](https://github.com/moonrepo/schematic/blob/5b426874cea1b4569089040ea4553f6e27871b0b/crates/schematic/src/config/loader.rs#L168)). A field's attributes: `default`, `env`, `merge`, `validate`, `transform`, `nested`, `required` and more ([`field.rs` L21-L36](https://github.com/moonrepo/schematic/blob/5b426874cea1b4569089040ea4553f6e27871b0b/crates/core/src/field.rs#L21-L36)). `Source` is `Code`, `Defaults`, `EnvVars`, `File`, `Memory` or `Url` ([`source.rs` L9-L27](https://github.com/moonrepo/schematic/blob/5b426874cea1b4569089040ea4553f6e27871b0b/crates/schematic/src/config/source.rs#L9-L27)) [source].
- **Probe.** `timeout = 2` (the environment), `cap = 0` (the merge function), `workers = 8`. The list of partials named each file, "the command line", "code" and `EnvVars`, in that order. `tmeout`: "unknown field `tmeout`, expected one of `timeout`, `cap`, `workers`" [run].
