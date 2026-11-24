// The probe body that `figment_rs.rs` and `figment2_rs.rs` share. Each binds `fig` to its crate.
// Run from `probes/` with `PURBA_TIMEOUT=2 PURBA_WORKERS=2`; `run.sh` does it.
#[path = "../../rules.rs"]
mod rules;

use fig::providers::{Env, Format, Serialized, Toml};
use fig::value::{Dict, Map};
use fig::{Error, Figment, Metadata, Profile, Provider};
use serde::Deserialize;
use std::collections::BTreeMap;

/// purba's code: a provider that names its level.
struct Named<P> {
    name: &'static str,
    inner: P,
}

impl<P: Provider> Provider for Named<P> {
    fn metadata(&self) -> Metadata {
        Metadata::named(self.name)
    }
    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        self.inner.data()
    }
}

fn memory(pairs: &[(&'static str, i64)]) -> Serialized<BTreeMap<&'static str, i64>> {
    Serialized::defaults(pairs.iter().copied().collect())
}

/// The six levels, lowest first.
fn six(user: &str, project: &str) -> Vec<Box<dyn Provider>> {
    vec![
        Box::new(Named { name: "the system", inner: Toml::file("levels/system.toml") }),
        Box::new(Named { name: "the user", inner: Toml::file(user) }),
        Box::new(Named { name: "the project", inner: Toml::file(project) }),
        Box::new(Named { name: "the environment", inner: Env::prefixed("PURBA_") }),
        Box::new(Named { name: "the command line", inner: memory(&[("timeout", 1), ("cap", 50), ("workers", 1)]) }),
        Box::new(Named { name: "code", inner: memory(&[("timeout", 3), ("workers", 3)]) }),
    ]
}

const NAMES: [&str; 6] = ["the system", "the user", "the project", "the environment", "the command line", "code"];

/// Lets a boxed provider be merged more than once.
struct Ref<'a>(&'a dyn Provider);

impl Provider for Ref<'_> {
    fn metadata(&self) -> Metadata {
        self.0.metadata()
    }
    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        self.0.data()
    }
}

fn build(levels: &[Box<dyn Provider>]) -> Figment {
    levels.iter().fold(Figment::new(), |figment, level| figment.merge(Ref(&**level)))
}

fn show(figment: &Figment, key: &str) -> String {
    match (figment.find_value(key), figment.find_metadata(key)) {
        (Ok(value), Some(metadata)) => format!("{key} = {:?} from {:?}", text(&value), metadata.name),
        (value, _) => format!("{key}: {value:?}"),
    }
}

/// purba's code: the keys of each level apart, for rules (b), (c) and (e).
fn entries(levels: &[Box<dyn Provider>]) -> Vec<rules::Entry> {
    let mut out = Vec::new();
    for (level, provider) in NAMES.iter().zip(levels) {
        if let Ok(map) = provider.data() {
            for dict in map.values() {
                for (key, value) in dict {
                    out.push(rules::Entry { level, key: key.clone(), value: text(value) });
                }
            }
        }
    }
    out
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct Every {
    timeout: Option<u32>,
    cap: Option<u32>,
    workers: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct DispatchStrict {
    timeout: Option<u32>,
    workers: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DispatchLoose {
    timeout: Option<u32>,
    workers: Option<u32>,
}

fn main() {
    let tag = CRATE;
    let levels = six("levels/user.toml", "levels/project.toml");
    for top in (1..=levels.len()).rev() {
        println!("(a) {tag}: up to {}: {}", NAMES[top - 1], show(&build(&levels[..top]), "workers"));
    }
    let figment = build(&levels);
    for key in ["timeout", "cap", "workers"] {
        println!("(d) {tag}: {}", show(&figment, key));
    }
    println!("(c) {tag}: out of the box, {}", show(&figment, "cap"));
    println!("(c) {tag}: purba's min over the levels = {:?}", rules::lower(&entries(&levels), "cap"));

    let closed = six("levels/user-closed.toml", "levels/project.toml");
    match build(&closed).extract::<Every>() {
        Ok(every) => println!("(b) {tag}: user sets timeout = 5; the crate accepts it: {every:?}"),
        Err(error) => println!("(b) {tag}: refused: {error}"),
    }
    println!("(b) {tag}: purba's check = {:?}", rules::refusals(&rules::every_dial(), &entries(&closed)));

    let typo = six("levels/user.toml", "levels/typo.toml");
    println!("(e) {tag}: typed, deny_unknown_fields: {:?}", build(&typo).extract::<Every>().map(|_| ()).map_err(|e| e.to_string()));
    println!("(e) {tag}: purba's check = {:?}", rules::refusals(&rules::every_dial(), &entries(&typo)));

    let bad = six("levels/bad-user.toml", "levels/bad-project.toml");
    match build(&bad).extract::<Every>() {
        Ok(_) => println!("(f) {tag}: three faults accepted"),
        Err(error) => {
            println!("(f) {tag}: three faults in two files gave {} error(s):", error.count());
            for one in error {
                println!("(f) {tag}:   {one}");
            }
        }
    }

    println!("(g) {tag}: dispatch's strict struct on the shared tree: {:?}", figment.extract::<DispatchStrict>().map_err(|e| e.to_string()));
    println!("(g) {tag}: dispatch's loose struct on the shared tree: {:?}", figment.extract::<DispatchLoose>().map_err(|e| e.to_string()));
}

/// A number, as text. `to_i128` returns `None` for the `USize` that `Env` gives.
fn text(value: &fig::value::Value) -> String {
    value.to_i128().map(|v| v.to_string()).or_else(|| value.to_u128().map(|v| v.to_string())).or_else(|| value.as_str().map(str::to_owned)).unwrap_or_else(|| format!("{value:?}"))
}
