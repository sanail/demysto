//! The Conversation: one Run of an Action plus the follow-up Turns the user
//! takes on the same Selection, and this session's store of them.
//!
//! In memory and nowhere else, per the spec's *Conversation store*: what the
//! user looked at today is not sitting on disk next month (user story 62).
//! Quitting therefore loses every Conversation, and asking whether it should is
//! asking about something the user was never promised.

use std::collections::VecDeque;

use crate::action::Action;
use crate::run::RunOutcome;
use crate::selection::{Kind, Selection};

/// How many Conversations a session holds before the oldest falls off.
pub(crate) const CAP: usize = 50;

/// How much of the Selection the list of Conversations shows, in characters.
/// Enough to tell two Runs of the same Action apart, and no more.
const ABOUT: usize = 80;

/// What the Provider is told each part of a Conversation is.
const USER: &str = "user";
const ASSISTANT: &str = "assistant";

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
    /// What every Turn in it is about. Held because the list shows it and
    /// because a Run declared before it happens has already been told it; what
    /// the Model is sent is the Turns, not this.
    #[serde(skip)]
    selection: Option<Selection>,
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
    #[serde(skip)]
    prompt: String,
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
}

impl Store {
    /// A session with nothing asked in it yet.
    pub(crate) fn new() -> Self {
        Self {
            held: VecDeque::new(),
            showing: None,
            opened: 0,
        }
    }

    /// Opens the Conversation the Run about to begin will fill, puts it on
    /// screen, and answers with what to fill it by.
    ///
    /// The interface declares a Run before it shows the window, and the Run
    /// declares it again for a caller that did not: the second finds the
    /// Conversation the first opened rather than leaving an empty one behind.
    pub(crate) fn open(&mut self, action: Option<Action>, selection: Option<Selection>) -> u64 {
        if !self.held.front().is_some_and(Conversation::unanswered) {
            self.opened += 1;
            self.held.push_front(Conversation {
                id: self.opened,
                action: None,
                turns: vec![Turn::opening()],
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
        opening.selection = selection;
        self.showing = Some(opening.id);

        opening.id
    }

    /// Adds the Turn a follow-up asks to the Conversation on screen, so that
    /// the window can show the question before there is an answer to it, and
    /// answers with the Conversation it was added to.
    ///
    /// `None` when there is no Conversation to add it to. Declared twice for
    /// the reason [`Self::open`] is, and added once for the same reason.
    pub(crate) fn follow_up(&mut self, question: &str) -> Option<&Conversation> {
        let showing = self.showing_mut()?;

        if !showing.turns.last().is_some_and(|last| last.asks(question)) {
            showing.turns.push(Turn::asking(question));
        }

        Some(showing)
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
    ) -> Option<Vec<(&'static str, String)>> {
        let conversation = self.held_mut(id)?;
        conversation.turns.last_mut()?.prompt = prompt;

        Some(conversation.said())
    }

    /// Records what the Turn now being asked produced.
    pub(crate) fn answered(&mut self, id: u64, outcome: RunOutcome) {
        if let Some(turn) = self.held_mut(id).and_then(|held| held.turns.last_mut()) {
            turn.outcome = Some(outcome);
        }
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
    /// The Model the Action that opened this Conversation bound, which every
    /// Turn in it goes to. `None` when it bound none, and the two defaults
    /// decide.
    pub(crate) fn binding(&self) -> Option<&str> {
        self.action
            .as_ref()
            .and_then(|action| action.model.as_deref())
    }

    /// What every Turn in this Conversation is about, which is what decides
    /// whether it needs a Model that can see.
    ///
    /// Text where there is no Selection at all: a Run without one fails before
    /// it opens a Conversation, so the only way here is the empty Conversation
    /// a declared Run leaves behind, and asking about nothing is asking in
    /// words.
    pub(crate) fn kind(&self) -> Kind {
        self.selection.as_ref().map_or(Kind::Text, Selection::kind)
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
    fn said(&self) -> Vec<(&'static str, String)> {
        self.turns
            .iter()
            .flat_map(|turn| {
                let asked = (!turn.prompt.is_empty()).then(|| (USER, turn.prompt.clone()));
                let replied = turn.replied().map(|reply| (ASSISTANT, reply.to_owned()));

                asked.into_iter().chain(replied)
            })
            .collect()
    }

    /// This Conversation as one line of the list of them.
    fn summary(&self) -> Summary {
        Summary {
            id: self.id,
            name: self.action.as_ref().map(|action| action.name.clone()),
            about: self
                .selection
                .as_ref()
                .map(|selection| about(selection.as_text()))
                .unwrap_or_default(),
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
        }
    }

    /// A follow-up Turn, whose question is the whole of what is sent: the
    /// context is the Turns before it.
    fn asking(question: &str) -> Self {
        Self {
            question: Some(question.to_owned()),
            outcome: None,
            prompt: question.to_owned(),
        }
    }

    /// Whether this Turn is that question, still waiting for its answer.
    fn asks(&self, question: &str) -> bool {
        self.outcome.is_none() && self.question.as_deref() == Some(question)
    }

    /// What the Model said, where it said anything: an answer, or as much of
    /// one as had arrived when the user stopped it. A failure is not something
    /// the Model said.
    fn replied(&self) -> Option<&str> {
        let said = match self.outcome.as_ref()? {
            RunOutcome::Answered(text) | RunOutcome::Stopped(text) => text,
            RunOutcome::Failed(_) => return None,
        };

        (!said.is_empty()).then_some(said.as_str())
    }
}

/// The opening words of a Selection, on one line: the list is a list, and a
/// Selection is as often a page as a phrase.
fn about(selection: &str) -> String {
    let mut about: String = selection.split_whitespace().collect::<Vec<_>>().join(" ");

    if let Some((end, _)) = about.char_indices().nth(ABOUT) {
        about.truncate(end);
        about.push('…');
    }

    about
}
