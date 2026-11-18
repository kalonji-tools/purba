use std::panic;

use purba_execution::panic::{Panic, take};

// A panic hook belongs to the process, and cargo runs the tests of one binary
// on several threads. A panic in another test could take the place of the
// note of the panic made here. So this test lives alone in its binary.
#[test]
fn set_hooks_wraps_the_panic_hook_so_a_panic_leaves_its_note() {
    purba_execution::set_hooks().expect("purba's signal handler takes SIGINT");

    let _ = panic::catch_unwind(|| panic!("a panic after set_hooks"));
    assert_eq!(
        take(),
        Some(Panic {
            part: Some("execution".to_owned()),
            message: "a panic after set_hooks".to_owned(),
        }),
        "set_hooks wraps Rust's panic hook, so the wrap notes the panic and its part"
    );
}
