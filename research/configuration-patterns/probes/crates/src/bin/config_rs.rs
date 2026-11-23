//! Probe of config 0.15.27 against purba's six levels.
//! Run with PURBA_TIMEOUT=2 in the environment, from the probe's root.
use config::{Config, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct Dials {
    timeout: Option<u32>,
    cap: Option<u32>,
    workers: Option<u32>,
}

fn main() {
    // Lowest level first: a source added later overwrites one added earlier.
    let built = Config::builder()
        .add_source(File::with_name("levels/system.toml"))
        .add_source(File::with_name("levels/user.toml"))
        .add_source(File::with_name("levels/project.toml"))
        .add_source(Environment::with_prefix("PURBA"))
        // The command line: `--timeout 1 --cap 50`.
        .set_override("timeout", 1)
        .unwrap()
        .set_override("cap", 50)
        .unwrap()
        // Code, added after the command line (rule 4). It is a source, and
        // every override wins over every source, whatever the order of calls.
        .add_source(File::from_str("timeout = 3", FileFormat::Toml))
        .build()
        .unwrap();
    let table = built.cache.clone().into_table().unwrap();
    for key in ["timeout", "cap", "workers"] {
        let value = &table[key];
        println!("A config: {key} = {value} , origin = {:?}", value.origin());
    }
    let dials: Dials = built.clone().try_deserialize().unwrap();
    println!("B config: typed = {dials:?}");

    let typo = Config::builder()
        .add_source(File::with_name("levels/typo.toml"))
        .build()
        .unwrap();
    println!("C config: get by key on typo = {:?}", typo.get::<Option<u32>>("timeout"));
    println!("C config: typed on typo = {:?}", typo.try_deserialize::<Dials>().map(|_| ()));
}
