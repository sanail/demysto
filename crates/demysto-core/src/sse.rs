//! Server-Sent Events, as much of them as the streaming half of the Chat
//! Completions contract uses.
//!
//! The bytes arrive from a socket in whatever sizes the network chose, so an
//! event may be split across two of them, or three, or arrive with four others
//! in one. This decoder holds what is not yet a whole event and answers with
//! the ones that are.
//!
//! Tested here rather than through the facade: what is worth asserting is where
//! the chunk boundaries fall, and nothing above a socket gets to choose those.

/// A decoder that turns chunks of bytes into the `data` payloads of events.
#[derive(Default)]
pub(crate) struct Events {
    /// Bytes received that are not yet a whole event.
    ///
    /// Kept as bytes rather than text because a chunk boundary may fall inside
    /// a character as readily as inside a line.
    pending: Vec<u8>,
}

impl Events {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Takes a chunk and answers with the payload of every event it completed.
    pub(crate) fn feed(&mut self, chunk: &[u8]) -> Vec<String> {
        self.pending.extend_from_slice(chunk);

        let mut payloads = Vec::new();
        while let Some((end, resume)) = blank_line(&self.pending) {
            let event: Vec<u8> = self.pending.drain(..resume).collect();
            payloads.extend(data(&String::from_utf8_lossy(&event[..end])));
        }

        payloads
    }

    /// The payload of an event the sender ended without a blank line after it,
    /// which is how a stream that stops at its last event leaves things.
    pub(crate) fn finish(&mut self) -> Vec<String> {
        let pending = std::mem::take(&mut self.pending);

        data(&String::from_utf8_lossy(&pending))
            .into_iter()
            .collect()
    }
}

/// Where the next event ends, and where the one after it begins.
///
/// Events are separated by a blank line, which either half of the wire may
/// write with carriage returns or without.
fn blank_line(bytes: &[u8]) -> Option<(usize, usize)> {
    (0..bytes.len()).find_map(|at| {
        let rest = &bytes[at..];

        if rest.starts_with(b"\n\n") {
            Some((at, at + 2))
        } else if rest.starts_with(b"\r\n\r\n") {
            Some((at, at + 4))
        } else {
            None
        }
    })
}

/// One event's payload: the text of its `data` fields, or `None` when it
/// carried none — a comment holding the connection open, or a field this has
/// no use for.
fn data(event: &str) -> Option<String> {
    let mut payload: Option<String> = None;

    for value in event.lines().filter_map(|line| line.strip_prefix("data:")) {
        // Exactly one space, and only if it is there: the rest of the line is
        // the value, spaces of its own included.
        let value = value.strip_prefix(' ').unwrap_or(value);

        match &mut payload {
            Some(payload) => {
                payload.push('\n');
                payload.push_str(value);
            }
            None => payload = Some(value.to_owned()),
        }
    }

    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Everything a decoder makes of one string of bytes, fed in one piece.
    fn decoding(body: &str) -> Vec<String> {
        let mut events = Events::new();
        let mut payloads = events.feed(body.as_bytes());
        payloads.extend(events.finish());

        payloads
    }

    /// The same, with the body cut into pieces of `size` bytes.
    fn decoding_in_pieces(body: &str, size: usize) -> Vec<String> {
        let mut events = Events::new();
        let mut payloads = Vec::new();

        for piece in body.as_bytes().chunks(size) {
            payloads.extend(events.feed(piece));
        }
        payloads.extend(events.finish());

        payloads
    }

    #[test]
    fn an_event_is_the_text_after_its_data_field() {
        assert_eq!(decoding("data: one\n\n"), ["one"]);
    }

    #[test]
    fn the_space_after_the_colon_is_optional() {
        assert_eq!(decoding("data:one\n\n"), ["one"]);
    }

    #[test]
    fn only_the_first_space_belongs_to_the_field() {
        assert_eq!(decoding("data:  one\n\n"), [" one"]);
    }

    #[test]
    fn several_events_in_one_chunk_all_arrive() {
        assert_eq!(decoding("data: one\n\ndata: two\n\n"), ["one", "two"]);
    }

    #[test]
    fn several_data_lines_in_one_event_are_joined_by_newlines() {
        assert_eq!(decoding("data: one\ndata: two\n\n"), ["one\ntwo"]);
    }

    #[test]
    fn a_comment_is_not_an_event() {
        // What a Provider sends to hold the connection open while a Model thinks.
        assert_eq!(decoding(": keep-alive\n\ndata: one\n\n"), ["one"]);
    }

    #[test]
    fn fields_other_than_data_are_ignored() {
        assert_eq!(decoding("event: message\nid: 7\ndata: one\n\n"), ["one"]);
    }

    #[test]
    fn an_event_with_no_data_field_yields_nothing() {
        assert_eq!(decoding("event: ping\n\n"), Vec::<String>::new());
    }

    #[test]
    fn carriage_returns_are_tolerated() {
        assert_eq!(
            decoding("data: one\r\n\r\ndata: two\r\n\r\n"),
            ["one", "two"]
        );
    }

    #[test]
    fn a_last_event_the_sender_did_not_terminate_still_arrives() {
        assert_eq!(decoding("data: one\n\ndata: two"), ["one", "two"]);
    }

    #[test]
    fn a_stream_that_ends_where_it_should_yields_nothing_further() {
        let mut events = Events::new();

        assert_eq!(events.feed(b"data: one\n\n"), ["one"]);
        assert_eq!(events.finish(), Vec::<String>::new());
    }

    #[test]
    fn a_boundary_anywhere_in_the_body_leaves_the_events_unchanged() {
        let body = "data: one\n\ndata: two\ndata: three\n\n: keep-alive\n\ndata: four\n\n";
        let whole = decoding(body);

        for size in 1..=body.len() {
            assert_eq!(
                decoding_in_pieces(body, size),
                whole,
                "cut into {size}-byte pieces"
            );
        }
    }

    #[test]
    fn a_boundary_inside_a_character_does_not_corrupt_it() {
        // A chunk ends wherever the network ended it, which is as likely to be
        // inside a multi-byte character as anywhere else.
        let body = "data: ceci n'est pas une pipe — être\n\n";

        for size in 1..=body.len() {
            assert_eq!(
                decoding_in_pieces(body, size),
                ["ceci n'est pas une pipe — être"],
                "cut into {size}-byte pieces"
            );
        }
    }

    #[test]
    fn nothing_at_all_is_no_event() {
        assert_eq!(decoding(""), Vec::<String>::new());
    }
}
