//! The phone's remote processors, as `remoteproc` holds them, and the one
//! thing the recovery does to them: ask one again that the boot left down.
//!
//! A processor is asked for its firmware from several threads at once as the
//! drivers load. One segment answered `ENOENT` stops the whole start —
//! `qcom_q6v5_pas` logs `can't start rproc 3000000.remoteproc-adsp: -2` — and
//! the loader never asks again, so the processor stays down for the rest of
//! the boot. Writing `start` to its state is what asks for it a second time,
//! which is worth doing once whatever the firmware is read through is ready.

use std::fs;
use std::path::Path;

/// Where the kernel keeps the processors.
const REMOTEPROC: &str = "/sys/class/remoteproc";

/// What a processor's state reads once it is up.
const RUNNING: &str = "running";

/// What asking a processor again came to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Raised {
    /// No such processor here: nothing to raise, and nothing wrong.
    Absent,
    /// It was already running, and was left alone.
    Already,
    /// The start was queued: the processor is up once its state says so.
    Asked,
    /// The asking was refused.
    Refused,
}

impl Raised {
    /// What this came to, for the log.
    pub fn said(self) -> &'static str {
        match self {
            Raised::Absent => "this phone has no processor by that name",
            Raised::Already => "it was already running",
            Raised::Asked => "it was asked to start again",
            Raised::Refused => "asking it again was refused",
        }
    }
}

/// Ask the processor whose name contains `wanted` to start, and return at once
/// without waiting for it: the write only queues the start, and the processor
/// is up once its own state says so. Whoever needs it reads that state.
///
/// The name is what finds it: the number `remoteproc` gives a processor is not
/// fixed, so `adsp` is looked for rather than `remoteproc1`. A processor that
/// is already running is left alone.
pub fn ask(wanted: &str) -> Raised {
    match state_of(wanted) {
        Some(state) => ask_at(&state),
        None => Raised::Absent,
    }
}

/// Ask the processor behind `state` to start, where it is not running already.
fn ask_at(state: &Path) -> Raised {
    let Ok(now) = fs::read_to_string(state) else {
        return Raised::Absent;
    };
    if now.trim() == RUNNING {
        return Raised::Already;
    }
    match fs::write(state, "start") {
        Ok(()) => Raised::Asked,
        Err(_) => Raised::Refused,
    }
}

/// The state node of the processor whose name contains `wanted`. An entry that
/// cannot be read is passed over rather than ending the search, since the
/// processors beside the one wanted are not it.
fn state_of(wanted: &str) -> Option<std::path::PathBuf> {
    for entry in fs::read_dir(REMOTEPROC).ok()? {
        let Ok(dir) = entry.map(|e| e.path()) else {
            continue;
        };
        let Ok(name) = fs::read_to_string(dir.join("name")) else {
            continue;
        };
        if name.contains(wanted) {
            return Some(dir.join("state"));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_processor_that_is_already_running_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("state");
        fs::write(&state, "running\n").unwrap();
        assert_eq!(ask_at(&state), Raised::Already);
        assert_eq!(fs::read_to_string(&state).unwrap(), "running\n", "the node was not written to");
    }

    #[test]
    fn a_processor_left_down_is_asked_to_start() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join("state");
        fs::write(&state, "offline\n").unwrap();
        assert_eq!(ask_at(&state), Raised::Asked);
        assert_eq!(fs::read_to_string(&state).unwrap(), "start", "the node was told to start");
    }

    #[test]
    fn a_processor_whose_state_cannot_be_read_is_not_there() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(ask_at(&dir.path().join("absent")), Raised::Absent);
    }

    #[test]
    fn each_outcome_says_what_it_came_to() {
        for raised in [Raised::Absent, Raised::Already, Raised::Asked, Raised::Refused] {
            assert!(!raised.said().is_empty(), "{raised:?} says something");
        }
    }
}
