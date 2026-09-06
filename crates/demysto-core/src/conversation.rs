//! The Conversation: one Run of an Action plus the follow-up Turns the user
//! takes on the same Selection, and this session's store of them.
//!
//! In memory and nowhere else, per the spec's *Conversation store*: what the
//! user looked at today is not sitting on disk next month (user story 62).
//! Quitting therefore loses every Conversation, and asking whether it should is
//! asking about something the user was never promised.

use std::collections::VecDeque;
use std::sync::Arc;

use crate::action::Action;
use crate::picture::Png;
use crate::run::RunOutcome;
use crate::selection::{Kind, Selection};

/// How many Conversations a session holds before the oldest falls off.
pub(crate) const CAP: usize = 50;

/// How much of this session's pictures are held at once before the oldest are
/// let go of, in bytes.
///
/// A picture is heavy in a way text is not, and a resident tool left running
/// all week must not grow without bound (user story 85). In any ordinary
/// session this never fires: it is the ceiling under the rule that a picture
/// lives as long as the window that shows it, not the rule itself.
pub(crate) const HELD_PICTURES: u64 = 128 * 1024 * 1024;

/// How much of the Selection the list of Conversations shows, in characters.
/// Enough to tell two Runs of the same Action apart, and no more.
const ABOUT: usize = 80;

/// How much of the Selection travels to the result window unasked, in
/// characters. Comfortably more than the two lines that window quotes, so that
/// anything short is already whole by the time somebody asks to see the rest —
/// and bounded, because a Selection is as often a chapter as a phrase and this
/// crosses the bridge every time the window refreshes.
pub(crate) const PREVIEW: usize = 400;

/// What the Provider is told each part of a Conversation is.
const USER: &str = "user";
const ASSISTANT: &str = "assistant";

/// What a continuation asks for, after the answer so far has been put back to
/// the Model as its own words.
///
/// A message rather than a bare assistant turn left hanging: the contract has
/// no prefill in it, and a Provider handed a Conversation ending mid-sentence
/// is entitled to start a new paragraph. Asking in words is the one thing every
/// service implementing this contract understands the same way.
const CARRY_ON: &str = "That answer was cut off before it finished. Carry on from exactly where \
                        it stops, and do not repeat any of it.";

/// What one message of a Conversation says, as the contract carries it.
///
/// Words alone for everything v1 sent, so that a text Run's request is byte for
/// byte the one it always was. The other variant is the message a picture rides
/// in — which is the first user message of an image Conversation, and no other:
/// the whole list is resent on every Turn, so the picture is in front of the
/// Model for every question asked about it.
pub(crate) enum Said {
    Words(String),
    WordsAndPicture { text: String, picture: Arc<Png> },
}

/// Why the Turn now being asked has nothing to be asked with.
pub(crate) enum Missing {
    /// There is no Conversation to ask in: none on screen, or one evicted out
    /// from under its own Run.
    Conversation,
    /// There is nothing in it to ask again: a Conversation whose last Turn is
    /// still being answered, or one with no Turn to act on.
    Turn,
    /// Its picture has been let go of. The Conversation is Sealed: it reads
    /// exactly as it did, and there is nothing left to resend.
    Picture,
}

/// One Run of an Action plus the follow-up Turns taken on the same Selection.
///
/// The unit the result window shows and the unit this session's history is
/// counted in.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Conversation {
    /// What this Conversation is asked for by, for as long as the session
    /// lasts. Nothing outlives the session, so nothing needs an identifier
    /// that would.
    pub id: u64,
    /// The Action the opening Run ran, `None` when it was not one Demysto has.
    pub action: Option<Action>,
    /// Every Turn taken so far, oldest first.
    pub turns: Vec<Turn>,
    /// The Model the user switched this Conversation to, `None` while it is
    /// still going to whatever the Action resolves to. Set from the window when
    /// a failed Turn is tried again somewhere else (user story 20), and it
    /// stands for every Turn after it: the switch is a decision about this
    /// Conversation, not about one Turn of it.
    pub model: Option<String>,
    /// What the user was told about the Selection before anything was sent —
    /// that it is unusually large, today. `None` when there was nothing to say.
    ///
    /// On the Conversation rather than on a Turn because it is about the
    /// Selection, which every Turn shares: a follow-up does not send it again
    /// and is not warned about it again.
    pub warning: Option<String>,
    /// The opening of the Selection, for the window to quote above the answer:
    /// what the Model is being asked about, in the user's own words rather than
    /// in the Action's name alone. `None` where there was no Selection.
    ///
    /// Held beside the Selection rather than taken from it at serialisation
    /// time, for the reason `warning` is: both are settled when the Conversation
    /// opens and neither changes after.
    pub preview: Option<String>,
    /// How the picture this Conversation is about now stands, `None` for one
    /// about words. Outlives the picture itself, which is what lets a Sealed
    /// Conversation still say how large it was.
    pub picture: Option<PictureStanding>,
    /// What every Turn in it is about. Held because the list shows it, because
    /// the window asks for the whole of it when the preview is expanded, and
    /// because a Run declared before it happens has already been told it; what
    /// the Model is sent is the Turns, not this.
    ///
    /// `None` for a Conversation whose picture has been let go of — see
    /// [`Conversation::release`].
    #[serde(skip)]
    selection: Option<Selection>,
}

/// How the picture a Conversation is about now stands, which is what the window
/// needs and the picture itself is not.
///
/// The picture crosses the bridge once, when the window asks for it; this
/// crosses every time a Turn begins or ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct PictureStanding {
    /// What asking again at the original resolution would send, so that the
    /// price is written on the button rather than in a warning over it (user
    /// story 78).
    pub original_bytes: u64,
    /// Whether fitting took anything off it. Where it did not, the original is
    /// the picture already being sent, and there is nothing to offer.
    pub fitted: bool,
    /// Whether every Turn from here on sends the original rather than the
    /// fitted picture, because somebody asked again at the original resolution.
    ///
    /// Held here and not on a Turn for the reason the switched Model is: a
    /// person who asked for resolution asked because of a detail, and the
    /// question after that one is about the same detail (user story 79).
    pub original: bool,
    /// Whether the picture has been let go of, which is what makes a
    /// Conversation Sealed: readable, but not continuable (user story 84).
    pub sealed: bool,
}

/// A single user message and the Model's reply.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Turn {
    /// What the user asked, in their own words. `None` for the Turn that opened
    /// the Conversation, which the Action asked on their behalf — the window
    /// heads that one with the Action's name instead.
    pub question: Option<String>,
    /// What the Turn produced, `None` while the Model is still answering.
    pub outcome: Option<RunOutcome>,
    /// What was actually sent for it. The prompt an Action assembles around a
    /// Selection is far longer than anything worth putting on screen, and it is
    /// the next Turn's context rather than the window's business.
    ///
    /// Recorded before the Turn goes anywhere, so that a Turn that failed can
    /// be asked again without the Palette that composed it.
    #[serde(skip)]
    prompt: String,
    /// What arrived for this Turn before it was interrupted, while it is being
    /// continued. Empty otherwise — including after the continuation lands,
    /// which puts the whole of it in the outcome.
    #[serde(skip)]
    delivered: String,
}

/// One line of the list of this session's Conversations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Summary {
    pub id: u64,
    /// What the list calls it: the Action that opened it, where it was one
    /// Demysto has.
    pub name: Option<String>,
    /// The opening words of what it is about, so that two Runs of one Action
    /// are not two identical lines.
    pub about: String,
}

/// Everything the Turn now being asked needs in order to be put to a Provider,
/// taken out of the store rather than composed by the caller.
///
/// A retry and a continuation both ask a Turn that already exists, so neither
/// has an Action or a Palette to take any of this from.
pub(crate) struct Asked {
    pub(crate) id: u64,
    pub(crate) binding: Option<String>,
    pub(crate) kind: Kind,
    pub(crate) prompt: String,
    /// What already arrived for this Turn, for a continuation; empty otherwise.
    pub(crate) delivered: String,
}

/// This session's Conversations, and which of them the window is showing.
pub(crate) struct Store {
    /// Newest first, which is the order the list is read in and the end
    /// eviction does not touch.
    held: VecDeque<Conversation>,
    /// The Conversation the result window is showing: the one a Run just
    /// opened, until the user goes back to an earlier one.
    showing: Option<u64>,
    /// How many have been opened this session, which is where the next
    /// identifier comes from.
    opened: u64,
    /// How much may be held in pictures before the oldest are let go of.
    ///
    /// A field rather than the constant itself, for the reason a Run's timeout
    /// is one: a ceiling nobody can reach in a test is a ceiling nobody has
    /// tested, and reaching this one honestly would mean a suite that encodes
    /// 128 MB of noise.
    ceiling: u64,
}

impl Store {
    /// A session with nothing asked in it yet.
    pub(crate) fn new() -> Self {
        Self {
            held: VecDeque::new(),
            showing: None,
            opened: 0,
            ceiling: HELD_PICTURES,
        }
    }

    /// The same store with a ceiling a test can reach.
    #[cfg(test)]
    pub(crate) fn holding_at_most(&mut self, bytes: u64) {
        self.ceiling = bytes;
    }

    /// Opens the Conversation the Run about to begin will fill, puts it on
    /// screen, and answers with what to fill it by.
    ///
    /// The interface declares a Run before it shows the window, and the Run
    /// declares it again for a caller that did not: the second finds the
    /// Conversation the first opened rather than leaving an empty one behind.
    pub(crate) fn open(
        &mut self,
        action: Option<Action>,
        selection: Option<Selection>,
        warning: Option<String>,
    ) -> u64 {
        if !self.held.front().is_some_and(Conversation::unanswered) {
            self.opened += 1;
            self.held.push_front(Conversation {
                id: self.opened,
                action: None,
                turns: vec![Turn::opening()],
                model: None,
                warning: None,
                preview: None,
                picture: None,
                selection: None,
            });

            // The oldest is the back, and the back is what goes: what the user
            // is still thinking about is what they most recently asked.
            self.held.truncate(CAP);
        }

        let opening = self
            .held
            .front_mut()
            .expect("a Conversation is open by this point");

        opening.action = action;
        // The dimensions where there is a picture and the opening words where
        // there is text: what the window quotes above the first Turn, and what
        // the list of Conversations shows, are the same fact said in the shape
        // the Selection has.
        opening.preview = selection.as_ref().map(|selection| {
            selection
                .dimensions()
                .unwrap_or_else(|| opening_of(selection.as_text(), PREVIEW))
        });
        opening.picture = selection
            .as_ref()
            .and_then(Selection::as_picture)
            .map(|picture| PictureStanding {
                original_bytes: picture.original_weight(),
                fitted: picture.was_fitted(),
                original: false,
                sealed: false,
            });
        opening.selection = selection;
        opening.warning = warning;
        self.showing = Some(opening.id);

        let id = opening.id;

        self.within_the_ceiling();

        id
    }

    /// Lets go of every picture this session is holding, which is what closing
    /// the result window asks for.
    ///
    /// A picture lives as long as the window that shows it — ADR-0017. Nothing
    /// is discarded by this: every Conversation stays in the list and reads
    /// exactly as it did, and what is gone is the ability to add to one.
    pub(crate) fn release_pictures(&mut self) {
        for conversation in &mut self.held {
            conversation.release();
        }
    }

    /// Lets the oldest pictures go until what is held is under the ceiling.
    ///
    /// The Conversation just opened is never one of them: sealing a Conversation
    /// before its first Turn would be answering a question by refusing it.
    fn within_the_ceiling(&mut self) {
        let mut held: u64 = self
            .held
            .iter()
            .filter_map(Conversation::picture_weight)
            .sum();

        for at in (1..self.held.len()).rev() {
            if held <= self.ceiling {
                return;
            }

            if let Some(released) = self.held[at].release() {
                held -= released;
            }
        }
    }

    /// Adds the Turn a follow-up asks to the Conversation on screen, so that
    /// the window can show the question before there is an answer to it, and
    /// answers with the Conversation it was added to.
    ///
    /// `None` when there is no Conversation to add it to. Declared twice for
    /// the reason [`Self::open`] is, and added once for the same reason.
    pub(crate) fn follow_up(&mut self, question: &str) -> Result<&Conversation, Missing> {
        let showing = self.showing_mut().ok_or(Missing::Conversation)?;

        // A Sealed Conversation is not one to add to: there is nothing left to
        // resend, and a Turn recorded here would be a question nobody could
        // ask. The window says so where the input box was, and this is what
        // keeps a window that did not from putting one on screen.
        if showing.sealed() {
            return Err(Missing::Picture);
        }

        if !showing.turns.last().is_some_and(|last| last.asks(question)) {
            showing.turns.push(Turn::asking(question));
        }

        Ok(showing)
    }

    /// Records what the Turn now being asked sends, and answers with everything
    /// said so far to send it among.
    ///
    /// `None` when the Conversation is no longer held, which is a Conversation
    /// evicted out from under its own Run.
    pub(crate) fn asking(
        &mut self,
        id: u64,
        prompt: String,
    ) -> Result<Vec<(&'static str, Said)>, Missing> {
        let conversation = self.held_mut(id).ok_or(Missing::Conversation)?;

        if conversation.sealed() {
            return Err(Missing::Picture);
        }

        let turn = conversation.turns.last_mut().ok_or(Missing::Conversation)?;
        turn.prompt = prompt;

        Ok(conversation.said())
    }

    /// Records what the Turn now being asked produced.
    ///
    /// What a continuation was carrying is let go of here: the outcome holds
    /// the whole answer, the part that arrived first included, and two copies
    /// of it would be two places for it to differ.
    pub(crate) fn answered(&mut self, id: u64, outcome: RunOutcome) {
        if let Some(turn) = self.held_mut(id).and_then(|held| held.turns.last_mut()) {
            turn.outcome = Some(outcome);
            turn.delivered = String::new();
        }
    }

    /// Puts the last Turn of the Conversation on screen back to being asked, so
    /// that it can be sent a second time — the retry a failed Turn is offered
    /// (user story 44), and the Model switch that goes with it (user story 20).
    ///
    /// `model` is the Model the user picked, `None` for trying again with
    /// nothing changed. A Model picked here stands for the rest of the
    /// Conversation, not for this Turn alone.
    ///
    /// `None` where there is nothing to try again: no Conversation on screen, or
    /// one whose last Turn is still being answered.
    pub(crate) fn retrying(&mut self, model: Option<&str>) -> Result<Asked, Missing> {
        let showing = self.showing_mut().ok_or(Missing::Conversation)?;

        // Refused before anything is cleared, for the reason the Model is
        // switched after rather than before: a Turn put back to being asked and
        // then refused is a Turn whose answer was thrown away for nothing.
        if showing.sealed() {
            return Err(Missing::Picture);
        }

        // Asked before the Model is switched, so that a retry with nothing to
        // retry changes nothing: switching the Conversation to a Model and then
        // not asking it anything would leave the window saying one thing and
        // the next Turn doing another.
        let turn = showing.turns.last_mut().ok_or(Missing::Turn)?;
        turn.outcome.as_ref().ok_or(Missing::Turn)?;

        turn.outcome = None;
        turn.delivered = String::new();

        if let Some(model) = model {
            showing.model = Some(model.to_owned());
        }

        Ok(showing.asked())
    }

    /// Puts the last Turn of the Conversation on screen back to being asked, at
    /// the original resolution of the picture it is about — and leaves it there
    /// for every Turn after this one (user stories 77 and 79).
    ///
    /// A retry with one thing changed, the way the Model switch is one: asking
    /// for resolution without asking anything again would leave the user
    /// looking at the same answer that missed the detail.
    ///
    /// The Turn asked again is the last one, where the spec says "re-runs the
    /// Action". The two are the same thing in the case the button exists for —
    /// a disappointing first answer — and differ only after a follow-up, where
    /// re-running the Action would either discard the follow-ups or ask the
    /// opening question a second time underneath them. What story 77 asks for
    /// is the answer that missed a detail, asked again; that is the last Turn.
    ///
    /// `None` where there is nothing to ask again, and where the Conversation is
    /// not about a picture at all.
    pub(crate) fn at_original_resolution(&mut self) -> Result<Asked, Missing> {
        let showing = self.showing_mut().ok_or(Missing::Conversation)?;

        if showing.sealed() {
            return Err(Missing::Picture);
        }

        showing
            .selection
            .as_ref()
            .and_then(Selection::as_picture)
            .ok_or(Missing::Turn)?;

        let turn = showing.turns.last_mut().ok_or(Missing::Turn)?;
        turn.outcome.as_ref().ok_or(Missing::Turn)?;

        turn.outcome = None;
        turn.delivered = String::new();

        showing
            .picture
            .as_mut()
            .expect("a Conversation with a picture stands for one")
            .original = true;

        Ok(showing.asked())
    }

    /// Puts the last Turn of the Conversation on screen back to being asked,
    /// keeping what already arrived for it, so that the Model can be asked for
    /// the rest (user story 46).
    ///
    /// `None` where the last Turn is not one that broke off part-way — there is
    /// nothing to continue from anything that finished, and nothing to continue
    /// at all where nothing arrived.
    pub(crate) fn continuing(&mut self) -> Result<Asked, Missing> {
        let showing = self.showing_mut().ok_or(Missing::Conversation)?;

        // Refused before anything is put back to being asked, for the reason a
        // retry is: what already arrived is the user's, and losing it to a
        // request that was never going to be made would be the worse failure.
        if showing.sealed() {
            return Err(Missing::Picture);
        }

        let turn = showing.turns.last_mut().ok_or(Missing::Turn)?;

        let RunOutcome::Interrupted { text, .. } = turn.outcome.as_ref().ok_or(Missing::Turn)?
        else {
            return Err(Missing::Turn);
        };

        turn.delivered = text.clone();
        turn.outcome = None;

        Ok(showing.asked())
    }

    /// The Conversation the result window is showing.
    pub(crate) fn showing(&self) -> Option<&Conversation> {
        self.held(self.showing?)
    }

    /// Puts an earlier Conversation on screen, which is what a window closed
    /// and a list gone back to are for.
    pub(crate) fn show(&mut self, id: u64) -> Option<&Conversation> {
        self.held(id)?;
        self.showing = Some(id);

        self.held(id)
    }

    /// This session's Conversations, newest first.
    pub(crate) fn summaries(&self) -> Vec<Summary> {
        self.held.iter().map(Conversation::summary).collect()
    }

    /// The Conversation the window is showing, to be added to.
    fn showing_mut(&mut self) -> Option<&mut Conversation> {
        self.held_mut(self.showing?)
    }

    /// The Conversation asked for by `id`, or `None` when the session no longer
    /// holds one by that name.
    fn held(&self, id: u64) -> Option<&Conversation> {
        self.held.iter().find(|held| held.id == id)
    }

    fn held_mut(&mut self, id: u64) -> Option<&mut Conversation> {
        self.held.iter_mut().find(|held| held.id == id)
    }
}

impl Conversation {
    /// The Model every Turn in this Conversation goes to: the one the user
    /// switched it to, else the one the Action that opened it bound. `None`
    /// when neither named one, and the two defaults decide.
    pub(crate) fn binding(&self) -> Option<&str> {
        self.model.as_deref().or_else(|| {
            self.action
                .as_ref()
                .and_then(|action| action.model.as_deref())
        })
    }

    /// Everything the Turn now being asked needs in order to be asked, taken
    /// off the Conversation it belongs to.
    fn asked(&self) -> Asked {
        let turn = self
            .turns
            .last()
            .expect("a Conversation asked in holds a Turn");

        Asked {
            id: self.id,
            binding: self.binding().map(ToOwned::to_owned),
            kind: self.kind(),
            prompt: turn.prompt.clone(),
            delivered: turn.delivered.clone(),
        }
    }

    /// What every Turn in this Conversation is about, which is what decides
    /// whether it needs a Model that can see.
    ///
    /// Text where there is no Selection at all: a Run without one fails before
    /// it opens a Conversation, so the only way here is the empty Conversation
    /// a declared Run leaves behind, and asking about nothing is asking in
    /// words. A Sealed Conversation is still about the picture it was about,
    /// whether or not Demysto still holds it.
    pub(crate) fn kind(&self) -> Kind {
        match (&self.selection, &self.picture) {
            (Some(selection), _) => selection.kind(),
            (None, Some(_)) => Kind::Image,
            (None, None) => Kind::Text,
        }
    }

    /// The whole of what every Turn in this Conversation is about, for the
    /// window that has quoted the opening of it and been asked for the rest.
    ///
    /// `None` for a picture: what that window is owed is the picture, which is
    /// [`Self::picture_url`]'s.
    pub(crate) fn selection_text(&self) -> Option<&str> {
        self.selection
            .as_ref()
            .map(Selection::as_text)
            .filter(|text| !text.is_empty())
    }

    /// The picture this Conversation is about, as the window shows one. `None`
    /// where it is about words, and where the picture has been let go of.
    pub(crate) fn picture_url(&self) -> Option<String> {
        self.selection.as_ref().and_then(Selection::picture_url)
    }

    /// What holding this Conversation's picture costs, `None` where it holds
    /// none.
    fn picture_weight(&self) -> Option<u64> {
        self.selection
            .as_ref()
            .and_then(Selection::as_picture)
            .map(crate::Picture::weight_held)
    }

    /// Lets go of the picture, and answers with what that gave back.
    ///
    /// The Conversation itself stays: the exchange reads exactly as it did, and
    /// what is gone is the ability to add to it. Only a picture is ever let go
    /// of — a text Selection keeps until the session ends, as in v1.
    fn release(&mut self) -> Option<u64> {
        let weight = self.picture_weight()?;

        self.selection = None;
        if let Some(picture) = self.picture.as_mut() {
            picture.sealed = true;
        }

        Some(weight)
    }

    /// Whether this is a Conversation that can be read but not added to.
    fn sealed(&self) -> bool {
        self.picture.is_some_and(|picture| picture.sealed)
    }

    /// Whether every Turn from here on sends the original picture.
    fn at_original_resolution(&self) -> bool {
        self.picture.is_some_and(|picture| picture.original)
    }

    /// What the list of Conversations calls this one: the dimensions where it
    /// is about a picture, and the opening words where it is about text.
    ///
    /// Read from the Selection rather than from the preview, so that the words
    /// are collapsed over the whole of it as they always were. A Sealed
    /// Conversation no longer holds one, and its preview is the dimensions —
    /// settled when it opened, and unchanged since.
    fn about_line(&self) -> String {
        match self.selection.as_ref() {
            Some(selection) => selection
                .dimensions()
                .unwrap_or_else(|| about(selection.as_text())),
            None => self.preview.clone().unwrap_or_default(),
        }
    }

    /// Whether this is a Conversation whose one Turn is still waiting for its
    /// first answer, and so is the one a Run about to begin belongs to.
    fn unanswered(&self) -> bool {
        matches!(self.turns.as_slice(), [only] if only.outcome.is_none())
    }

    /// Everything said in this Conversation so far, in the order it was said,
    /// as the Provider is told it — ending on the Turn being asked now.
    ///
    /// Every question travels, answered or not. The Turn that failed is where
    /// that matters: the Turn which opened the Conversation is the one carrying
    /// the Selection, and dropping it because the Provider refused it once
    /// would leave the Turn after it asking about nothing at all. What does not
    /// travel is a reply that was never given.
    ///
    /// A question with nothing in it is not one: the opening Turn of a Run that
    /// failed before it assembled a prompt has none, and an empty message is
    /// not something to put to a Provider.
    fn said(&self) -> Vec<(&'static str, Said)> {
        let mut said: Vec<(&'static str, Said)> = self
            .turns
            .iter()
            .flat_map(|turn| {
                let asked =
                    (!turn.prompt.is_empty()).then(|| (USER, Said::Words(turn.prompt.clone())));
                let replied = turn
                    .replied()
                    .map(|reply| (ASSISTANT, Said::Words(reply.to_owned())));

                asked.into_iter().chain(replied)
            })
            .collect();

        // A Turn being continued ends the Conversation on what the Model got
        // through before the stream broke, which is not a question — so it is
        // followed by the one that asks for the rest.
        if self
            .turns
            .last()
            .is_some_and(|turn| turn.outcome.is_none() && !turn.delivered.is_empty())
        {
            said.push((USER, Said::Words(CARRY_ON.to_owned())));
        }

        self.carrying_the_picture(&mut said);

        said
    }

    /// Puts the picture in the first user message, where there is one to put.
    ///
    /// The first and no other. Because the whole list is resent on every Turn
    /// and the contract holds no state, one copy there is one copy in front of
    /// the Model for every question in the Conversation — and the alternative
    /// is a second question answered from the Model's memory of its own first
    /// answer.
    fn carrying_the_picture(&self, said: &mut [(&'static str, Said)]) {
        let Some(picture) = self.selection.as_ref().and_then(Selection::as_picture) else {
            return;
        };

        let Some((_, first)) = said.iter_mut().find(|(role, _)| *role == USER) else {
            return;
        };

        let Said::Words(text) = first else {
            return;
        };

        *first = Said::WordsAndPicture {
            text: std::mem::take(text),
            picture: match self.at_original_resolution() {
                true => Arc::clone(picture.original()),
                false => Arc::clone(picture.fitted()),
            },
        };
    }

    /// This Conversation as one line of the list of them.
    fn summary(&self) -> Summary {
        Summary {
            id: self.id,
            name: self.action.as_ref().map(|action| action.name.clone()),
            about: self.about_line(),
        }
    }
}

impl Turn {
    /// The Turn that opens a Conversation, asked by the Action rather than in
    /// the user's own words. Its prompt arrives when the Run assembles one.
    fn opening() -> Self {
        Self {
            question: None,
            outcome: None,
            prompt: String::new(),
            delivered: String::new(),
        }
    }

    /// A follow-up Turn, whose question is the whole of what is sent: the
    /// context is the Turns before it.
    fn asking(question: &str) -> Self {
        Self {
            question: Some(question.to_owned()),
            outcome: None,
            prompt: question.to_owned(),
            delivered: String::new(),
        }
    }

    /// Whether this Turn is that question, still waiting for its answer.
    fn asks(&self, question: &str) -> bool {
        self.outcome.is_none() && self.question.as_deref() == Some(question)
    }

    /// What the Model said, where it said anything: an answer, as much of one
    /// as had arrived when the user stopped it or when the stream broke, or —
    /// for a Turn now being continued — as much as arrived before it did. A
    /// failure is not something the Model said.
    fn replied(&self) -> Option<&str> {
        let said = match self.outcome.as_ref() {
            Some(outcome) => outcome.text().unwrap_or_default(),
            None => self.delivered.as_str(),
        };

        (!said.is_empty()).then_some(said)
    }
}

/// The opening words of a Selection, on one line: the list is a list, and a
/// Selection is as often a page as a phrase.
fn about(selection: &str) -> String {
    opening_of(
        &selection.split_whitespace().collect::<Vec<_>>().join(" "),
        ABOUT,
    )
}

/// The first `at` characters of some text, saying so where there were more.
///
/// Counted in characters and cut on their boundaries: a Selection is as likely
/// to be Japanese as English, and `String::truncate` on a byte offset in the
/// middle of one panics.
///
/// Newlines are left where they are — unlike [`about`], whose line has no room
/// for them. The window quotes this as it was written.
fn opening_of(text: &str, at: usize) -> String {
    let Some((end, _)) = text.char_indices().nth(at) else {
        return text.to_owned();
    };

    let mut opening = text[..end].to_owned();
    opening.push('…');

    opening
}
