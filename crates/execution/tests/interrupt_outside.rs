use pyo3::Python;

// purba's signal handler keeps the first signal for the life of the process,
// and a second signal ends the process. So this test lives alone in its
// binary.
#[test]
fn purbas_signal_handler_keeps_a_signal_outside_a_call_and_raises_nothing() {
    let (outside, kept) = purba_execution::attach(|worker| {
        purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
        let outside = Python::attach(|py| {
            py.run(
                c"import signal\nsignal.raise_signal(signal.SIGINT)",
                None,
                None,
            )
        })
        .map_err(|error| error.to_string());
        (outside, worker.interrupted())
    });
    assert_eq!(
        (outside, kept),
        (Ok(()), true),
        "purba keeps a signal outside a call, and raises nothing there"
    );
}
