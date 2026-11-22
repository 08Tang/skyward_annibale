//! Sending a report to the address that keeps them: the words the user wrote,
//! and the two logs that say what the recovery did and what the kernel did
//! around it.
//!
//! The request is a multipart form, written out as it is sent so that how far
//! it has come can be told while it goes: a report carries megabytes of logs,
//! and a page that said nothing until it ended would look as if it had
//! stalled.

use crate::http;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::io::{self, Read};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Where reports are sent.
pub const ADDRESS: &str = "https://a.clccv.com/api/Skyward";

/// The key the address checks before it takes a report.
const KEY: &str = "annibale";

/// The most a log may be, as the address says: a report that carried more
/// would be turned down whole.
pub const MOST_LOG: u64 = 10 * 1024 * 1024;

/// The most the words may be, as the address says.
pub const MOST_WORDS: u64 = 1024 * 1024;

/// The name of the field the words go in, and of the two the logs go in.
const WORDS_FIELD: &str = "question";
const KERNEL_FIELD: &str = "kernel.log";
const RECOVERY_FIELD: &str = "recovery.log";

/// What the address answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Taken {
    /// The number the report was given, which is what the user is told to
    /// name when they write of it.
    pub id: String,
}

/// Why a report was not taken.
#[derive(Debug)]
pub enum Refused {
    /// The address answered with an error of its own, which it names by a
    /// code that says what to do about it.
    Said { status: u16, error: String, message: String },
    /// The address answered in words this does not know.
    Unreadable(String),
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refused::Said { status, error, message } => write!(f, "{error} ({status}): {message}"),
            Refused::Unreadable(why) => write!(f, "{why}"),
        }
    }
}

impl std::error::Error for Refused {}

/// What the address answers with, both when it takes a report and when it
/// turns one down.
#[derive(Deserialize)]
struct Answer {
    success: bool,
    question_id: Option<String>,
    error: Option<String>,
    message: Option<String>,
}

/// Send a report of `words` with the two logs `kernel` and `recovery`.
///
/// `progress` is written with how many bytes have gone, and `total` with how
/// many the request is in all, both by the sending thread alone, so a page
/// can show how far it has come.
pub fn send(words: &str, kernel: &[u8], recovery: &[u8], progress: Arc<AtomicU64>, total: Arc<AtomicU64>, servers: http::Servers) -> Result<Taken, Refused> {
    let boundary = boundary();
    let body = Body::new(&boundary, words, kernel, recovery, progress);
    total.store(body.len() as u64, Ordering::Relaxed);
    let agent = http::agent(servers);
    let mut response = agent
        .post(ADDRESS)
        .header("Content-Type", format!("multipart/form-data; boundary={boundary}"))
        .header("Accept", "application/json")
        .header("User-Agent", format!("Skyward/{}", env!("CARGO_PKG_VERSION")))
        .send(ureq::SendBody::from_owned_reader(body))
        .map_err(|e| Refused::Unreadable(format!("{e}")))?;
    let status = response.status().as_u16();
    let text = response.body_mut().read_to_string().map_err(|e| Refused::Unreadable(format!("read the answer: {e}")))?;
    let answer: Answer = serde_json::from_str(&text).map_err(|e| Refused::Unreadable(format!("the address answered in words this does not know ({e}): {}", shortened(&text))))?;
    if answer.success {
        let id = answer.question_id.context("the address took the report but named no number for it")?;
        return Ok(Taken { id });
    }
    Err(Refused::Said { status, error: answer.error.unwrap_or_else(|| "UNKNOWN".into()), message: answer.message.unwrap_or_default() })
}

/// A few of the words the address answered with, for saying what went wrong
/// without writing a page of it.
fn shortened(text: &str) -> String {
    let one_line: String = text.chars().take(200).map(|c| if c.is_control() { ' ' } else { c }).collect();
    if text.chars().count() > 200 { format!("{one_line}…") } else { one_line }
}

/// What says where one field of the form ends and the next begins. It is
/// long and unlikely enough that no log holds it.
fn boundary() -> String {
    format!("----Skyward{:016x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0))
}

/// The request body, written out as it is read: a part for each field, in
/// the order the address reads them.
struct Body<'a> {
    parts: Vec<Part<'a>>,
    /// Where in the whole body the next read starts.
    at: usize,
    progress: Arc<AtomicU64>,
}

enum Part<'a> {
    /// Words written out at once: the form's own lines, and the report.
    Whole(Vec<u8>),
    /// A log, read from what is already in memory.
    Log(&'a [u8]),
}

impl<'a> Body<'a> {
    fn new(boundary: &str, words: &str, kernel: &'a [u8], recovery: &'a [u8], progress: Arc<AtomicU64>) -> Body<'a> {
        let mut parts = Vec::with_capacity(6);
        parts.push(Part::Whole(field(boundary, "key", KEY.as_bytes(), None)));
        parts.push(Part::Whole(field(boundary, WORDS_FIELD, words.as_bytes(), Some("question.txt"))));
        parts.push(Part::Whole(field(boundary, KERNEL_FIELD, &[], Some("kernel.log"))));
        parts.push(Part::Log(kernel));
        parts.push(Part::Whole(field(boundary, RECOVERY_FIELD, &[], Some("recovery.log"))));
        parts.push(Part::Log(recovery));
        // The last line closes the form, and is not a field's own.
        parts.push(Part::Whole(format!("\r\n--{boundary}--\r\n").into_bytes()));
        Body { parts, at: 0, progress }
    }

    fn len(&self) -> usize {
        self.parts
            .iter()
            .map(|p| match p {
                Part::Whole(bytes) => bytes.len(),
                Part::Log(bytes) => bytes.len(),
            })
            .sum()
    }
}

impl Read for Body<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.at >= self.len() {
            return Ok(0);
        }
        // Which part the next byte is in, and how far into it.
        let mut left = self.at;
        let mut n = 0;
        for part in &self.parts {
            let (bytes, len) = match part {
                Part::Whole(bytes) => (bytes.as_slice(), bytes.len()),
                Part::Log(bytes) => (*bytes, bytes.len()),
            };
            if left < len {
                let take = (len - left).min(out.len() - n);
                out[n..n + take].copy_from_slice(&bytes[left..left + take]);
                n += take;
                left = 0;
                if n == out.len() {
                    break;
                }
            } else {
                left -= len;
            }
        }
        self.at += n;
        self.progress.store(self.at as u64, Ordering::Relaxed);
        Ok(n)
    }
}

/// One field of the form: the line that names it, its contents, and the blank
/// line after it.
fn field(boundary: &str, name: &str, contents: &[u8], file: Option<&str>) -> Vec<u8> {
    let mut out = Vec::with_capacity(contents.len() + 256);
    out.extend_from_slice(format!("\r\n--{boundary}\r\n").as_bytes());
    match file {
        Some(filename) => out.extend_from_slice(format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n").as_bytes()),
        None => out.extend_from_slice(format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes()),
    }
    out.extend_from_slice(contents);
    out
}

/// What a refusal means for the one who read it, in the words the page shows:
/// what to do about it, rather than the code alone.
pub fn explain(refused: &Refused) -> &'static str {
    match refused {
        Refused::Said { status, error, .. } => match (status, error.as_str()) {
            (403, "INVALID_KEY") | (_, "INVALID_KEY") => "the address turned the key down; this build cannot send reports",
            (413, "FILE_TOO_LARGE") | (413, "QUESTION_TOO_LARGE") | (413, "PAYLOAD_TOO_LARGE") | (413, _) => "the report is larger than the address takes",
            (400, "MISSING_FIELD") | (400, "DUPLICATE_FIELD") | (400, "UNKNOWN_FIELD") | (400, "INVALID_MULTIPART") | (400, "INVALID_QUESTION") => {
                "the address could not read the form this sent"
            }
            (405, _) => "the address does not take reports this way",
            (415, _) => "the address could not read the form this sent",
            (500, _) => "the address could not keep the report",
            (status, _) if status >= 500 => "the address is not answering as it should",
            _ => "the address turned the report down",
        },
        Refused::Unreadable(_) => "the address could not be reached, or answered in a way this does not know",
    }
}

/// Whether a refusal is worth trying again: an address that is down or a
/// connection that dropped may answer the next time, while one that turned
/// the key down or could not read the form will answer the same way.
pub fn again(refused: &Refused) -> bool {
    match refused {
        Refused::Said { status, error, .. } => *status >= 500 || (*status == 400 && error.is_empty()),
        Refused::Unreadable(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_form_holds_every_field_the_address_asks_for() {
        let body = Body::new("X", "words", b"k", b"r", Arc::new(AtomicU64::new(0)));
        let mut all = Vec::new();
        let mut body = body;
        body.read_to_end(&mut all).unwrap();
        let text = String::from_utf8_lossy(&all);
        for field in ["name=\"key\"", "name=\"question\"", "name=\"kernel.log\"", "name=\"recovery.log\""] {
            assert!(text.contains(field), "{field} is in the form");
        }
        assert!(text.ends_with("--X--\r\n"), "the form is closed");
    }

    #[test]
    fn a_body_reads_out_exactly_the_bytes_it_says_it_is() {
        let body = Body::new("X", "words", b"kernel", b"recovery", Arc::new(AtomicU64::new(0)));
        let len = body.len();
        let mut all = Vec::new();
        let mut body = body;
        body.read_to_end(&mut all).unwrap();
        assert_eq!(all.len(), len, "what it says it is, is what it reads out");
    }

    #[test]
    fn the_progress_written_is_how_much_has_been_read() {
        let progress = Arc::new(AtomicU64::new(0));
        let mut body = Body::new("X", "words", b"kernel", b"recovery", Arc::clone(&progress));
        let len = body.len() as u64;
        let mut buf = [0u8; 7];
        while body.read(&mut buf).unwrap() > 0 {}
        assert_eq!(progress.load(Ordering::Relaxed), len, "the last byte read is the whole of it");
    }

    #[test]
    fn a_refusal_that_turned_the_key_down_is_not_tried_again() {
        let refused = Refused::Said { status: 403, error: "INVALID_KEY".into(), message: String::new() };
        assert!(!again(&refused), "the same key is turned down the same way");
    }

    #[test]
    fn an_address_that_is_down_is_tried_again() {
        let refused = Refused::Said { status: 500, error: "INTERNAL_ERROR".into(), message: String::new() };
        assert!(again(&refused), "a server that failed may answer the next time");
        assert!(again(&Refused::Unreadable("no route".into())), "a connection that dropped may come back");
    }
}
