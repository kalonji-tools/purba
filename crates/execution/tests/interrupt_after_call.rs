//! A signal that arrives after the last line of Python of a call does not land
//! in that call.

mod common;

use purba_execution::Purba;

/// A test that is C's `raise` of SIGINT, so its call runs no line of Python.
/// purba's native handler notes the signal as it arrives. Python's own C
/// handler only marks it for later. So purba's signal handler runs after the
/// call returned, at the next line of Python.
const SIGINT_AS_A_CALL: &str = "import ctypes\nimport functools\n\n\
    test_raise = functools.partial(ctypes.CDLL(None)['raise'], 2)\n";

// purba's signal handler keeps the first signal for the life of the process.
// So the one signal of this binary lives in this one test.
#[test]
fn a_sigint_that_arrives_after_the_last_line_of_a_call_leaves_the_case_to_its_facts() {
    let module = common::module("test_after.py", SIGINT_AS_A_CALL);
    let (result, kept) = purba_execution::attach(|purba: &Purba<'_>| {
        purba_execution::set_hooks().expect("purba's signal handler takes SIGINT");
        let (mut zygote, _) = purba.zygote(&module).expect("the zygote loads the module");
        let (result, _, _) = zygote.run("test_raise", &[]);
        zygote.end();
        (
            result
                .map(|outcome| outcome.kind().clone())
                .map_err(|error| error.to_string()),
            purba_execution::interrupted(),
        )
    });
    assert_eq!(
        (result, kept),
        (
            Err("[no-expectation] no expectation stated".to_owned()),
            true
        ),
        "a SIGINT that arrives after the last line of Python of a call does not make the case \
         interrupted: the facts of the case decide it, and purba keeps the signal"
    );
}
