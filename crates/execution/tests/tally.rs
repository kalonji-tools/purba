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
        Err("test_a.py did not load"),
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
            errors: 1,
        },
        "one of each kind, and one error"
    );
}
