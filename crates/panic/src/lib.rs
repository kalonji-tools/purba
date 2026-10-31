//! PROTOTYPE: the wrap of Rust's panic hook, and its record, as a part of its own.
use std::panic;
use std::sync::{Mutex, Once, PoisonError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Panic {
    pub part: Option<String>,
    pub message: String,
}

static LAST: Mutex<Option<Panic>> = Mutex::new(None);

pub fn wrap_hook() {
    static WRAP: Once = Once::new();
    WRAP.call_once(|| {
        let hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let part = info.location().and_then(|place| part(place.file()));
            let message = info.payload_as_str().unwrap_or("Box<dyn Any>").to_owned();
            *LAST.lock().unwrap_or_else(PoisonError::into_inner) = Some(Panic { part, message });
            hook(info);
        }));
    });
}

pub fn take() -> Option<Panic> {
    LAST.lock().unwrap_or_else(PoisonError::into_inner).take()
}

fn part(file: &str) -> Option<String> {
    let rest = file.strip_prefix("crates/")?;
    let (part, _) = rest.split_once('/')?;
    (part != "purba").then(|| part.to_owned())
}
