//! Probe of figment 0.10.19 against purba's six levels.
//! Run with PURBA_TIMEOUT=2 in the environment, from the probe's root.
use figment::providers::{Env, Format, Serialized, Toml};
use figment::value::magic::Tagged;
use figment::Figment;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct Dials {
    timeout: Option<u32>,
    cap: Option<u32>,
    workers: Option<u32>,
}

/// A consumer that keeps the level of its value.
#[derive(Debug, Deserialize)]
struct TimeoutOnly {
    timeout: Tagged<u32>,
}

fn main() {
    // Lowest level first: `merge` lets the later provider win.
    let figment = Figment::new()
        .merge(Toml::file("levels/system.toml"))
        .merge(Toml::file("levels/user.toml"))
        .merge(Toml::file("levels/project.toml"))
        .merge(Env::prefixed("PURBA_"))
        // The command line: `--timeout 1 --cap 50`.
        .merge(Serialized::default("timeout", 1))
        .merge(Serialized::default("cap", 50))
        // Code, above the command line (rule 4).
        .merge(Serialized::default("timeout", 3));
    for key in ["timeout", "cap", "workers"] {
        let value = figment.find_value(key).unwrap();
        let metadata = figment.find_metadata(key).unwrap();
        println!(
            "A figment: {key} = {:?} , from {:?} {:?}",
            value.to_i128(),
            metadata.name,
            metadata.source.as_ref().map(ToString::to_string)
        );
    }
    let dials: Dials = figment.extract().unwrap();
    println!("B figment: typed = {dials:?}");
    let only: TimeoutOnly = figment.extract().unwrap();
    let tag = only.timeout.tag();
    println!(
        "B figment: Tagged timeout = {} , from {:?}",
        only.timeout.clone().into_inner(),
        figment.get_metadata(tag).map(|m| m.name.to_string())
    );
    println!(
        "B figment: by key = {:?}",
        figment.extract_inner::<u32>("timeout")
    );

    // Rule 2 without custom code: `join` keeps the first value set, so it
    // gives "the lower level wins", not "the lower value wins".
    let joined = Figment::new()
        .join(Serialized::default("cap", 50))
        .join(Toml::file("levels/project.toml"));
    println!("D figment: join cap = {:?}", joined.extract_inner::<u32>("cap"));

    let typo = Figment::new().merge(Toml::file("levels/typo.toml"));
    match typo.extract::<Dials>() {
        Ok(_) => println!("C figment: typo accepted"),
        Err(error) => println!("C figment: typo refused: {error}"),
    }
}
