//! Selection and execution each write the names of the checks of the practice
//! tier whose breach shows while a case runs. Dispatch uses both parts, so its
//! tests can compare the two.

#[test]
fn execution_names_each_check_of_the_practice_tier_that_the_run_sees() {
    let mut table: Vec<&str> = purba_selection::seen_by_the_run().collect();
    let mut copy: Vec<&str> = purba_execution::SUPPRESSIBLE.to_vec();
    table.sort_unstable();
    copy.sort_unstable();

    assert_eq!(
        copy, table,
        "execution's copy holds each name that selection's table gives the run, and no other"
    );
}
