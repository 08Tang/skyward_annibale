//! Asking the phone maker's update server for the newest system and
//! downloading its full package; and asking GitHub for Skyward's newest
//! release, downloading its build for the phone and writing it into the
//! recovery partitions: on a thread of their own that tells the screen how
//! far they have come.

use anyhow::{Context, Result};
use skyward_install::upgrade;
use skyward_net::download::{self, Damaged, Failure, NoRoom, Progress, Refused, Sum};
use skyward_net::github::{self, Build, Newest};
use skyward_net::http::{self, Servers};
use skyward_net::reach::{self, Reach};
use skyward_net::release::{self, Installed, Release, Unanswered};
use skyward_vab::Slot;
use std::os::fd::RawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

/// The most often how far a download has come is told.
const PROGRESS_EVERY: Duration = Duration::from_millis(200);

/// How long the speed shown is taken over.
const RATE_OVER: Duration = Duration::from_secs(3);

/// How many tries in a row may fail with nothing more downloaded before
/// the download gives up, and how long it waits between them.
const TRIES: u32 = 5;
const BETWEEN: Duration = Duration::from_secs(3);

/// How long each download source is measured before the fastest is used.
const PROBE: Duration = Duration::from_secs(3);

/// How long the fastest source for one download is remembered.
const PREFERRED_FOR: Duration = Duration::from_secs(30 * 60);

#[derive(Clone)]
struct Preferred {
    url: String,
    at: Instant,
}

static PREFERRED: OnceLock<Mutex<Option<Preferred>>> = OnceLock::new();

pub enum Task {
    /// Ask `server` for the newest system for `installed`. Where that
    /// fails, `probe`, the page that tells whether the network joined
    /// reaches the internet, says whether the network is why.
    Check {
        server: String,
        probe: String,
        installed: Installed,
    },
    /// Download `release`'s full package to `path`; `server` is asked again
    /// for where it is when the addresses it gave have run out.
    Download {
        server: String,
        installed: Installed,
        release: Release,
        path: PathBuf,
    },
    /// Ask GitHub for Skyward's newest release and its zip for the phone
    /// `codename` names; where that fails, `probe` says whether the network
    /// is why.
    CheckSkyward { probe: String, codename: String },
    /// Download `build` to `path`, check that its image is Skyward for the
    /// phone `codename` names, and write the image into the recovery
    /// partitions of the slot that starts next and of any other slot that
    /// holds Skyward.
    UpdateSkyward {
        build: Build,
        codename: String,
        path: PathBuf,
    },
}

pub enum Event {
    Checked(Result<Option<Release>, Unanswered>),
    /// What GitHub answered of Skyward's newest release.
    SkywardChecked(Result<Newest, Unanswered>),
    /// How far the download has come, and how fast it goes, in bytes a
    /// second.
    Progress(Progress, u64),
    /// Skyward's build is written into the recovery partitions and read
    /// back: bytes so far, of all.
    Writing {
        done: u64,
        total: u64,
    },
    Ended(End),
}

pub enum End {
    Downloaded,
    /// Skyward's build is in the recovery partitions of these slots.
    Updated(Vec<Slot>),
    Stopped,
    Failed(Failure),
    /// Skyward's build could not go in, or where it goes could not be told:
    /// why. Nothing was written.
    NotWritten(String),
    /// Writing Skyward's build into the recovery partitions of these slots
    /// failed, after some of it was written: why.
    WriteFailed(Vec<Slot>, String),
}

pub struct Worker {
    stop: Arc<AtomicBool>,
    /// The connection in use, which stopping cuts at once.
    agent: Arc<Mutex<Option<ureq::Agent>>>,
    /// The screen lets it go once the work has ended, or once it has been
    /// told to stop.
    worker: skyward_sys::worker::Worker<Event>,
}

impl Worker {
    pub fn start(task: Task, servers: Servers) -> Result<Worker> {
        let stop = Arc::new(AtomicBool::new(false));
        let agent: Arc<Mutex<Option<ureq::Agent>>> = Arc::new(Mutex::new(None));
        let (stopping, using) = (stop.clone(), agent.clone());
        let worker = skyward_sys::worker::Worker::start("updates", move |teller| {
            let tell = |event: Event| {
                teller.tell(event);
            };
            match task {
                // The package's exact length is asked of where it is
                // downloaded from: the server gives it rounded, in units
                // of 1024. Without it the rounded one shows.
                Task::Check {
                    server,
                    probe,
                    installed,
                } => {
                    let a = http::agent(servers.clone());
                    *using
                        .lock()
                        .expect("the agent is not held by a thread that panicked") =
                        Some(a.clone());
                    let mut answer = release::ask(&a, &server, &installed)
                        .map_err(|e| unanswered(&e, &server, &probe, &servers));
                    if let Ok(Some(r)) = &mut answer {
                        match release::length(&a, &r.urls[0]) {
                            Ok(length) => r.length = Some(length),
                            Err(e) => {
                                let _ =
                                    skyward_sys::kmsg(&format!("how long {} is: {e:#}", r.name));
                            }
                        }
                    }
                    tell(Event::Checked(answer));
                }
                // A signed address runs out a day after the server gave
                // it: once one is turned down, the server is asked again
                // for where the package is.
                Task::Download {
                    server,
                    installed,
                    release,
                    path,
                } => {
                    let mut asked_again = false;
                    let mut refused = |a: &ureq::Agent| {
                        if asked_again {
                            return Again::Retry;
                        }
                        asked_again = true;
                        match release::ask(a, &server, &installed) {
                            Ok(Some(fresh)) if fresh.md5 == release.md5 => Again::From(fresh.urls),
                            Ok(_) => Again::End(End::Failed(Failure::Changed)),
                            Err(again) => {
                                let _ = skyward_sys::kmsg(&format!(
                                    "asking where {} is: {again:#}",
                                    release.name
                                ));
                                Again::Retry
                            }
                        }
                    };
                    let sum = Sum::Md5(release.md5.clone());
                    let end = fetch(
                        &servers,
                        release.urls.clone(),
                        &sum,
                        &release.name,
                        &path,
                        &stopping,
                        &using,
                        &tell,
                        &mut refused,
                    );
                    tell(Event::Ended(end));
                }
                Task::CheckSkyward { probe, codename } => {
                    let a = http::agent(servers.clone());
                    *using
                        .lock()
                        .expect("the agent is not held by a thread that panicked") =
                        Some(a.clone());
                    let answer = github::ask(&a, &codename, skyward_info::VERSION)
                        .map_err(|e| unanswered(&e, github::LATEST, &probe, &servers));
                    tell(Event::SkywardChecked(answer));
                }
                // GitHub's API hands the zip out at a short-lived address
                // of the file itself, which it gives again once one has
                // run out and is turned down; the release's link on
                // github.com is tried after it.
                Task::UpdateSkyward {
                    build,
                    codename,
                    path,
                } => {
                    let a = http::agent(servers.clone());
                    *using
                        .lock()
                        .expect("the agent is not held by a thread that panicked") =
                        Some(a.clone());
                    let located = github::locate(&a, &build.url, skyward_info::VERSION)
                        .inspect_err(|e| {
                            let _ = skyward_sys::kmsg(&format!(
                                "asking GitHub where {} is: {e:#}",
                                build.name
                            ));
                        })
                        .ok();
                    let mut located_again = false;
                    let mut refused = |a: &ureq::Agent| {
                        if located_again {
                            return Again::Retry;
                        }
                        located_again = true;
                        match github::locate(a, &build.url, skyward_info::VERSION) {
                            Ok(fresh) => Again::From(build.sources(Some(fresh))),
                            Err(e) => {
                                let _ = skyward_sys::kmsg(&format!(
                                    "asking GitHub again where {} is: {e:#}",
                                    build.name
                                ));
                                Again::Retry
                            }
                        }
                    };
                    let sum = Sum::Sha256(build.sha256.clone());
                    let end = match fetch(
                        &servers,
                        build.sources(located),
                        &sum,
                        &build.name,
                        &path,
                        &stopping,
                        &using,
                        &tell,
                        &mut refused,
                    ) {
                        // Stopping asked for as the last of it came is
                        // stopping still: nothing is written.
                        End::Downloaded if stopping.load(Ordering::Relaxed) => End::Stopped,
                        End::Downloaded => write(&path, &codename, &tell),
                        other => other,
                    };
                    // The zip is held in memory, and is done with once
                    // whole; a part stays, to go on from.
                    match std::fs::remove_file(&path) {
                        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                            let _ = skyward_sys::kmsg(&format!("delete {}: {e}", path.display()));
                        }
                        _ => {}
                    }
                    tell(Event::Ended(end));
                }
            }
        })?;
        Ok(Worker {
            stop,
            agent,
            worker,
        })
    }

    /// Readable while events wait.
    pub fn fd(&self) -> RawFd {
        self.worker.fd()
    }

    /// The events since the last call.
    pub fn events(&self) -> Vec<Event> {
        self.worker.events()
    }

    /// Stop at once, cutting the connection in use.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(a) = self
            .agent
            .lock()
            .expect("the agent is not held by a thread that panicked")
            .take()
        {
            a.shutdown();
        }
    }
}

/// Why `server` gave no answer: that the network joined wants a login
/// first, or reaches nothing, where `probe` shows it, rather than how
/// asking failed through such a network.
fn unanswered(e: &anyhow::Error, server: &str, probe: &str, servers: &Servers) -> Unanswered {
    let dns = servers
        .lock()
        .expect("the DNS servers are not held by a thread that panicked")
        .clone();
    let reach = reach::probe(probe, &dns);
    let _ = skyward_sys::kmsg(&format!(
        "asking {server}: {e:#}; the network joined {reach}"
    ));
    match reach {
        Reach::Online => Unanswered::Failed(format!("{e:#}")),
        Reach::Login(_) => Unanswered::Login,
        Reach::Offline(why) => Unanswered::Offline(why),
    }
}

/// What a server turning the download down leads to.
enum Again {
    /// Go on from these addresses.
    From(Vec<String>),
    /// Try again, as after any other failure.
    Retry,
    End(End),
}

/// Download the package `name`, whose digest is `sum`, to `path`, from
/// `urls` in turn, trying again while the connection fails and something
/// more comes each time; `refused` says what a server turning it down leads
/// to.
#[allow(clippy::too_many_arguments)]
fn fetch(
    servers: &Servers,
    urls: Vec<String>,
    sum: &Sum,
    name: &str,
    path: &Path,
    stop: &AtomicBool,
    using: &Mutex<Option<ureq::Agent>>,
    tell: &dyn Fn(Event),
    refused: &mut dyn FnMut(&ureq::Agent) -> Again,
) -> End {
    let mut urls = rank(servers, urls, stop);
    let mut url = 0;
    let mut failed = 0;
    let mut samples: Vec<(Instant, u64)> = Vec::new();
    let mut told = Instant::now() - PROGRESS_EVERY;
    loop {
        if stop.load(Ordering::Relaxed) {
            return End::Stopped;
        }
        let a = http::agent(servers.clone());
        *using
            .lock()
            .expect("the agent is not held by a thread that panicked") = Some(a.clone());
        let mut came = None::<u64>;
        let mut gained = false;
        let mut last = None;
        let mut progress = |p: Progress| {
            if let Progress::Downloading { done, .. } = p {
                let now = Instant::now();
                if came.is_some_and(|before| done > before) {
                    gained = true;
                }
                came = Some(done);
                samples.push((now, done));
                samples.retain(|(at, _)| now.duration_since(*at) <= RATE_OVER);
            }
            let rate = match (samples.first(), samples.last()) {
                (Some((t0, d0)), Some((t1, d1))) if t1 > t0 => {
                    ((d1 - d0) as f64 / t1.duration_since(*t0).as_secs_f64()) as u64
                }
                _ => 0,
            };
            if told.elapsed() >= PROGRESS_EVERY {
                told = Instant::now();
                tell(Event::Progress(p, rate));
            }
            last = Some((p, rate));
        };
        let result = download::fetch(&a, &urls[url], path, sum, stop, &mut progress);
        // How far it came shows in the end, whatever stopped it.
        if let Some((p, rate)) = last {
            tell(Event::Progress(p, rate));
        }
        let e = match result {
            Ok(()) => return End::Downloaded,
            Err(e) => e,
        };
        if e.is::<skyward_net::Stopped>() || stop.load(Ordering::Relaxed) {
            return End::Stopped;
        }
        if let Some(room) = e.downcast_ref::<NoRoom>() {
            return End::Failed(Failure::NoRoom {
                needed: room.needed,
                free: room.free,
            });
        }
        if e.is::<Damaged>() {
            return End::Failed(Failure::Damaged);
        }
        let _ = skyward_sys::kmsg(&format!(
            "downloading {name} from {}: {e:#}",
            host(&urls[url])
        ));
        if e.is::<Refused>() {
            match refused(&a) {
                Again::From(fresh) => {
                    urls = rank(servers, fresh, stop);
                    url = 0;
                    continue;
                }
                Again::End(end) => return end,
                Again::Retry => {}
            }
        }
        failed = if gained { 1 } else { failed + 1 };
        if failed >= TRIES {
            return End::Failed(Failure::Other(format!("{e:#}")));
        }
        url = (url + 1) % urls.len();
        let until = Instant::now() + BETWEEN;
        while Instant::now() < until {
            if stop.load(Ordering::Relaxed) {
                return End::Stopped;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

/// Measure `urls` at the same time and put the fastest first. Sources that
/// sent nothing keep their order after the others. A source measured before
/// within [`PREFERRED_FOR`] is put first when it still sends something.
fn rank(servers: &Servers, urls: Vec<String>, stop: &AtomicBool) -> Vec<String> {
    if urls.len() < 2 || stop.load(Ordering::Relaxed) {
        return urls;
    }
    let speeds: Vec<(usize, u64)> = thread::scope(|scope| {
        let probes: Vec<_> = urls
            .iter()
            .enumerate()
            .map(|(i, url)| {
                let servers = servers.clone();
                (i, scope.spawn(move || (i, probe(&servers, url, stop))))
            })
            .collect();
        probes
            .into_iter()
            .map(|(i, probe)| probe.join().unwrap_or((i, 0)))
            .collect()
    });
    let cached = preferred();
    let (ranked, fastest) = order(urls, speeds, cached.as_deref());
    if let Some(url) = fastest {
        remember(url);
    }
    ranked
}

/// How many bytes `url` sends within [`PROBE`].
fn probe(servers: &Servers, url: &str, stop: &AtomicBool) -> u64 {
    use std::io::Read;
    let agent = http::agent(servers.clone());
    let request = agent.get(url).config().timeout_global(Some(PROBE)).build();
    let mut response = match request.call() {
        Ok(response) if response.status().is_success() => response,
        _ => return 0,
    };
    let mut body = response.body_mut().with_config().limit(u64::MAX).reader();
    let mut buf = [0u8; 64 * 1024];
    let mut bytes = 0;
    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        match body.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => bytes += n as u64,
        }
    }
    bytes
}

fn order(
    urls: Vec<String>,
    mut speeds: Vec<(usize, u64)>,
    cached: Option<&str>,
) -> (Vec<String>, Option<String>) {
    if speeds.iter().all(|(_, speed)| *speed == 0) {
        return (urls, None);
    }
    speeds.sort_by(|(ia, a), (ib, b)| b.cmp(a).then_with(|| ia.cmp(ib)));
    let fastest = speeds[0].0;
    let first = cached
        .and_then(|url| urls.iter().position(|candidate| candidate == url))
        .filter(|i| speeds.iter().any(|(i0, speed)| i0 == i && *speed > 0))
        .unwrap_or(fastest);
    let mut indices: Vec<usize> = speeds.into_iter().map(|(i, _)| i).collect();
    if first != fastest {
        indices.retain(|i| *i != first);
        indices.insert(0, first);
    }
    let winner = urls[first].clone();
    (
        indices
            .into_iter()
            .filter_map(|i| urls.get(i).cloned())
            .collect(),
        Some(winner),
    )
}

fn preferred() -> Option<String> {
    let mut preferred = PREFERRED
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("the preferred download source is not held by a thread that panicked");
    if preferred
        .as_ref()
        .is_some_and(|p| p.at.elapsed() >= PREFERRED_FOR)
    {
        *preferred = None;
    }
    preferred.as_ref().map(|p| p.url.clone())
}

fn remember(url: String) {
    *PREFERRED
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("the preferred download source is not held by a thread that panicked") =
        Some(Preferred {
            url,
            at: Instant::now(),
        });
}

/// Write the image in the zip at `path`, once it is Skyward for the phone
/// `codename` names, into the recovery partitions of the slot that starts
/// next and of any other slot that holds Skyward, telling how far that has
/// come.
fn write(path: &Path, codename: &str, tell: &dyn Fn(Event)) -> End {
    let dir = Path::new(skyward_block::BY_NAME);
    let ready = || -> Result<(Vec<u8>, Vec<Slot>)> {
        let image = upgrade::unzip(path)?;
        upgrade::check(&image, codename)?;
        let next = skyward_vab::boot::next_slot().context("ask which slot starts next")?;
        Ok((image, upgrade::slots(dir, next)?))
    };
    let (image, slots) = match ready() {
        Ok(ready) => ready,
        Err(e) => return End::NotWritten(format!("{e:#}")),
    };
    let mut told: Option<Instant> = None;
    let written = upgrade::write(dir, &image, &slots, &mut |done, total| {
        if done == total || told.is_none_or(|t| t.elapsed() >= PROGRESS_EVERY) {
            tell(Event::Writing { done, total });
            told = Some(Instant::now());
        }
    });
    let _ = skyward_sys::kmsg(&format!(
        "Skyward's build written to {slots:?}: {written:?}"
    ));
    match written {
        Ok(()) => End::Updated(slots),
        // The first part written is told at once, so nothing told is
        // nothing written.
        Err(e) if told.is_none() => End::NotWritten(format!("{e:#}")),
        Err(e) => End::WriteFailed(slots, format!("{e:#}")),
    }
}

/// The host of a download's address, for the log; the signature after it
/// stays out.
fn host(url: &str) -> &str {
    url.split('/').nth(2).unwrap_or(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn urls(n: usize) -> Vec<String> {
        (0..n)
            .map(|i| format!("https://mirror-{i}/file.zip"))
            .collect()
    }

    #[test]
    fn the_fastest_source_comes_first_and_zeroes_keep_their_order() {
        let urls = urls(4);
        let (ordered, fastest) = order(urls.clone(), vec![(0, 0), (1, 30), (2, 0), (3, 50)], None);
        assert_eq!(
            ordered,
            [
                urls[3].clone(),
                urls[1].clone(),
                urls[0].clone(),
                urls[2].clone()
            ]
        );
        assert_eq!(fastest, Some(urls[3].clone()));
    }

    #[test]
    fn a_cached_source_that_still_sends_anything_is_tried_first() {
        let urls = urls(3);
        let (ordered, fastest) =
            order(urls.clone(), vec![(0, 100), (1, 1), (2, 0)], Some(&urls[1]));
        assert_eq!(ordered, [urls[1].clone(), urls[0].clone(), urls[2].clone()]);
        assert_eq!(
            fastest,
            Some(urls[0].clone()),
            "the cache does not replace the measured winner"
        );
    }

    #[test]
    fn a_dead_cached_source_is_not_preferred() {
        let urls = urls(3);
        let (ordered, _) = order(urls.clone(), vec![(0, 0), (1, 20), (2, 0)], Some(&urls[0]));
        assert_eq!(ordered, [urls[1].clone(), urls[0].clone(), urls[2].clone()]);
    }

    #[test]
    fn no_source_sending_anything_keeps_the_original_order_and_cache() {
        let urls = urls(3);
        assert_eq!(
            order(urls.clone(), vec![(0, 0), (1, 0), (2, 0)], Some(&urls[2])),
            (urls, None)
        );
    }
}
