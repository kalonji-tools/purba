//! What a load or a call met, and the result that the step gives.

use crate::{Error, ErrorKind, panic};

/// What a load or a call met besides its own result, from the opening of its
/// window to its end.
pub struct Met {
    /// A panic that the panic hook noted, which the step caught.
    pub panic: Option<panic::Panic>,
    /// Whether the first interrupt landed in the step.
    pub interrupted: bool,
    /// The call to `os._exit` that the guard kept, as text.
    pub exit: Option<String>,
}

/// The result that a step gives: its own, or the one that hides it.
#[derive(Debug)]
pub enum Gives<T> {
    Own(T),
    Interrupted,
    Exit(String),
}

/// A panic, then an interrupt, then the step's own error, then a call to
/// `os._exit`, hide each one after them. An internal error that the step gives
/// itself ranks as a panic.
pub fn decide<T>(own: Result<T, Error>, met: Met) -> Result<Gives<T>, Error> {
    if let Some(noted) = met.panic {
        Err(Error(crate::Reason::Internal(noted.caught_by("execution"))))
    } else if let Err(error) = own.as_ref()
        && error.kind() != ErrorKind::Project
    {
        own.map(Gives::Own)
    } else if met.interrupted {
        Ok(Gives::Interrupted)
    } else if let Err(error) = own {
        Err(error)
    } else if let Some(exit) = met.exit {
        Ok(Gives::Exit(exit))
    } else {
        Ok(Gives::Own(own?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Kind;

    /// What a step met: nothing.
    const NOTHING: Met = Met {
        panic: None,
        interrupted: false,
        exit: None,
    };

    /// The result that `decide` gives, as text a row can compare.
    fn shown(result: Result<Gives<Kind>, Error>) -> String {
        match result {
            Ok(Gives::Own(kind)) => format!("own: {kind:?}"),
            Ok(Gives::Interrupted) => "interrupted".to_owned(),
            Ok(Gives::Exit(call)) => format!("exit: {call}"),
            Err(error) => match error.kind() {
                ErrorKind::Project => format!("project error: {error}"),
                ErrorKind::Internal { part } => format!("internal error: {part}: {error}"),
            },
        }
    }

    #[test]
    fn a_panic_an_interrupt_the_steps_own_error_and_a_call_to_os_exit_hide_each_one_after_them() {
        // Each row: what it checks, the step's own result, what the step met,
        // and what `decide` gives.
        let a_panic = Some(panic::Panic {
            part: Some("execution".to_owned()),
            message: "boom".to_owned(),
        });
        let rows: Vec<(&str, Result<Kind, Error>, Met, &str)> = vec![
            // fails when: the last line gives `Gives::Interrupted`, not the step's own result
            (
                "a step that met nothing gives its own result",
                Ok(Kind::Passed),
                NOTHING,
                "own: Passed",
            ),
            // fails when: the check of the interrupt moves above the check of the panic
            (
                "a step that met panic and an interrupt results in internal error",
                Ok(Kind::Passed),
                Met {
                    panic: a_panic.clone(),
                    interrupted: true,
                    exit: None,
                },
                "internal error: execution: boom",
            ),
            // fails when: the project error moves above the check of the panic
            (
                "a step that met panic and project error results in internal error",
                Err(Error::project("...".to_owned())),
                Met {
                    panic: a_panic.clone(),
                    interrupted: false,
                    exit: None,
                },
                "internal error: execution: boom",
            ),
            // fails when: the check of the os._exit moves above the check of the panic
            (
                "a step that met panic and os._exit results in internal error",
                Ok(Kind::Passed),
                Met {
                    panic: a_panic,
                    interrupted: false,
                    exit: Some("os._exit".to_string()),
                },
                "internal error: execution: boom",
            ),
            // fails when: the project error moves above the check of the interrupted
            (
                "a step that met interrupt and self error gives interrupted",
                Err(Error::project("...".to_owned())),
                Met {
                    panic: None,
                    interrupted: true,
                    exit: None,
                },
                "interrupted",
            ),
            // fails when: the check of the os._exit moves above the check of the interrupted
            (
                "a step that met interrupt and os._exit gives interrupted",
                Ok(Kind::Passed),
                Met {
                    panic: None,
                    interrupted: true,
                    exit: Some("os._exit".to_string()),
                },
                "interrupted",
            ),
            // fails when: the check of the os._exit moves above the project error
            (
                "a step that met os._exit and self error results in project error",
                Err(Error::project("...".to_owned())),
                Met {
                    panic: None,
                    interrupted: false,
                    exit: Some("os._exit".to_string()),
                },
                "project error: ...",
            ),
            // fails when: the check of the interrupt moves above the check of the own internal
            // error
            (
                "a step that met an interrupt and gave an internal error results in that error",
                Err(Error(crate::Reason::Internal(panic::Caught {
                    part: "execution".to_owned(),
                    message: "crossed".to_owned(),
                }))),
                Met {
                    panic: None,
                    interrupted: true,
                    exit: None,
                },
                "internal error: execution: crossed",
            ),
        ];
        for (row, own, met, gives) in rows {
            assert_eq!(shown(decide(own, met)), gives, "{row}");
        }
    }
}
