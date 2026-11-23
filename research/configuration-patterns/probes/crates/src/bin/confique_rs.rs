//! Probe of confique 0.5.0 against purba's six levels.
//! Run with PURBA_TIMEOUT=2 in the environment, from the probe's root.
use confique::Config;

#[derive(Debug, Config)]
#[allow(dead_code)]
struct Dials {
    #[config(env = "PURBA_TIMEOUT")]
    timeout: Option<u32>,
    cap: Option<u32>,
    workers: Option<u32>,
}

/// The same dials, with the partial type made strict by hand.
#[derive(Debug, Config)]
#[config(layer_attr(serde(deny_unknown_fields)))]
#[allow(dead_code)]
struct StrictDials {
    timeout: Option<u32>,
    cap: Option<u32>,
    workers: Option<u32>,
}

fn main() {
    // Highest level first: a source added earlier wins.
    let code = DialsLayer { timeout: Some(3), cap: None, workers: None };
    let cli = DialsLayer { timeout: Some(1), cap: Some(50), workers: None };
    let dials = Dials::builder()
        .layer(code)
        .layer(cli)
        .env()
        .file("levels/project.toml")
        .file("levels/user.toml")
        .file("levels/system.toml")
        .load()
        .unwrap();
    println!("A confique: typed = {dials:?} (no level is kept)");

    match Dials::builder().file("levels/typo.toml").load() {
        Ok(d) => println!("C confique: typo accepted, typed = {d:?}"),
        Err(error) => println!("C confique: typo refused: {error}"),
    }
    match StrictDials::builder().file("levels/typo.toml").load() {
        Ok(d) => println!("C confique strict: typo accepted, typed = {d:?}"),
        Err(error) => println!("C confique strict: typo refused: {error:#}"),
    }
}
