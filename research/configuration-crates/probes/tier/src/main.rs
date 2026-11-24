//! Probe of `tier` 0.1.21 against purba's rules (a) to (g).
//! Run from `probes/` with `PURBA_TIMEOUT=2 PURBA_WORKERS=2`; `run.sh` does it.
#[path = "../../rules.rs"]
mod rules;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tier::{ConfigLoader, ConfigMetadata, FieldMetadata, Layer, LoadedConfig, SourceKind};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Every {
    timeout: Option<u32>,
    cap: Option<u32>,
    workers: Option<u32>,
}

/// A level: a file that tier reads, or a layer that purba hands in.
#[derive(Clone)]
enum Level {
    File(&'static str),
    Custom(&'static str, Value),
}

/// purba's code: tier reads `PURBA__TIMEOUT`, not `PURBA_TIMEOUT`, and puts its own
/// environment above every custom layer. So purba reads the variables itself.
fn environment() -> Value {
    let mut map = serde_json::Map::new();
    for (name, text) in std::env::vars() {
        if let Some(key) = name.strip_prefix("PURBA_") {
            map.insert(key.to_lowercase(), text.parse::<u64>().map_or(Value::String(text), Value::from));
        }
    }
    Value::Object(map)
}

/// The six levels, lowest first. `project_as_file` false hands the project in as a layer.
fn six(user: &'static str, project: &'static str, project_as_file: bool) -> Vec<(&'static str, Level)> {
    let project = if project_as_file {
        Level::File(project)
    } else {
        Level::Custom("the project", toml::from_str::<Value>(&std::fs::read_to_string(project).unwrap()).unwrap())
    };
    vec![
        ("the system", Level::File("levels/system.toml")),
        ("the user", Level::File(user)),
        ("the project", project),
        ("the environment", Level::Custom("the environment", environment())),
        ("the command line", Level::Custom("the command line", json!({"timeout": 1, "cap": 50, "workers": 1}))),
        ("code", Level::Custom("code", json!({"timeout": 3, "workers": 3}))),
    ]
}

fn load<T>(defaults: T, levels: &[(&'static str, Level)], metadata: ConfigMetadata) -> Result<LoadedConfig<T>, tier::ConfigError>
where
    T: Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
{
    let mut loader = ConfigLoader::new(defaults).deny_unknown_fields().metadata(metadata);
    for (_, level) in levels {
        loader = match level {
            Level::File(path) => loader.file(*path),
            Level::Custom(name, value) => loader.layer(Layer::custom(*name, value.clone())?),
        };
    }
    loader.load()
}

fn show<T>(loaded: &LoadedConfig<T>, key: &str) -> String {
    match loaded.report().explain(key) {
        Some(explained) => {
            let last = explained.steps.last().map(|step| step.source.name.clone());
            format!("{key} = {:?} from {last:?}; every step: {:?}", explained.final_value, explained.steps.iter().map(|s| (s.source.name.as_str(), s.value.to_string())).collect::<Vec<_>>())
        }
        None => format!("{key}: no trace"),
    }
}

/// purba's code: the keys of each level, read from tier's trace of every step.
fn entries<T>(loaded: &LoadedConfig<T>) -> Vec<rules::Entry> {
    let mut out = Vec::new();
    // The trace also holds the empty path, for each whole document.
    for (key, steps) in loaded.report().traces().iter().filter(|(key, _)| !key.is_empty()) {
        for step in steps {
            let level = match step.source.name.as_str() {
                "levels/system.toml" => "the system",
                "levels/user.toml" | "levels/user-closed.toml" => "the user",
                "levels/project.toml" | "the project" => "the project",
                "the environment" => "the environment",
                "the command line" => "the command line",
                "code" => "code",
                _ => continue,
            };
            out.push(rules::Entry { level, key: key.clone(), value: step.value.to_string() });
        }
    }
    out
}

fn main() {
    let none = ConfigMetadata::new;
    let levels = six("levels/user.toml", "levels/project.toml", true);
    for top in (1..=levels.len()).rev() {
        let loaded = load(Every::default(), &levels[..top], none()).unwrap();
        println!("(a) tier: up to {}: {}", levels[top - 1].0, show(&loaded, "workers"));
    }
    let loaded = load(Every::default(), &levels, none()).unwrap();
    for key in ["timeout", "cap", "workers"] {
        println!("(d) tier: {}", show(&loaded, key));
    }
    println!("(c) tier: out of the box, cap = {:?}", loaded.cap);
    println!("(c) tier: purba's min over the traced steps = {:?}", rules::lower(&entries(&loaded), "cap"));

    let closed = six("levels/user-closed.toml", "levels/project.toml", true);
    match load(Every::default(), &closed, none()) {
        Ok(loaded) => println!("(b) tier: user sets timeout = 5; accepted: timeout = {:?}; purba's check = {:?}", loaded.timeout, rules::refusals(&rules::every_dial(), &entries(&loaded))),
        Err(error) => println!("(b) tier: refused: {error}"),
    }
    // tier's own policy works by kind of source. It can tell the user's file from the
    // project only when the project is not a file.
    let deny_files = || ConfigMetadata::from_fields([FieldMetadata::new("timeout").deny_sources([SourceKind::File]), FieldMetadata::new("cap").deny_sources([SourceKind::File])]);
    let closed_layer = six("levels/user-closed.toml", "levels/project.toml", false);
    match load(Every::default(), &closed_layer, deny_files()) {
        Ok(loaded) => println!("(b) tier: deny File on outcome dials, project as a layer: accepted, timeout = {:?}", loaded.timeout),
        Err(error) => println!("(b) tier: deny File on outcome dials, project as a layer: refused: {error}"),
    }
    match load(Every::default(), &six("levels/user.toml", "levels/project.toml", false), deny_files()) {
        Ok(loaded) => println!("(b) tier: the same policy, user sets no outcome dial: accepted, timeout = {:?}, cap = {:?}", loaded.timeout, loaded.cap),
        Err(error) => println!("(b) tier: the same policy, user sets no outcome dial: refused: {error}"),
    }

    match load(Every::default(), &six("levels/user.toml", "levels/typo.toml", true), none()) {
        Ok(_) => println!("(e) tier: typo accepted"),
        Err(error) => println!("(e) tier: typo refused: {error}"),
    }

    match load(Every::default(), &six("levels/bad-user.toml", "levels/bad-project.toml", true), none()) {
        Ok(_) => println!("(f) tier: three faults accepted"),
        Err(error) => println!("(f) tier: three faults in two files gave: {error}"),
    }
    let mut both = six("levels/user-closed.toml", "levels/typo.toml", false);
    both[2] = ("the project", Level::Custom("the project", json!({"tmeout": 4})));
    match load(Every::default(), &both, deny_files()) {
        Ok(_) => println!("(f) tier: closed level and typo accepted"),
        Err(error) => println!("(f) tier: a closed-level fault and an unknown key gave: {error}"),
    }

    // (g): each part declares its own metadata; command extends one with the other.
    // The typed value is a JSON tree, so no struct names both parts.
    let mut metadata = ConfigMetadata::from_fields([FieldMetadata::new("timeout"), FieldMetadata::new("workers")]);
    metadata.extend(ConfigMetadata::from_fields([FieldMetadata::new("cap")]));
    let defaults = json!({"timeout": null, "cap": null, "workers": null});
    match load(defaults.clone(), &six("levels/user.toml", "levels/typo.toml", true), metadata.clone()) {
        Ok(loaded) => println!("(g) tier: JSON tree, typo: accepted: {}", loaded.config()),
        Err(error) => println!("(g) tier: JSON tree, typo: refused: {error}"),
    }
    match load(defaults, &levels, metadata) {
        Ok(loaded) => {
            let dispatch: Result<DispatchDials, _> = serde_json::from_value(loaded.config().clone());
            println!("(g) tier: JSON tree, dispatch takes its own struct: {dispatch:?}");
        }
        Err(error) => println!("(g) tier: JSON tree refused: {error}"),
    }
    // (g): command composes the parts' structs; purba-config stays generic over the type.
    match load(Composed::default(), &six("levels/user.toml", "levels/typo.toml", true), none()) {
        Ok(loaded) => println!("(g) tier: composed struct with flatten, typo: accepted: {:?}", loaded.config()),
        Err(error) => println!("(g) tier: composed struct with flatten, typo: refused: {error}"),
    }
    match load(Composed::default(), &levels, none()) {
        Ok(loaded) => println!("(g) tier: composed struct with flatten: {:?}; {}", loaded.config(), show(&loaded, "cap")),
        Err(error) => println!("(g) tier: composed struct with flatten: refused: {error}"),
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ReportingDials {
    cap: Option<u32>,
}

/// What command would hold: one struct that names each part's struct.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Composed {
    #[serde(flatten)]
    dispatch: DispatchDials,
    #[serde(flatten)]
    reporting: ReportingDials,
}

/// Part dispatch's own struct.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(dead_code)]
struct DispatchDials {
    timeout: Option<u32>,
    workers: Option<u32>,
}
