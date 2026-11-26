//! The frame that crosses a pipe between purba, a zygote and a test's process:
//! a tag of one byte, the length of the text in 4 bytes, and the text in UTF-8.
//! Each frame holds one message.

use std::io::{self, ErrorKind, Read, Write};
use std::str::FromStr;

use purba_assertion::{Swallowed, Try, Verdict};
use purba_errors::Shown;

use crate::order::{Called, Facts, How, PathFault};
use crate::{Error, internal, panic};

/// Writes one frame of `tag` and `text`.
///
/// # Errors
///
/// The pipe refused the write, or `text` is longer than 4 bytes can say.
fn write(to: &mut impl Write, tag: u8, text: &str) -> io::Result<()> {
    let length = u32::try_from(text.len()).map_err(io::Error::other)?;
    let mut frame = Vec::with_capacity(5 + text.len());
    frame.push(tag);
    frame.extend(length.to_be_bytes());
    frame.extend(text.as_bytes());
    to.write_all(&frame)
}

/// Reads one frame: its tag and its text. Gives `None` when the pipe ends before
/// a frame starts.
///
/// # Errors
///
/// `UnexpectedEof` when the pipe ends inside a frame, `InvalidData` when the
/// text is not UTF-8, or the error of the read.
fn read(from: &mut impl Read) -> io::Result<Option<(u8, String)>> {
    let mut tag = [0; 1];
    loop {
        match from.read(&mut tag) {
            Ok(0) => return Ok(None),
            Ok(_) => break,
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    let mut length = [0; 4];
    from.read_exact(&mut length)?;
    let length = usize::try_from(u32::from_be_bytes(length)).map_err(io::Error::other)?;
    let mut text = vec![0; length];
    from.read_exact(&mut text)?;
    String::from_utf8(text)
        .map(|text| Some((tag[0], text)))
        .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))
}

/// Each message that crosses a pipe between purba, a zygote and a test's
/// process. Each one is one frame: the tag of its kind, then each of its
/// fields, as `put` writes it. A field holds any text, a NUL too.
#[derive(Debug)]
pub enum Message {
    /// purba names the test to run: the name of its function.
    Test { function: String },
    /// purba asks for a replay of the test `function` that reads the
    /// expectation at `index` alone.
    Replay { function: String, index: usize },
    /// The load ended, and the zygote forks each test from now on.
    Ready,
    /// A thread is alive after the load, and the zygote ends.
    Threaded,
    /// The load returned, and kept its module or met an interrupt.
    Loaded,
    /// The test's process started: its pid.
    Forked(libc::pid_t),
    /// The facts of the case, from which purba decides its outcome.
    Facts(Box<Facts>),
    /// The first interrupt landed in the test's step, so the case is
    /// `interrupted`. It comes after the facts, when the call ran.
    Interrupted,
    /// The error in place of the result of a load or the outcome of a case.
    Error(Error),
    /// The error of a thread that the test left, as text.
    TestThread(String),
    /// The error of a thread that the load left, as text.
    LoadThread(String),
    /// The first interrupt that the process kept: the number of its signal.
    Kept(i32),
    /// What a replay read of its expectation.
    Verdict(Verdict),
    /// The zygote could not fork a replay: its words.
    Refused(String),
    /// The test's process or the replay ended: its wait status.
    Ended(libc::c_int),
}

/// The tag of each kind of message.
mod tag {
    pub const TEST: u8 = b'n';
    pub const REPLAY: u8 = b'y';
    pub const READY: u8 = b'g';
    pub const THREADED: u8 = b't';
    pub const LOADED: u8 = b'l';
    pub const FORKED: u8 = b'c';
    pub const FACTS: u8 = b'a';
    pub const INTERRUPTED: u8 = b'i';
    pub const PROJECT: u8 = b'e';
    pub const INTERNAL: u8 = b'x';
    pub const TEST_THREAD: u8 = b'h';
    pub const LOAD_THREAD: u8 = b'H';
    pub const KEPT: u8 = b'k';
    pub const VERDICT: u8 = b'v';
    pub const REFUSED: u8 = b'r';
    pub const ENDED: u8 = b'w';
}

impl Message {
    /// Writes the message as one frame.
    pub fn write(&self, to: &mut impl Write) -> io::Result<()> {
        let mut text = String::new();
        let mut put = |field: &str| put(&mut text, field);
        let tag = match self {
            Self::Test { function } => {
                put(function);
                tag::TEST
            }
            Self::Replay { function, index } => {
                put(function);
                put(&index.to_string());
                tag::REPLAY
            }
            Self::Ready => tag::READY,
            Self::Threaded => tag::THREADED,
            Self::Loaded => tag::LOADED,
            Self::Forked(pid) => {
                put(&pid.to_string());
                tag::FORKED
            }
            Self::Facts(facts) => {
                put_facts(&mut put, facts);
                tag::FACTS
            }
            Self::Interrupted => tag::INTERRUPTED,
            // An error's `Display` is the text of a project error, and the
            // message of an internal error.
            Self::Error(error) => match error.kind() {
                purba_errors::ErrorKind::Project => {
                    put(&error.to_string());
                    tag::PROJECT
                }
                purba_errors::ErrorKind::Internal { part } => {
                    put(part);
                    put(&error.to_string());
                    tag::INTERNAL
                }
            },
            Self::TestThread(error) => {
                put(error);
                tag::TEST_THREAD
            }
            Self::LoadThread(error) => {
                put(error);
                tag::LOAD_THREAD
            }
            Self::Kept(signum) => {
                put(&signum.to_string());
                tag::KEPT
            }
            Self::Verdict(verdict) => {
                put(letter(*verdict));
                tag::VERDICT
            }
            Self::Refused(words) => {
                put(words);
                tag::REFUSED
            }
            Self::Ended(status) => {
                put(&status.to_string());
                tag::ENDED
            }
        };
        write(to, tag, &text)
    }

    /// Reads one message. Gives `None` when the pipe ends before a frame
    /// starts. A frame that holds no message gives an internal error of
    /// execution in its place.
    pub fn read(from: &mut impl Read) -> io::Result<Option<Result<Self, Error>>> {
        Ok(read(from)?.map(|(tag, text)| {
            Self::of(tag, &text).ok_or_else(|| {
                internal(panic::Caught {
                    part: "execution".to_owned(),
                    message: format!("the frame {:?} holds no message", char::from(tag)),
                })
            })
        }))
    }

    /// The message that the frame of `tag` and `text` holds, or `None` when it
    /// holds none.
    fn of(tag: u8, text: &str) -> Option<Self> {
        let mut fields = Fields(text);
        let message = match tag {
            tag::TEST => Self::Test {
                function: fields.text()?,
            },
            tag::REPLAY => Self::Replay {
                function: fields.text()?,
                index: fields.number()?,
            },
            tag::READY => Self::Ready,
            tag::THREADED => Self::Threaded,
            tag::LOADED => Self::Loaded,
            tag::FORKED => Self::Forked(fields.number()?),
            tag::FACTS => Self::Facts(Box::new(fields.facts()?)),
            tag::INTERRUPTED => Self::Interrupted,
            tag::PROJECT => Self::Error(Error::project(fields.text()?)),
            tag::INTERNAL => Self::Error(internal(panic::Caught {
                part: fields.text()?,
                message: fields.text()?,
            })),
            tag::TEST_THREAD => Self::TestThread(fields.text()?),
            tag::LOAD_THREAD => Self::LoadThread(fields.text()?),
            tag::KEPT => Self::Kept(fields.number()?),
            tag::VERDICT => Self::Verdict(fields.verdict()?),
            tag::REFUSED => Self::Refused(fields.text()?),
            tag::ENDED => Self::Ended(fields.number()?),
            _ => return None,
        };
        fields.0.is_empty().then_some(message)
    }
}

/// The fields of a frame that are left to read.
struct Fields<'a>(&'a str);

impl Fields<'_> {
    /// Takes the next field, which `put` wrote.
    fn text(&mut self) -> Option<String> {
        let (length, rest) = self.0.split_once('\0')?;
        let length: usize = length.parse().ok()?;
        let field = rest.get(..length)?;
        self.0 = &rest[length..];
        Some(field.to_owned())
    }

    /// Takes the next field as a number.
    fn number<N: FromStr>(&mut self) -> Option<N> {
        self.text()?.parse().ok()
    }

    /// Takes the next field as a verdict, which `letter` wrote.
    fn verdict(&mut self) -> Option<Verdict> {
        Some(match self.text()?.as_str() {
            "h" => Verdict::Holds,
            "f" => Verdict::Fails,
            "r" => Verdict::Raises,
            "x" => Verdict::Refused,
            _ => return None,
        })
    }

    /// Takes the next field as a flag, which `put_facts` wrote as `1` or `0`.
    fn flag(&mut self) -> Option<bool> {
        match self.text()?.as_str() {
            "1" => Some(true),
            "0" => Some(false),
            _ => None,
        }
    }

    /// Takes a count, then that many items, each of which `item` takes.
    fn list<I>(&mut self, mut item: impl FnMut(&mut Self) -> Option<I>) -> Option<Vec<I>> {
        let count: usize = self.number()?;
        (0..count).map(|_| item(self)).collect()
    }

    /// Takes an option, which `put_facts` wrote as a list of no field or one.
    /// Gives the list, which `pop` turns into the option.
    fn optional(&mut self) -> Option<Vec<String>> {
        let list = self.list(Self::text)?;
        (list.len() <= 1).then_some(list)
    }

    /// Takes the facts of a case, which `put_facts` wrote.
    fn facts(&mut self) -> Option<Facts> {
        let pair = |fields: &mut Self| Some((fields.text()?, fields.text()?));
        Some(Facts {
            form: self.list(pair)?,
            called: if self.flag()? {
                Called::Raised {
                    exception: self.text()?,
                    whence: self.optional()?.pop(),
                    assertion: self.flag()?,
                }
            } else {
                Called::Returned
            },
            with: self.optional()?.pop(),
            mark: if self.flag()? {
                Some((self.text()?, self.flag()?))
            } else {
                None
            },
            path: self.list(Self::path_fault)?,
            swallowed: self.list(|fields| {
                Some(Swallowed {
                    with: fields.number()?,
                    skipped: fields.number()?,
                    last: fields.number()?,
                })
            })?,
            tries: self.list(|fields| {
                Some(Try {
                    lines: fields.number()?..=fields.number()?,
                    except: if fields.flag()? {
                        Some(fields.number()?..=fields.number()?)
                    } else {
                        None
                    },
                })
            })?,
            refused: self.optional()?.pop(),
            in_order: self.list(|fields| Some((fields.number()?, fields.flag()?)))?,
            shown: self.list(|fields| Some((fields.number()?, fields.shown()?)))?,
            alone: self.list(|fields| Some((fields.number()?, fields.verdict()?)))?,
            later: self.list(Self::number)?,
            texts: self.list(|fields| Some((fields.number()?, fields.text()?)))?,
            missed: self.list(Self::text)?,
            stated: self.flag()?,
        })
    }

    /// Takes a fault of a path, which `put_paths` wrote.
    fn path_fault(&mut self) -> Option<PathFault> {
        Some(PathFault {
            line: self.number()?,
            reached: if self.flag()? {
                Some((self.number()?, self.text()?))
            } else {
                None
            },
            how: match self.text()?.as_str() {
                "n" => How::DidNotRun,
                "o" => How::OneWay,
                _ => return None,
            },
        })
    }

    /// Takes what an expectation that did not hold shows.
    fn shown(&mut self) -> Option<Shown> {
        Some(match self.text()?.as_str() {
            "1" => Shown::Values {
                expected: self.text()?,
                reading: self.text()?,
            },
            "2" => Shown::Items {
                heading: self.text()?,
                items: self.list(Self::text)?,
            },
            _ => return None,
        })
    }
}

/// Writes three lists of `facts` through `put`: each fault of the paths, each
/// `with` that swallowed a raise, and each `try` of the test. Each list is its
/// count, then its items, as in [`put_facts`].
fn put_paths(put: &mut impl FnMut(&str), facts: &Facts) {
    put(&facts.path.len().to_string());
    for fault in &facts.path {
        put(&fault.line.to_string());
        match &fault.reached {
            Some((at, name)) => {
                put("1");
                put(&at.to_string());
                put(name);
            }
            None => put("0"),
        }
        put(match fault.how {
            How::DidNotRun => "n",
            How::OneWay => "o",
        });
    }
    put(&facts.swallowed.len().to_string());
    for swallow in &facts.swallowed {
        put(&swallow.with.to_string());
        put(&swallow.skipped.to_string());
        put(&swallow.last.to_string());
    }
    put(&facts.tries.len().to_string());
    for attempt in &facts.tries {
        put(&attempt.lines.start().to_string());
        put(&attempt.lines.end().to_string());
        match &attempt.except {
            Some(except) => {
                put("1");
                put(&except.start().to_string());
                put(&except.end().to_string());
            }
            None => put("0"),
        }
    }
}

/// Writes the fields of `facts` through `put`, in the order that
/// [`Fields::facts`] takes them. A list is its count, then its items, and an
/// option is a list of no item or one.
fn put_facts(put: &mut impl FnMut(&str), facts: &Facts) {
    let flag = |set: bool| if set { "1" } else { "0" };
    let option = |put: &mut dyn FnMut(&str), value: Option<&str>| {
        put(flag(value.is_some()));
        if let Some(value) = value {
            put(value);
        }
    };
    let pairs = |put: &mut dyn FnMut(&str), pairs: &[(String, String)]| {
        put(&pairs.len().to_string());
        for (first, second) in pairs {
            put(first);
            put(second);
        }
    };
    pairs(put, &facts.form);
    match &facts.called {
        Called::Returned => put("0"),
        Called::Raised {
            exception,
            whence,
            assertion,
        } => {
            put("1");
            put(exception);
            option(put, whence.as_deref());
            put(flag(*assertion));
        }
    }
    option(put, facts.with.as_deref());
    match &facts.mark {
        Some((assert, held)) => {
            put("1");
            put(assert);
            put(flag(*held));
        }
        None => put("0"),
    }
    put_paths(put, facts);
    option(put, facts.refused.as_deref());
    put(&facts.in_order.len().to_string());
    for (index, held) in &facts.in_order {
        put(&index.to_string());
        put(flag(*held));
    }
    put(&facts.shown.len().to_string());
    for (index, shown) in &facts.shown {
        put(&index.to_string());
        match shown {
            Shown::Values { expected, reading } => {
                put("1");
                put(expected);
                put(reading);
            }
            Shown::Items { heading, items } => {
                put("2");
                put(heading);
                put(&items.len().to_string());
                for item in items {
                    put(item);
                }
            }
        }
    }
    put(&facts.alone.len().to_string());
    for (index, verdict) in &facts.alone {
        put(&index.to_string());
        put(letter(*verdict));
    }
    put(&facts.later.len().to_string());
    for index in &facts.later {
        put(&index.to_string());
    }
    put(&facts.texts.len().to_string());
    for (index, text) in &facts.texts {
        put(&index.to_string());
        put(text);
    }
    put(&facts.missed.len().to_string());
    for words in &facts.missed {
        put(words);
    }
    put(flag(facts.stated));
}

/// The letter of `verdict` in a frame.
const fn letter(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Holds => "h",
        Verdict::Fails => "f",
        Verdict::Raises => "r",
        Verdict::Refused => "x",
    }
}

/// Writes `field` after its length in bytes and a NUL, so that a field holds
/// any text, a NUL too.
fn put(frame: &mut String, field: &str) {
    frame.push_str(&field.len().to_string());
    frame.push('\0');
    frame.push_str(field);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn a_frame_reads_back_as_its_tag_and_its_text() {
        let mut pipe = Vec::new();
        write(&mut pipe, b'r', "ValueError: é").expect("a Vec takes the write");
        write(&mut pipe, b'p', "").expect("a Vec takes the write");
        let mut from = pipe.as_slice();
        assert_eq!(
            read(&mut from).expect("a whole frame reads"),
            Some((b'r', "ValueError: é".to_owned())),
            "the first frame, with text that is not ASCII"
        );
        assert_eq!(
            read(&mut from).expect("a whole frame reads"),
            Some((b'p', String::new())),
            "the second frame, with no text"
        );
        assert_eq!(
            read(&mut from).expect("the end reads"),
            None,
            "the pipe ends between frames"
        );
    }

    #[test]
    fn a_frame_shorter_than_its_length_is_an_unexpected_end() {
        let mut pipe = Vec::new();
        write(&mut pipe, b'r', "ValueError: boom").expect("a Vec takes the write");
        for cut in 1..pipe.len() {
            let error = read(&mut &pipe[..cut]).expect_err("a cut frame does not read");
            assert_eq!(
                error.kind(),
                ErrorKind::UnexpectedEof,
                "a frame cut after {cut} bytes"
            );
        }
    }

    /// Facts that hold a fault of a path of each shape, a `with` that swallowed
    /// a raise, and a `try` of each shape, with `text` as the name of the code
    /// that a fault reached.
    fn facts_of_the_paths(text: impl Fn(&str) -> String) -> Facts {
        Facts {
            path: vec![
                PathFault {
                    line: 3,
                    reached: None,
                    how: How::DidNotRun,
                },
                PathFault {
                    line: 4,
                    reached: Some((12, text("key"))),
                    how: How::OneWay,
                },
            ],
            swallowed: vec![Swallowed {
                with: 6,
                skipped: 8,
                last: 11,
            }],
            tries: vec![
                Try {
                    lines: 8..=11,
                    except: None,
                },
                Try {
                    lines: 13..=16,
                    except: Some(15..=16),
                },
            ],
            ..Facts::default()
        }
    }

    /// One message of each variant, with a NUL, a line feed and text that is
    /// not ASCII in its fields, an empty field, and each shape of a field that
    /// is optional.
    fn each_variant() -> Vec<Message> {
        let text = |words: &str| format!("{words}\0\n é");
        vec![
            Message::Test {
                function: text("test_one"),
            },
            Message::Replay {
                function: text("test_one"),
                index: 2,
            },
            Message::Ready,
            Message::Threaded,
            Message::Loaded,
            Message::Forked(4321),
            Message::Facts(Box::new(Facts {
                form: vec![
                    (text("iterator-read"), text("words")),
                    (String::new(), String::new()),
                ],
                called: Called::Raised {
                    exception: text("ValueError: no"),
                    whence: Some(text("helper")),
                    assertion: true,
                },
                with: Some(text("with raises(ValueError):")),
                mark: Some((text("assert x == 1"), false)),
                refused: Some(text("purba cannot fork")),
                in_order: vec![(1, true), (2, false)],
                shown: vec![
                    (
                        2,
                        Shown::Values {
                            expected: text("1"),
                            reading: text("2"),
                        },
                    ),
                    (
                        3,
                        Shown::Items {
                            heading: text("the first item that breaks"),
                            items: vec![text("0"), String::new()],
                        },
                    ),
                ],
                alone: vec![
                    (2, Verdict::Holds),
                    (3, Verdict::Fails),
                    (4, Verdict::Raises),
                    (5, Verdict::Refused),
                ],
                later: vec![2, 3],
                texts: vec![(1, text("assert x == 1")), (2, String::new())],
                missed: vec![text("try: x()")],
                stated: true,
                ..facts_of_the_paths(text)
            })),
            Message::Facts(Box::new(Facts {
                called: Called::Raised {
                    exception: text("ValueError: no"),
                    whence: None,
                    assertion: false,
                },
                ..Facts::default()
            })),
            Message::Facts(Box::default()),
            Message::Interrupted,
            Message::Error(Error::project(text("[no-expectation] no expectation ran"))),
            Message::Error(internal(panic::Caught {
                part: text("assertion"),
                message: text("a panic"),
            })),
            Message::TestThread(text("thread A raised ValueError")),
            Message::LoadThread(text("thread L raised ValueError")),
            Message::Kept(libc::SIGTERM),
            Message::Verdict(Verdict::Holds),
            Message::Verdict(Verdict::Fails),
            Message::Verdict(Verdict::Raises),
            Message::Verdict(Verdict::Refused),
            Message::Refused(text("purba cannot fork to read an expectation alone")),
            Message::Ended(256),
        ]
    }

    /// The number of arms of `variant`, one for each variant of `Message`.
    const VARIANTS: usize = 15;

    /// The name of the variant of `message`. A variant that this match lacks
    /// fails to compile. The count above it must then grow, and
    /// `each_variant` must send the new one.
    const fn variant(message: &Message) -> &'static str {
        match message {
            Message::Test { .. } => "Test",
            Message::Replay { .. } => "Replay",
            Message::Ready => "Ready",
            Message::Threaded => "Threaded",
            Message::Loaded => "Loaded",
            Message::Forked(_) => "Forked",
            Message::Facts(_) => "Facts",
            Message::Interrupted => "Interrupted",
            Message::Error(_) => "Error",
            Message::TestThread(_) => "TestThread",
            Message::LoadThread(_) => "LoadThread",
            Message::Kept(_) => "Kept",
            Message::Verdict(_) => "Verdict",
            Message::Refused(_) => "Refused",
            Message::Ended(_) => "Ended",
        }
    }

    #[test]
    fn each_message_reads_back_as_itself() {
        let sent = each_variant();
        let mut pipe = Vec::new();
        for message in &sent {
            message.write(&mut pipe).expect("a Vec takes the write");
        }
        let mut from = pipe.as_slice();
        for message in &sent {
            let read = Message::read(&mut from)
                .expect("a whole frame reads")
                .expect("a frame is left")
                .expect("the frame holds a message");
            assert_eq!(
                format!("{read:?}"),
                format!("{message:?}"),
                "a {} message reads back as itself",
                variant(message)
            );
        }
        assert!(
            Message::read(&mut from).expect("the end reads").is_none(),
            "the pipe ends after the last message"
        );
        let mut names: Vec<&str> = sent.iter().map(variant).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), VARIANTS, "each variant is sent: {names:?}");
    }

    /// The internal error of a frame of `tag` that holds no message.
    fn no_message(tag: char) -> String {
        format!(
            "{:?}",
            internal(panic::Caught {
                part: "execution".to_owned(),
                message: format!("the frame {tag:?} holds no message"),
            })
        )
    }

    /// The bytes of a frame that holds no message, and the `Debug` text of the
    /// internal error that it reads as.
    pub fn holding_no_message() -> (Vec<u8>, String) {
        let mut pipe = Vec::new();
        write(&mut pipe, tag::ENDED, "1\0x").expect("a Vec takes the write");
        (pipe, no_message(char::from(tag::ENDED)))
    }

    #[test]
    fn a_frame_that_holds_no_message_reads_as_an_internal_error() {
        let frames: [(u8, &str, &str); 8] = [
            (tag::FACTS, "9\0cut", "a field longer than the frame"),
            (b'?', "", "a tag of no message"),
            (tag::FORKED, "1\0x", "a pid that is no number"),
            (tag::REPLAY, "1\0a", "a replay with one of its two fields"),
            (tag::ENDED, "1\0x", "a wait status that is no number"),
            (tag::KEPT, "1\0x", "a signal that is no number"),
            (tag::READY, "1\0x", "a field left over"),
            (
                tag::FACTS,
                "1\x0001\x002",
                "a flag of the facts that is neither 0 nor 1",
            ),
        ];
        let mut pipe = Vec::new();
        for (tag, text, _) in frames {
            write(&mut pipe, tag, text).expect("a Vec takes the write");
        }
        let mut from = pipe.as_slice();
        for (tag, _, what) in frames {
            let read = Message::read(&mut from)
                .expect("a whole frame reads")
                .expect("a frame is left");
            assert_eq!(
                read.map_err(|error| format!("{error:?}"))
                    .map(|message| format!("{message:?}")),
                Err(no_message(char::from(tag))),
                "{what}"
            );
        }
    }
}
