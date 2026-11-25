use std::ffi::{OsStr, OsString};

use purba_config::{Environment, Level, Variable};

/// A dial for the test: `PURBA_WORKERS`, a whole number, 1 by default.
static WORKERS: Variable<i64> = Variable::new("PURBA_WORKERS", read_workers, || 1);

fn read_workers(value: &OsStr) -> Result<i64, Vec<String>> {
    value
        .to_str()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| {
            vec![format!(
                "PURBA_WORKERS is {}; set a number",
                value.display()
            )]
        })
}

/// The level of the environment that the pairs of `variables` set.
fn environment(variables: &[(&str, &str)]) -> Environment {
    Environment::new(
        variables
            .iter()
            .map(|&(name, value)| (OsString::from(name), OsString::from(value))),
    )
}

#[test]
fn a_variable_that_is_not_set_takes_its_default_from_no_level() {
    let resolved = environment(&[("HOME", "/home/a")])
        .resolve(&WORKERS)
        .expect("no variable is no error");

    assert_eq!(
        (resolved.value, resolved.level),
        (1, Level::Default),
        "the default, from no level"
    );
}

#[test]
fn a_variable_that_is_set_takes_its_value_from_the_environment() {
    let resolved = environment(&[("PURBA_WORKERS", "4")])
        .resolve(&WORKERS)
        .expect("a number is no error");

    assert_eq!(
        (resolved.value, resolved.level),
        (4, Level::Environment),
        "the value of the variable, from the level of the environment"
    );
}

#[test]
fn a_value_that_the_dial_refuses_gives_the_dials_words() {
    assert_eq!(
        environment(&[("PURBA_WORKERS", "four")])
            .resolve(&WORKERS)
            .map(|resolved| resolved.value),
        Err(vec!["PURBA_WORKERS is four; set a number".to_owned()]),
        "the words of the dial's part"
    );
}

#[test]
fn a_purba_variable_one_or_two_edits_from_a_dial_names_the_dial_sorted_by_name() {
    let near = environment(&[
        ("PURBA_WORKRES", "2"),
        ("PURBA_WORKER", "2"),
        ("PURBA_WORKRE_", "2"),
        ("PURBA_wORKERS", "2"),
    ])
    .near(&["PURBA_WORKERS"]);
    let line = |name: &str| format!("{name} is no variable of purba; the nearest is PURBA_WORKERS");

    assert_eq!(
        near,
        vec![
            line("PURBA_WORKER"),
            line("PURBA_WORKRES"),
            line("PURBA_WORKRE_"),
            line("PURBA_wORKERS"),
        ],
        "a swap, a drop and a change of case are one edit each, and two edits are near too"
    );
}

#[test]
fn a_variable_three_edits_away_or_outside_purba_is_ignored() {
    assert_eq!(
        environment(&[
            ("PURBA_WORK", "2"),
            ("PURBA_SENTINEL", "2"),
            ("PURBA_workers", "2"),
            ("PURBX_WORKERS", "2"),
            ("purba_workers", "2"),
        ])
        .near(&["PURBA_WORKERS"]),
        Vec::<String>::new(),
        "three edits are far, each letter of another case is an edit, and a name that does \
         not start with PURBA_ is not purba's"
    );
}

#[test]
fn a_dials_own_variable_is_not_near() {
    assert_eq!(
        environment(&[("PURBA_WORKERS", "2")]).near(&["PURBA_WORKERS"]),
        Vec::<String>::new(),
        "the variable of a dial names no nearest"
    );
}
