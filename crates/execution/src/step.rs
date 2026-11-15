//! What a load or a call met, and the result that the step gives.

use crate::{Error, ErrorKind, panic};

/// What a load or a call met besides its own result, from the opening of its
/// window to its end.
pub struct Met {
    /// A panic that the panic hook noted, which the step caught.
    pub panic: Option<panic::Panic>,
    /// Whether the first interrupt landed in the step.
    pub interrupted: bool,
}

/// The result that a step gives: its own, or the one that hides it.
#[derive(Debug)]
pub enum Gives<T> {
    Own(T),
    Interrupted,
}

/// A panic, then an interrupt, then the step's own error, hide each one after
/// them. An internal error that the step gives itself ranks as a panic.
pub fn decide<T>(own: Result<T, Error>, met: Met) -> Result<Gives<T>, Error> {
    if let Some(noted) = met.panic {
        Err(Error(crate::Reason::Internal(noted.caught_by("execution"))))
    } else if let Err(error) = own.as_ref()
        && error.kind() != ErrorKind::Project
    {
        own.map(Gives::Own)
    } else if met.interrupted {
        Ok(Gives::Interrupted)
    } else {
        own.map(Gives::Own)
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
    };

    /// The result that `decide` gives, as text a row can compare.
    fn shown(result: Result<Gives<Kind>, Error>) -> String {
        match result {
            Ok(Gives::Own(kind)) => format!("own: {kind:?}"),
            Ok(Gives::Interrupted) => "interrupted".to_owned(),
            Err(error) => match error.kind() {
                ErrorKind::Project => format!("project error: {error}"),
                ErrorKind::Internal { part } => format!("internal error: {part}: {error}"),
            },
        }
    }

    #[test]
    fn a_panic_an_interrupt_and_the_steps_own_error_hide_each_one_after_them() {
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
                },
                "internal error: execution: boom",
            ),
            // fails when: the project error moves above the check of the panic
            (
                "a step that met panic and project error results in internal error",
                Err(Error::project("...".to_owned())),
                Met {
                    panic: a_panic,
                    interrupted: false,
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
                },
                "interrupted",
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
                },
                "internal error: execution: crossed",
            ),
        ];
        for (row, own, met, gives) in rows {
            assert_eq!(shown(decide(own, met)), gives, "{row}");
        }
    }
}
