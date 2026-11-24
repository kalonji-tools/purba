//! The rules that purba would write itself, whichever crate merges the levels.
//! Each probe includes this file. It needs, for each level, the keys that the level sets.
//! It meets rule (b) closed levels, (c) the cap, and (e) the nearest dial, and it
//! gathers every refusal, as rule (f) asks.

/// A dial, as a part declares it.
pub struct Dial {
    pub name: &'static str,
    /// The dial changes an outcome, so the user and the system may not set it.
    pub outcome: bool,
}

/// One key that one level sets, before any merge.
pub struct Entry {
    pub level: &'static str,
    pub key: String,
    pub value: String,
}

pub const CLOSED: [&str; 2] = ["the user", "the system"];

/// Every refusal of rules (b) and (e), in the order of the entries.
pub fn refusals(dials: &[&Dial], entries: &[Entry]) -> Vec<String> {
    let mut out = Vec::new();
    for entry in entries {
        match dials.iter().find(|dial| dial.name == entry.key) {
            None => out.push(format!(
                "{}: unknown key `{}`; the nearest dial is `{}`",
                entry.level,
                entry.key,
                nearest(&entry.key, dials)
            )),
            Some(dial) if dial.outcome && CLOSED.contains(&entry.level) => out.push(format!(
                "{}: `{}` changes an outcome, and this level may not set it",
                entry.level, entry.key
            )),
            Some(_) => {}
        }
    }
    out
}

/// Rule (c): the lower value of the cap applies, and keeps its level.
pub fn lower(entries: &[Entry], key: &str) -> Option<(i64, &'static str)> {
    entries
        .iter()
        .filter(|entry| entry.key == key)
        .filter_map(|entry| entry.value.parse::<i64>().ok().map(|value| (value, entry.level)))
        .min_by_key(|(value, _)| *value)
}

fn nearest(key: &str, dials: &[&Dial]) -> &'static str {
    dials
        .iter()
        .min_by_key(|dial| distance(key, dial.name))
        .map_or("", |dial| dial.name)
}

fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (above + 1).min(row[j] + 1).min(diagonal + usize::from(ca != *cb));
            diagonal = above;
        }
    }
    row[b.len()]
}

/// The two parts of the probe. Each declares its own dials; the rules above name neither.
pub mod dispatch {
    pub const DIALS: [super::Dial; 2] = [
        super::Dial { name: "timeout", outcome: true },
        super::Dial { name: "workers", outcome: false },
    ];
}

pub mod reporting {
    pub const DIALS: [super::Dial; 1] = [super::Dial { name: "cap", outcome: true }];
}

/// What command does: it gathers each part's declarations.
pub fn every_dial() -> Vec<&'static Dial> {
    dispatch::DIALS.iter().chain(reporting::DIALS.iter()).collect()
}
