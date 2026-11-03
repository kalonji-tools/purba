use std::hint::black_box;
use std::panic;
use std::sync::atomic::{AtomicUsize, Ordering};

use purba_execution::Error;
use purba_execution::panic::{Panic, take, wrap_hook};

// A panic hook belongs to the process, and cargo runs the tests of one binary
// on several threads. A panic in another test could take the place of a panic
// made here. So each panic in this process lives in this one test, alone in
// its binary.
#[test]
fn the_wrap_records_each_panic_and_calls_the_hook_it_replaced() {
    static REPLACED: AtomicUsize = AtomicUsize::new(0);
    panic::set_hook(Box::new(|_| {
        REPLACED.fetch_add(1, Ordering::SeqCst);
    }));
    wrap_hook();

    let _ = panic::catch_unwind(|| panic!("a panic in this file"));
    assert_eq!(
        take(),
        Some(Panic {
            part: Some("execution".to_owned()),
            message: "a panic in this file".to_owned(),
        }),
        "a panic in the source of execution names execution"
    );
    assert_eq!(take(), None, "take clears the note");

    let _ = panic::catch_unwind(|| Vec::<u64>::with_capacity(black_box(usize::MAX)));
    assert_eq!(
        take(),
        Some(Panic {
            part: None,
            message: "capacity overflow".to_owned(),
        }),
        "a panic inside alloc names no part"
    );

    let _ = panic::catch_unwind(|| panic::panic_any(5_u8));
    assert_eq!(
        take().map(|panic| panic.message),
        Some("Box<dyn Any>".to_owned()),
        "a payload that is not text gives what Rust's hook writes"
    );

    let error = purba_execution::catch("dispatch", || -> Result<(), Error> {
        panic!("a panic that the note holds")
    })
    .expect_err("a panic gives an error");
    assert_eq!(
        (error.part(), error.to_string()),
        (Some("execution"), "a panic that the note holds".to_owned()),
        "the note names the part of a panic that it holds"
    );

    let _ = panic::catch_unwind(|| panic!("a panic that a test caught"));
    let error = purba_execution::catch("dispatch", || -> Result<(), Error> {
        panic::resume_unwind(Box::new("resumed"))
    })
    .expect_err("a panic gives an error");
    assert_eq!(
        (error.part(), error.to_string()),
        (Some("dispatch"), "resumed".to_owned()),
        "a note that holds another message leaves the part to the catcher"
    );
    assert_eq!(take(), None, "catch takes the note");

    assert_eq!(
        REPLACED.load(Ordering::SeqCst),
        5,
        "the hook that the wrap replaced runs once for each panic, and never for a resumed one"
    );
}
