//! Probe of schematic 0.21.2 against purba's six levels.
//! Run with PURBA_TIMEOUT=2 in the environment, from the probe's root.
use schematic::{Config, ConfigLoader, MergeResult};

/// Rule 2's cap: the lower value applies, whatever the level.
fn lower<C>(prev: u32, next: u32, _: &C) -> MergeResult<u32> {
    Ok(Some(prev.min(next)))
}

#[derive(Debug, Config)]
#[allow(dead_code)]
struct Dials {
    #[setting(env = "PURBA_TIMEOUT")]
    timeout: Option<u32>,
    #[setting(merge = lower)]
    cap: Option<u32>,
    workers: Option<u32>,
}

fn main() {
    // Lowest level first; the loader puts the environment above every source.
    let cli = PartialDials { timeout: Some(1), cap: Some(50), workers: None };
    let code = PartialDials { timeout: Some(3), cap: None, workers: None };
    let mut loader = ConfigLoader::<Dials>::new();
    loader
        .file("levels/system.toml")
        .unwrap()
        .file("levels/user.toml")
        .unwrap()
        .file("levels/project.toml")
        .unwrap()
        .partial("the command line", cli)
        .partial("code", code);
    let result = loader.load().unwrap();
    println!("A schematic: typed = {:?}", result.config);
    for each in &result.layers {
        println!("A schematic: source {:?} sets {:?}", each.source, each.partial);
    }

    let mut typo = ConfigLoader::<Dials>::new();
    typo.file("levels/typo.toml").unwrap();
    match typo.load() {
        Ok(r) => println!("C schematic: typo accepted, typed = {:?}", r.config),
        Err(error) => {
            print!("C schematic: typo refused: {error}");
            let mut cause = std::error::Error::source(&error);
            while let Some(next) = cause {
                print!(" / {next}");
                cause = next.source();
            }
            println!();
        }
    }
}
