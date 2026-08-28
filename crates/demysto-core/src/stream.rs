//! Stream assembly: what the user is shown while an answer is still arriving.
//!
//! Two things stand between a stream of fragments and something worth putting
//! on screen. A code fence that has been opened and not yet closed is not a code
//! block until it is, so a Markdown renderer shown the text as it stands would
//! draw the half-written block as paragraphs and then redraw it as a block —
//! flickering once per fragment. And a renderer asked to run on every fragment
//! runs some hundreds of times for one answer, to show differences no eye can
//! follow.
//!
//! So the accumulated text is closed off and handed over at a rate a reader can
//! use. This lives in Rust rather than the frontend on purpose: it is a
//! presentation concern moved across a layer in exchange for one test language
//! rather than two (the spec's *Further Notes* records the trade).

use std::time::{Duration, Instant};

/// How often the accumulated answer is handed over.
///
/// Fast enough that the text reads as arriving continuously, slow enough that a
/// Model emitting a token at a time does not ask the renderer for hundreds of
/// passes nobody sees.
pub(crate) const THROTTLE: Duration = Duration::from_millis(50);

/// The markers a fenced code block may be written with, per CommonMark.
const MARKERS: [char; 2] = ['`', '~'];

/// The shortest run of a marker that opens a fence.
const SHORTEST_FENCE: usize = 3;

/// The deepest a fence may be indented before it is an indented code block
/// instead of a fenced one.
const DEEPEST_INDENT: usize = 3;

/// An answer under assembly: everything that has arrived, and when the last of
/// it was handed over.
pub(crate) struct Assembly {
    text: String,
    throttle: Duration,
    shown: Option<Instant>,
}

impl Assembly {
    pub(crate) fn new(throttle: Duration) -> Self {
        Self {
            text: String::new(),
            throttle,
            shown: None,
        }
    }

    /// Adds a fragment, and answers with the whole answer so far, render-ready,
    /// when enough time has passed to be worth showing it.
    ///
    /// The whole text rather than the fragment: a window that missed a hand-over
    /// — because it was still loading, or because the throttle skipped one — is
    /// then corrected by the next one rather than left permanently behind.
    pub(crate) fn push(&mut self, fragment: &str) -> Option<String> {
        self.push_at(fragment, Instant::now())
    }

    /// The whole answer, exactly as it arrived.
    ///
    /// Not closed off: a finished answer goes to a renderer that is allowed to
    /// take its time, and one truncated mid-fence is rendered correctly by any
    /// CommonMark implementation without help.
    pub(crate) fn text(self) -> String {
        self.text
    }

    /// [`Assembly::push`] with the clock supplied, so that the throttle can be
    /// tested without waiting for one.
    fn push_at(&mut self, fragment: &str, now: Instant) -> Option<String> {
        self.text.push_str(fragment);

        // The first fragment is shown the moment it lands: the whole promise of
        // streaming is that the first words arrive immediately.
        if self.shown.is_some_and(|shown| now - shown < self.throttle) {
            return None;
        }

        self.shown = Some(now);

        Some(renderable(&self.text))
    }
}

/// Text a Markdown renderer can be given mid-stream: whatever has arrived, with
/// a fence still arriving held back and a fence left open closed off.
pub(crate) fn renderable(text: &str) -> String {
    let text = &text[..settled(text)];

    let Some(fence) = unterminated(text) else {
        return text.to_owned();
    };

    let mut closed = String::with_capacity(text.len() + fence.len() + 1);
    closed.push_str(text);
    if !text.ends_with('\n') {
        closed.push('\n');
    }
    closed.push_str(&fence);

    closed
}

/// Where the text stops being worth showing: before a run of fence markers too
/// short to be a fence yet.
///
/// A fence arrives in fragments like everything else, and a line holding one
/// backtick is a line of prose until the third one lands. Shown as it stands, it
/// is a stray backtick that is then taken away — and where the fence opens a
/// block, the lines after it are drawn as prose and redrawn as code, which is
/// the flicker all of this exists to prevent. Held back for the one fragment it
/// takes to become a fence, or to turn out not to be one.
fn settled(text: &str) -> usize {
    let start = text.rfind('\n').map_or(0, |newline| newline + 1);
    let last = &text[start..];

    let indent = last.len() - last.trim_start_matches(' ').len();
    if indent > DEEPEST_INDENT {
        return text.len();
    }

    let last = &last[indent..];
    let Some(marker) = last.chars().next().filter(|first| MARKERS.contains(first)) else {
        return text.len();
    };

    // Only a line that is nothing but markers: `Call ` is prose that happens to
    // end in one, and the user is entitled to read it.
    let run = last.chars().take_while(|char| *char == marker).count();
    let arriving = run == last.chars().count() && run < SHORTEST_FENCE;

    match arriving {
        true => start,
        false => text.len(),
    }
}

/// The fence that would close the block this text leaves open, if it leaves one.
fn unterminated(text: &str) -> Option<String> {
    let mut open: Option<(char, usize)> = None;

    for line in text.lines() {
        match open {
            None => open = opening(line),
            Some((marker, length)) if closes(line, marker, length) => open = None,
            Some(_) => {}
        }
    }

    open.map(|(marker, length)| String::from(marker).repeat(length))
}

/// The marker and length of the fence this line opens, if it opens one.
fn opening(line: &str) -> Option<(char, usize)> {
    let (marker, run, rest) = fence(line)?;

    // A backtick fence's info string may not itself contain a backtick, which
    // is what keeps `` `a` `` from reading as one.
    let admissible = marker == '~' || !rest.contains('`');

    admissible.then_some((marker, run))
}

/// Whether this line closes a fence opened with `length` of `marker`.
fn closes(line: &str, marker: char, length: usize) -> bool {
    let Some((found, run, rest)) = fence(line) else {
        return false;
    };

    // A closing fence is at least as long as the one it closes and carries
    // nothing else at all.
    found == marker && run >= length && rest.trim().is_empty()
}

/// The marker a line's fence is written with, how long its run is, and what
/// follows it — or `None` when the line is not a fence.
fn fence(line: &str) -> Option<(char, usize, &str)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > DEEPEST_INDENT {
        return None;
    }

    let line = &line[indent..];
    let marker = line
        .chars()
        .next()
        .filter(|first| MARKERS.contains(first))?;
    let run = line.chars().take_while(|char| *char == marker).count();

    (run >= SHORTEST_FENCE).then(|| (marker, run, &line[run..]))
}

#[cfg(test)]
mod tests {
    //! What a fence is, and when the accumulated text is handed over. The
    //! intermediate states of a whole answer are asserted at the facade, which
    //! is where a stream actually exists; these are the cases a stream would
    //! have to be contrived to produce.

    use super::*;

    #[test]
    fn text_with_no_fence_in_it_is_shown_as_it_stands() {
        assert_eq!(renderable("A pipe is not a pipe."), "A pipe is not a pipe.");
    }

    #[test]
    fn a_fence_that_has_been_opened_is_closed() {
        assert_eq!(renderable("```\nfn main() {"), "```\nfn main() {\n```");
    }

    #[test]
    fn a_fence_that_has_been_closed_is_left_alone() {
        assert_eq!(
            renderable("```\nfn main() {}\n```"),
            "```\nfn main() {}\n```"
        );
    }

    #[test]
    fn the_language_a_fence_names_does_not_stop_it_being_one() {
        assert_eq!(renderable("```rust\nfn main"), "```rust\nfn main\n```");
    }

    #[test]
    fn a_fence_opened_at_the_end_of_the_text_is_closed() {
        assert_eq!(renderable("Here:\n\n```rust"), "Here:\n\n```rust\n```");
    }

    #[test]
    fn a_text_ending_in_a_newline_does_not_gain_a_second_one() {
        assert_eq!(renderable("```\ncode\n"), "```\ncode\n```");
    }

    #[test]
    fn a_second_fence_after_a_closed_one_is_closed_in_its_turn() {
        assert_eq!(
            renderable("```\none\n```\n\n```\ntwo"),
            "```\none\n```\n\n```\ntwo\n```"
        );
    }

    #[test]
    fn a_tilde_fence_is_closed_with_tildes() {
        assert_eq!(renderable("~~~\ncode"), "~~~\ncode\n~~~");
    }

    #[test]
    fn a_longer_fence_is_closed_with_one_as_long() {
        assert_eq!(renderable("````\ncode"), "````\ncode\n````");
    }

    #[test]
    fn a_shorter_run_inside_a_longer_fence_does_not_close_it() {
        // Which is the whole reason to write a fence longer than three: the
        // block contains a fence of its own.
        assert_eq!(renderable("````\n```\ncode"), "````\n```\ncode\n````");
    }

    #[test]
    fn a_tilde_fence_is_not_closed_by_backticks() {
        assert_eq!(renderable("~~~\n```\ncode"), "~~~\n```\ncode\n~~~");
    }

    #[test]
    fn a_fence_indented_within_reason_is_still_a_fence() {
        assert_eq!(renderable("   ```\ncode"), "   ```\ncode\n```");
    }

    #[test]
    fn four_spaces_is_an_indented_code_block_and_not_a_fence() {
        assert_eq!(renderable("    ```\ncode"), "    ```\ncode");
    }

    #[test]
    fn a_run_of_two_is_not_a_fence() {
        assert_eq!(renderable("``code"), "``code");
    }

    #[test]
    fn a_closing_fence_may_not_carry_anything_else() {
        assert_eq!(
            renderable("```\ncode\n``` and more"),
            "```\ncode\n``` and more\n```"
        );
    }

    #[test]
    fn an_inline_span_is_not_a_fence_to_close() {
        assert_eq!(
            renderable("Call `main`, then `exit`."),
            "Call `main`, then `exit`."
        );
    }

    #[test]
    fn a_backtick_in_a_backtick_fences_info_string_makes_it_no_fence() {
        assert_eq!(renderable("```a`b"), "```a`b");
    }

    #[test]
    fn a_fence_arriving_a_character_at_a_time_is_held_back_until_it_is_one() {
        assert_eq!(renderable("Like so:\n\n`"), "Like so:\n\n");
        assert_eq!(renderable("Like so:\n\n``"), "Like so:\n\n");
        assert_eq!(renderable("Like so:\n\n```"), "Like so:\n\n```\n```");
    }

    #[test]
    fn a_fence_closing_a_character_at_a_time_is_held_back_the_same_way() {
        assert_eq!(renderable("```\ncode\n``"), "```\ncode\n```");
    }

    #[test]
    fn a_line_that_merely_ends_in_a_marker_is_prose_and_is_shown() {
        assert_eq!(renderable("Call `"), "Call `");
    }

    #[test]
    fn every_state_of_a_streaming_answer_keeps_the_blocks_it_has_drawn() {
        // The anti-flicker property, stated over every prefix a stream could
        // stop at: no state leaves a block open, and no state takes back a
        // block an earlier one drew.
        let answer = "Like so:\n\n```rust\nfn main() {}\n```\n\nor:\n\n~~~\nplain\n~~~\n";
        let mut drawn = 0;

        for end in (0..=answer.len()).filter(|end| answer.is_char_boundary(*end)) {
            let so_far = &answer[..end];
            let fences = fences(&renderable(so_far));

            assert_eq!(fences % 2, 0, "{so_far:?} leaves a block open");
            assert!(fences / 2 >= drawn, "{so_far:?} takes back a block it drew");
            drawn = fences / 2;
        }

        assert_eq!(drawn, 2, "the whole answer draws both of its blocks");
    }

    /// How many fence lines a renderer would find. Twice the number of blocks it
    /// draws, given text whose blocks hold no fences of their own — which is
    /// what the answer above is written to be.
    fn fences(text: &str) -> usize {
        text.lines().filter(|line| fence(line).is_some()).count()
    }

    #[test]
    fn the_first_fragment_is_shown_the_moment_it_arrives() {
        let mut assembly = Assembly::new(THROTTLE);

        assert_eq!(
            assembly.push_at("A pipe", Instant::now()),
            Some("A pipe".to_owned())
        );
    }

    #[test]
    fn a_fragment_arriving_within_the_throttle_is_accumulated_and_not_shown() {
        let start = Instant::now();
        let mut assembly = Assembly::new(THROTTLE);

        assembly.push_at("A pipe", start);

        assert_eq!(assembly.push_at(" is not", start + THROTTLE / 2), None);
    }

    #[test]
    fn the_next_hand_over_carries_everything_the_throttle_held_back() {
        let start = Instant::now();
        let mut assembly = Assembly::new(THROTTLE);

        assembly.push_at("A pipe", start);
        assembly.push_at(" is not", start + THROTTLE / 2);

        assert_eq!(
            assembly.push_at(" a pipe.", start + THROTTLE),
            Some("A pipe is not a pipe.".to_owned())
        );
    }

    #[test]
    fn the_whole_answer_is_what_arrived_and_not_what_was_shown() {
        let start = Instant::now();
        let mut assembly = Assembly::new(THROTTLE);

        assembly.push_at("```\ncode", start);
        assembly.push_at("\nmore", start + THROTTLE / 2);

        assert_eq!(assembly.text(), "```\ncode\nmore");
    }
}
