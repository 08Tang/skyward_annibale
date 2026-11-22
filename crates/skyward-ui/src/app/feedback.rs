//! Reporting a bug: what the page is told to send, and where the sending
//! stands.
//!
//! A report is the words the user writes and the two logs the recovery keeps
//! — its own and the kernel's. The page says what goes, that nothing else
//! does, and that a report written before the bug was seen cannot show it.
//!
//! A report is sent once per start of the recovery: the number it is given is
//! worth nothing if a second report follows it with the same logs, and the
//! page says the number the first one got instead of offering to send again.

use super::*;

/// Where a report stands, as the page shows it.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Report {
    /// Nothing sent: the words may be written, and a report sent.
    #[default]
    Writing,
    /// On its way, with how much of it has gone and how much there is.
    Sending { done: u64, total: u64 },
    /// Taken, and the number it was given.
    Sent { id: String },
    /// Turned down, why, and whether trying again may end otherwise.
    Refused { why: String, again: bool },
}

impl Report {
    /// Whether the report has been taken, so nothing more may be sent this
    /// start: what the page says instead of the form.
    pub fn sent(&self) -> bool {
        matches!(self, Report::Sent { .. })
    }

    /// Whether a report is on its way, so the form takes no touch.
    pub fn sending(&self) -> bool {
        matches!(self, Report::Sending { .. })
    }

    /// How far the report on its way has come, as a share of it.
    pub fn share(&self) -> Option<f32> {
        match self {
            Report::Sending { done, total } if *total > 0 => Some((*done as f32 / *total as f32).clamp(0.0, 1.0)),
            Report::Sending { .. } => Some(0.0),
            _ => None,
        }
    }
}

/// What the feedback page holds: the words written about the bug, and where
/// the report stands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Feedback {
    /// The words written, kept as they are typed so that a report turned
    /// down can be written again without writing them out again.
    pub words: String,
    /// What was last said of the report.
    pub report: Report,
}

/// How many characters the words may be. The address takes more, but a page
/// of them is more than a report needs, and the box on the page is only so
/// tall.
pub const MOST_WORDS: usize = 4000;

impl Feedback {
    /// Add `c` to the words, where there is room for it.
    pub fn push(&mut self, c: char) -> bool {
        if self.words.chars().count() >= MOST_WORDS {
            return false;
        }
        self.words.push(c);
        true
    }

    /// Take the last character back.
    pub fn pop(&mut self) -> bool {
        self.words.pop().is_some()
    }

    /// Whether the words are enough to send: something has to be said of the
    /// bug, or the logs are all there is.
    pub fn ready(&self) -> bool {
        !self.words.trim().is_empty()
    }
}

impl App {
    /// A report is on its way: how much of it has gone, and how much there is.
    pub fn report_at(&mut self, done: u64, total: u64) {
        let was = std::mem::replace(&mut self.state.feedback.report, Report::Sending { done, total });
        // The page is drawn again only where what it shows has changed: the
        // bar moves a whole percent at a time, and drawing for every byte
        // would spend more time on the page than on the report.
        let moved = match (was.share(), self.state.feedback.report.share()) {
            (Some(was), Some(now)) => (was * 100.0) as u32 != (now * 100.0) as u32,
            (was, now) => was != now,
        };
        if moved {
            self.refresh();
        }
    }

    /// The report was taken, and given the number `id`.
    pub fn report_taken(&mut self, id: &str) {
        self.state.feedback.report = Report::Sent { id: id.to_owned() };
        self.refresh();
    }

    /// The report was turned down: why, and whether trying again may end
    /// otherwise.
    pub fn report_refused(&mut self, why: &str, again: bool) {
        self.state.feedback.report = Report::Refused { why: why.to_owned(), again };
        self.refresh();
    }

    /// Write the report out again, keeping the words as they were written.
    pub fn report_again(&mut self) {
        self.state.feedback.report = Report::Writing;
        self.refresh();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_report_sent_is_not_sent_again() {
        let report = Report::Sent { id: "7".into() };
        assert!(report.sent(), "the page says the number, not the form");
        assert!(!report.sending(), "nothing is on its way");
    }

    #[test]
    fn a_report_on_its_way_says_how_far_it_has_come() {
        let report = Report::Sending { done: 50, total: 200 };
        assert_eq!(report.share(), Some(0.25));
        assert!(report.sending());
        assert!(!report.sent(), "one on its way has not been taken");
    }

    #[test]
    fn words_are_kept_until_the_box_is_full() {
        let mut feedback = Feedback::default();
        for _ in 0..MOST_WORDS {
            assert!(feedback.push('a'), "there is room");
        }
        assert!(!feedback.push('a'), "the box is full");
        assert_eq!(feedback.words.chars().count(), MOST_WORDS);
        assert!(feedback.pop(), "the last one comes back");
        assert_eq!(feedback.words.chars().count(), MOST_WORDS - 1);
    }

    #[test]
    fn nothing_written_is_not_a_report() {
        let mut feedback = Feedback::default();
        assert!(!feedback.ready(), "nothing said of the bug");
        feedback.words = "   \n ".into();
        assert!(!feedback.ready(), "space is not words");
        feedback.words = "it will not start".into();
        assert!(feedback.ready());
    }
}
