//! Probe of `config` 0.15.27 against purba's rules (a) to (g).
//! Run from `probes/` with `PURBA_TIMEOUT=2 PURBA_WORKERS=2`; `run.sh` does it.
#[path = "../../rules.rs"]
mod rules;

use config::{Config, ConfigError, Environment, File, FileFormat, Map, Source, Value, ValueKind};
use serde::Deserialize;

/// A level that purba hands in from memory: the command line, or code.
#[derive(Debug, Clone)]
struct Memory {
    name: String,
    pairs: Vec<(&'static str, i64)>,
}

impl Source for Memory {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.clone())
    }
    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        Ok(self
            .pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), Value::new(Some(&self.name), ValueKind::I64(*value))))
            .collect())
    }
}

/// The six levels, lowest first, each with its name.
fn six(user: &str, project: &str) -> Vec<(&'static str, Box<dyn Source + Send + Sync>)> {
    vec![
        ("the system", Box::new(File::new("levels/system.toml", FileFormat::Toml))),
        ("the user", Box::new(File::new(user, FileFormat::Toml))),
        ("the project", Box::new(File::new(project, FileFormat::Toml))),
        ("the environment", Box::new(Environment::with_prefix("PURBA").try_parsing(true))),
        ("the command line", Box::new(Memory { name: "the command line".into(), pairs: vec![("timeout", 1), ("cap", 50), ("workers", 1)] })),
        ("code", Box::new(Memory { name: "code".into(), pairs: vec![("timeout", 3), ("workers", 3)] })),
    ]
}

fn build(levels: &[(&'static str, Box<dyn Source + Send + Sync>)]) -> Result<Config, ConfigError> {
    let mut builder = Config::builder();
    for (_, source) in levels {
        builder = builder.add_source(vec![source.clone_into_box()]);
    }
    builder.build()
}

fn show(config: &Config, key: &str) -> String {
    // `get::<Value>` drops the origin; the public `cache` keeps it.
    let table = config.cache.clone().into_table().unwrap();
    match table.get(key) {
        Some(value) => format!("{key} = {value} from {:?}", value.origin()),
        None => format!("{key}: not set"),
    }
}

/// purba's code: the keys of each level apart, for rules (b), (c) and (e).
fn entries(levels: &[(&'static str, Box<dyn Source + Send + Sync>)]) -> Vec<rules::Entry> {
    let mut out = Vec::new();
    for (level, source) in levels {
        if let Ok(map) = source.collect() {
            for (key, value) in map {
                out.push(rules::Entry { level, key, value: value.to_string() });
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

/// Part dispatch declares its own struct; it does not know reporting's `cap`.
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
    let levels = six("levels/user.toml", "levels/project.toml");
    for top in (1..=levels.len()).rev() {
        let config = build(&levels[..top]).unwrap();
        println!("(a) config: up to {}: {}", levels[top - 1].0, show(&config, "workers"));
    }
    let config = build(&levels).unwrap();
    for key in ["timeout", "cap", "workers"] {
        println!("(d) config: {}", show(&config, key));
    }
    println!("(c) config: out of the box, {}", show(&config, "cap"));
    println!("(c) config: purba's min over the levels = {:?}", rules::lower(&entries(&levels), "cap"));

    let closed = six("levels/user-closed.toml", "levels/project.toml");
    match build(&closed) {
        Ok(config) => println!("(b) config: user sets timeout = 5; the crate accepts it: {}", show(&config, "timeout")),
        Err(error) => println!("(b) config: refused: {error}"),
    }
    println!("(b) config: purba's check = {:?}", rules::refusals(&rules::every_dial(), &entries(&closed)));

    let typo = six("levels/user.toml", "levels/typo.toml");
    let result = build(&typo).and_then(Config::try_deserialize::<Every>);
    println!("(e) config: typed, deny_unknown_fields: {:?}", result.map(|_| ()).map_err(|e| e.to_string()));
    println!("(e) config: purba's check = {:?}", rules::refusals(&rules::every_dial(), &entries(&typo)));

    let bad = six("levels/bad-user.toml", "levels/bad-project.toml");
    let result = build(&bad).and_then(Config::try_deserialize::<Every>);
    println!("(f) config: three faults in two files gave: {:?}", result.map(|_| ()).map_err(|e| e.to_string()));

    let result = config.clone().try_deserialize::<DispatchStrict>();
    println!("(g) config: dispatch's strict struct on the shared tree: {:?}", result.map_err(|e| e.to_string()));
    let result = config.clone().try_deserialize::<DispatchLoose>();
    println!("(g) config: dispatch's loose struct on the shared tree: {:?}", result.map_err(|e| e.to_string()));
}
