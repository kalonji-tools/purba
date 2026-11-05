use purba_execution::{Error, ErrorKind, Kind, Tally};

/// The internal error that `catch` gives for a panic, which names `part`.
fn internal(part: &str) -> Error {
    purba_execution::catch(part, || -> Result<(), Error> { panic!("boom") })
        .expect_err("a panic gives an error")
}

#[test]
fn a_tally_adds_each_kind_and_each_error_by_its_kind() {
    let failed = Kind::Failed {
        assert: "assert 1 == 2".to_owned(),
    };
    let raised = Kind::Raised {
        exception: "ValueError: boom".to_owned(),
    };
    let project = Error::project("TypeError: 'int' object is not callable".to_owned());
    let internal = internal("dispatch");
    let mut tally = Tally::default();
    for result in [
        Ok(&Kind::Passed),
        Ok(&failed),
        Ok(&raised),
        Ok(&Kind::Unknown),
        Err(&project),
        Err(&internal),
    ] {
        tally.add(result);
    }
    assert_eq!(
        tally,
        Tally {
            passed: 1,
            failed: 1,
            raised: 1,
            unknown: 1,
            blocked: 2,
            project_errors: 1,
            internal_errors: 1,
            invocation_errors: 0,
            unreached: 0,
        },
        "one of each kind, and a test whose own error blocks it counts as blocked \
         and as one error of that error's kind"
    );
}

#[test]
fn a_tally_counts_an_error_that_blocks_no_test_by_its_kind_and_apart_from_its_tests() {
    let mut tally = Tally::default();
    tally.add_error(&Error::project("ValueError: x".to_owned()));
    tally.add_blocked();
    tally.add_blocked();
    tally.add_error(&internal("dispatch"));
    assert_eq!(
        tally,
        Tally {
            blocked: 2,
            project_errors: 1,
            internal_errors: 1,
            ..Tally::default()
        },
        "each module's error counts once by its kind, and each of the two tests as blocked"
    );
}

#[test]
fn a_tally_notes_a_refused_write_as_an_invocation_error() {
    let mut tally = Tally::default();
    tally.add_invocation_error();
    assert_eq!(
        tally,
        Tally {
            invocation_errors: 1,
            ..Tally::default()
        },
        "a refused write counts no test"
    );
}

#[test]
fn an_error_says_its_kind() {
    let project = Error::project("ValueError: x".to_owned());
    assert_eq!(
        (project.kind(), project.to_string()),
        (ErrorKind::Project, "ValueError: x".to_owned()),
        "a project error built from text prints that text"
    );
    let internal = internal("command");
    assert_eq!(
        (internal.kind(), internal.to_string()),
        (ErrorKind::Internal { part: "command" }, "boom".to_owned()),
        "an internal error names its part, and prints its message"
    );
}

#[test]
fn a_tally_counts_an_unreached_test_apart_from_each_outcome() {
    let mut tally = Tally::default();
    tally.add(Ok(&Kind::Passed));
    tally.add_unreached();
    tally.add_unreached();
    assert_eq!(
        tally,
        Tally {
            passed: 1,
            unreached: 2,
            ..Tally::default()
        },
        "one passed test, and two that the run never reached"
    );
}
