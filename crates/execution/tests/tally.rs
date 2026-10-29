use purba_execution::{Kind, Tally};

#[test]
fn a_tally_adds_each_kind_and_each_error() {
    let failed = Kind::Failed {
        assert: "assert 1 == 2".to_owned(),
    };
    let raised = Kind::Raised {
        exception: "ValueError: boom".to_owned(),
    };
    let mut tally = Tally::default();
    for result in [
        Ok(&Kind::Passed),
        Ok(&failed),
        Ok(&raised),
        Ok(&Kind::Unknown),
        Err("TypeError: 'int' object is not callable"),
    ] {
        tally.add(&result);
    }
    assert_eq!(
        tally,
        Tally {
            passed: 1,
            failed: 1,
            raised: 1,
            unknown: 1,
            blocked: 1,
            errors: 1,
        },
        "one of each kind, and a test whose own error blocks it counts as blocked and as one error"
    );
}

#[test]
fn a_tally_counts_a_blocked_test_apart_from_its_error() {
    let mut tally = Tally::default();
    tally.add_error();
    tally.add_blocked();
    tally.add_blocked();
    assert_eq!(
        tally,
        Tally {
            blocked: 2,
            errors: 1,
            ..Tally::default()
        },
        "a module that did not load counts once, and each of its two tests as blocked"
    );
}
