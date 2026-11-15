//! The frame that a zygote and a test's process write to purba on a pipe: a tag
//! of one byte, the length of the text in 4 bytes, and the text in UTF-8.

use std::io::{self, ErrorKind, Read, Write};

/// Writes one frame of `tag` and `text`.
///
/// # Errors
///
/// The pipe refused the write, or `text` is longer than 4 bytes can say.
pub fn write(to: &mut impl Write, tag: u8, text: &str) -> io::Result<()> {
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
pub fn read(from: &mut impl Read) -> io::Result<Option<(u8, String)>> {
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

#[cfg(test)]
mod tests {
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
}
