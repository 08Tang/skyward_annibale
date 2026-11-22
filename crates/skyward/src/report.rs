//! Sending a bug report on a thread of its own, so the page keeps up with how
//! far it has come and can be drawn while megabytes of logs go out.

use anyhow::{Context, Result};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

/// How a report ended.
pub enum Ended {
    /// Taken, and the number it was given.
    Taken(String),
    /// Turned down: why, in the page's words, and whether trying again may
    /// end otherwise.
    Refused { why: String, again: bool },
    /// The logs could not be read to send them, or the sending failed in a
    /// way that is worth trying again.
    Failed(String),
}

pub struct Worker {
    ended: Receiver<Ended>,
    /// Readable once the sending has ended.
    wake: OwnedFd,
    /// How much of the request has gone, and how much there is in all.
    done: Arc<AtomicU64>,
    total: Arc<AtomicU64>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Send a report of `words` with the logs read from `logs`, which is
    /// `kernel.log` and `recovery.log` by name, to the address that keeps
    /// reports.
    pub fn start(words: String, logs: Vec<(String, Vec<u8>)>, servers: skyward_net::http::Servers) -> Result<Worker> {
        let mut fds = [0; 2];
        if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) } != 0 {
            return Err(std::io::Error::last_os_error()).context("make the sending thread's wake pipe");
        }
        let (wake, signal) = unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) };
        let done = Arc::new(AtomicU64::new(0));
        let total = Arc::new(AtomicU64::new(0));
        let (tx, ended) = mpsc::channel();
        let (done_as, total_as) = (Arc::clone(&done), Arc::clone(&total));
        let thread = thread::Builder::new()
            .name("report".into())
            .spawn(move || {
                let ended_as = match send(&words, &logs, &done_as, &total_as, servers) {
                    Ok(id) => Ended::Taken(id),
                    // The address turning a report down is not a failure of
                    // this: it says why, in words the page shows, and whether
                    // trying again may end otherwise.
                    Err(refused) => Ended::Refused { why: skyward_net::report::explain(&refused).to_owned(), again: skyward_net::report::again(&refused) },
                };
                let _ = tx.send(ended_as);
                unsafe { libc::write(signal.as_raw_fd(), [1u8].as_ptr().cast(), 1) };
            })
            .context("start the sending thread")?;
        Ok(Worker { ended, wake, done, total, thread: Some(thread) })
    }

    /// How much of the report has gone, and how much there is in all.
    pub fn progress(&self) -> (u64, u64) {
        (self.done.load(Ordering::Relaxed), self.total.load(Ordering::Relaxed))
    }

    /// How the sending ended, once it has.
    pub fn ended(&self) -> Option<Ended> {
        let mut drain = [0u8; 8];
        while unsafe { libc::read(self.wake.as_raw_fd(), drain.as_mut_ptr().cast(), drain.len()) } > 0 {}
        self.ended.try_recv().ok()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Read the two logs out of `logs` and send them with `words`.
fn send(words: &str, logs: &[(String, Vec<u8>)], done: &Arc<AtomicU64>, total: &Arc<AtomicU64>, servers: skyward_net::http::Servers) -> Result<String, skyward_net::report::Refused> {
    let find = |name: &str| logs.iter().find(|(n, _)| n == name).map(|(_, bytes)| bytes.as_slice()).unwrap_or_default();
    let kernel = find("kernel.log");
    let recovery = find("recovery.log");
    // The address takes no more than this of either log, and a report that
    // carried more would be turned down whole: what is past the end is cut,
    // and the end of a log is what says how it ended.
    let most = skyward_net::report::MOST_LOG as usize;
    let kernel = &kernel[kernel.len().saturating_sub(most)..];
    let recovery = &recovery[recovery.len().saturating_sub(most)..];
    let taken = skyward_net::report::send(words, kernel, recovery, Arc::clone(done), Arc::clone(total), servers)?;
    Ok(taken.id)
}

/// Read the two logs the recovery keeps, as the report sends them: its own
/// and the kernel's.
pub fn logs() -> Result<Vec<(String, Vec<u8>)>> {
    let recovery = std::fs::read(skyward_sys::LOG).with_context(|| format!("read {}", skyward_sys::LOG))?;
    let kernel = skyward_sys::kernel_log()?.into_bytes();
    Ok(vec![("kernel.log".to_owned(), kernel), ("recovery.log".to_owned(), recovery)])
}

/// What a report says it has done, for the log the recovery keeps.
pub fn described(ended: &Ended) -> String {
    match ended {
        Ended::Taken(id) => format!("the report was taken as {id}"),
        Ended::Refused { why, again } => format!("the report was turned down: {why} (again: {again})"),
        Ended::Failed(why) => format!("the report could not be sent: {why}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_report_says_in_the_log_how_it_ended() {
        assert_eq!(described(&Ended::Taken("12".into())), "the report was taken as 12");
        assert!(described(&Ended::Failed("no route".into())).contains("no route"), "the reason is kept");
        assert!(described(&Ended::Refused { why: "too large".into(), again: false }).contains("too large"));
    }
}
