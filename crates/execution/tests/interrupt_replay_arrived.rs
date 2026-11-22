//! A signal that arrives as purba waits for a replay makes the case
//! interrupted, though the replay keeps none.

mod common;

use std::path::Path;

use purba_execution::{Kind, Purba};

/// A test whose thread makes the zygote replay its second expectation. The
/// case's process marks `ran`, so only the replay finds it. The replay marks
/// `replayed`, gives SIGINT the default action, sends SIGINT to purba, and
/// waits. purba's native handler notes the signal as it arrives, and forwards
/// it to the replay, which dies by it and sends no `Kept`.
fn replay_sends_sigint(ran: &str, replayed: &str, purba: u32) -> String {
    format!(
        "import os\nimport signal\nimport threading\nimport time\nfrom pathlib import Path\n\n\
         def test_one():\n    \
         threading.Thread(target=threading.Event().wait, daemon=True).start()\n    \
         if Path({ran:?}).exists():\n        \
         Path({replayed:?}).touch()\n        \
         signal.signal(signal.SIGINT, signal.SIG_DFL)\n        \
         os.kill({purba}, signal.SIGINT)\n        \
         time.sleep(10)\n    \
         Path({ran:?}).touch()\n    \
         assert 1 == 1\n    \
         assert 2 == 2\n"
    )
}

/// Whether the marker at `path` exists, and removes it if it does.
fn take(path: &Path) -> bool {
    let exists = path.exists();
    if exists {
        std::fs::remove_file(path).expect("the marker goes");
    }
    exists
}

// purba's signal handler keeps the first signal for the life of the process.
// So the one signal of this binary lives in this one test.
#[test]
fn a_sigint_that_arrives_as_purba_waits_for_a_replay_makes_the_case_interrupted() {
    let marker = |name: &str| {
        std::env::temp_dir().join(format!("purba-replay-{name}-{}", std::process::id()))
    };
    let (ran, replayed) = (marker("ran"), marker("replayed"));
    // A marker that an earlier run left would make the case's process take the
    // branch of the replay.
    take(&ran);
    take(&replayed);
    let module = common::module(
        "test_replay.py",
        &replay_sends_sigint(
            ran.to_str().expect("the marker's path is text"),
            replayed.to_str().expect("the marker's path is text"),
            std::process::id(),
        ),
    );
    let (result, kept) = purba_execution::attach(|purba: &Purba<'_>| {
        purba_execution::set_hooks().expect("purba's signal handler takes SIGINT");
        let (mut zygote, _) = purba.zygote(&module).expect("the zygote loads the module");
        let (result, _, _) = zygote.run("test_one", &[]);
        zygote.end();
        (
            result
                .map(|outcome| outcome.kind().clone())
                .map_err(|error| error.to_string()),
            purba_execution::interrupted(),
        )
    });
    take(&ran);
    assert_eq!(
        (result, kept, take(&replayed)),
        (Ok(Kind::Interrupted), true, true),
        "a SIGINT that arrives as a replay runs, while purba waits for it, makes the case \
         interrupted, though the replay died by it and kept none, and purba keeps it"
    );
}
