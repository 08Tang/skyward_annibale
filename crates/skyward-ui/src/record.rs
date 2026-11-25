//! Screen recording: the frames a page draws are shrunk to the size chosen,
//! encoded as JPEG, and laid into an AVI (MJPEG) at a fixed frame rate.
//!
//! Recovery has no MediaCodec and no ffmpeg; all there is is the CPU. One
//! 1200×2608 RGBA frame is 12.5 MB, and the full picture encodes to about
//! 170–280 KB of JPEG. Several threads encode, and one more writes what they
//! finish into the file along a timeline. A page is not moving every frame,
//! and where encoding cannot keep up the writer writes the last frame again:
//! the player sees a fixed frame rate, and the page never waits on an encode.
//!
//! Three things came out of the take of 2026-11-21, which held 118 seconds of
//! one unmoving picture:
//!
//! * The writer filled the timeline come what may, and falling behind only
//!   made it fall further behind, until the end dumped hundreds of megabytes
//!   of repeated frames at once. Now falling more than two slots behind moves
//!   the timeline to the present instead of filling them.
//! * Between the encoders and the writer was a channel of eight, and when an
//!   install took the disk the encoders blocked in `send`, the queue filled,
//!   and the page began dropping frames it never got back. Now the writer
//!   takes what has arrived and keeps only the newest frame, so nothing
//!   blocks.
//! * One 1200×2608 frame encodes at only 16 frames a second over three
//!   threads, so a 30-frame timeline had to be filled. Now the default is half
//!   the panel (600×1304), which leaves the encoders room and the picture
//!   actually moving.
//!
//! The file is plain AVI: `RIFF` / `LIST hdrl` (`avih`, `strh`, `strf`) /
//! `LIST movi` (a run of `00dc`) / `idx1`. Cameras, galleries, VLC and ffmpeg
//! all open it.

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::canvas::Canvas;
use crate::mp4::Mp4;

/// The sizes offered, as a share of the panel: whole, three quarters, half,
/// three eighths.
pub const SCALES: [(u32, u32); 4] = [(1, 1), (3, 4), (1, 2), (3, 8)];

/// The qualities offered, as the JPEG quality itself: high, medium, low.
pub const QUALITIES: [u8; 3] = [88, 75, 60];

/// The frame rates offered, in frames a second.
pub const RATES: [u32; 4] = [60, 30, 24, 15];

/// The container: AVI (MJPEG) or MP4 (mp4v carrying a JPEG elementary
/// stream).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Avi,
    Mp4,
}

impl Format {
    /// The file name's suffix.
    pub fn ext(self) -> &'static str {
        match self {
            Format::Avi => "avi",
            Format::Mp4 => "mp4",
        }
    }
}

/// The choice made for you: half the panel, medium, 30 frames -- three
/// encoders at 20 ms a frame leave 30 frames a second room, at about 90 MB a
/// minute.
pub const DEFAULT_SCALE: usize = 2;
pub const DEFAULT_QUALITY: usize = 1;
pub const DEFAULT_FPS: usize = 1;

/// How many frames are kept in memory for the encoders. More than this is
/// rubbish collected, and dropping is better.
const POOL: usize = 6;

/// An AVI's lengths are 32 bits: the file stops here rather than breaking
/// at 4 GB.
const MOST_BYTES: u64 = 3_900_000_000;

/// How far behind the timeline the writer may fall before it moves the
/// timeline to the present rather than filling the slots.
const LATE_SLOTS: u64 = 2;

/// The size, quality and frame rate one take is recorded at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub width: u32,
    pub height: u32,
    pub quality: u8,
    pub fps: u32,
    pub format: Format,
}

impl Settings {
    /// The `scale`th size on the `panel`, the `quality`th quality and the
    /// `fps`th frame rate.
    pub fn of(panel: (u32, u32), scale: usize, quality: usize, fps: usize) -> Settings {
        let (n, d) = SCALES[scale.min(SCALES.len() - 1)];
        let even = |v: u32| ((v / 2) * 2).max(2);
        Settings {
            width: even((panel.0 * n + d / 2) / d),
            height: even((panel.1 * n + d / 2) / d),
            quality: QUALITIES[quality.min(QUALITIES.len() - 1)],
            fps: RATES[fps.min(RATES.len() - 1)],
            format: Format::Avi,
        }
    }

    /// The same take in another container.
    pub fn with_format(mut self, format: Format) -> Settings {
        self.format = format;
        self
    }

    /// How many pixels this size holds.
    pub fn pixels(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// About how big one frame is, by rule of thumb: a JPEG's bits a pixel
    /// follow its quality.
    pub fn per_frame(&self) -> u64 {
        // 1200×2608 at quality 80 measured 169 KB on average, about 0.43
        // bits a pixel; higher quality is more, lower is less.
        let bits = match self.quality {
            0..=65 => 0.30,
            66..=80 => 0.42,
            81..=92 => 0.62,
            _ => 0.90,
        };
        (self.pixels() as f64 * bits / 8.0) as u64
    }
}

/// One take, as it came out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub path: PathBuf,
    /// The frames written into the file, the repeated ones among them.
    pub frames: u64,
    /// How many bytes the file holds.
    pub bytes: u64,
    /// How long it runs, by the frame rate.
    pub millis: u64,
    /// The frames left out because encoding could not keep up.
    pub dropped: u64,
}

/// A take in progress. Dropped rather than [`Recorder::finish`]ed, the
/// writer finishes the half-written file into one that plays, and stops.
pub struct Recorder {
    tx: Vec<SyncSender<Job>>,
    /// Pausing and resuming is the writer's to keep: the timeline is in its
    /// hands, and only it can say how long a pause was.
    ctl: Sender<bool>,
    /// While paused the page does not even copy its canvas: a frame is
        /// twelve megabytes, and a copy would be thrown away.
    paused: Arc<AtomicBool>,
    pool: Arc<Mutex<Vec<Vec<u8>>>>,
    /// How many frames have gone to the encoders, which is the next one's
    /// number.
    taken: Arc<AtomicU64>,
    dropped: Arc<AtomicU64>,
    writer: Option<JoinHandle<Result<Summary>>>,
    encoders: Vec<JoinHandle<()>>,
    source: (u32, u32),
    size: (u32, u32),
    path: PathBuf,
}

struct Job {
    seq: u64,
    rgba: Vec<u8>,
}

impl Recorder {
    /// Begin a take on the open `file`, written at `path`. `source` is the
    /// page canvas's size; `settings` is the size, quality and frame rate it
    /// is recorded at.
    pub fn start(file: File, path: PathBuf, source: (u32, u32), settings: Settings) -> Result<Recorder> {
        let fps = settings.fps.max(1);
        let (w, h) = (settings.width, settings.height);
        let sink = match settings.format {
            Format::Avi => Sink::Avi(Avi::new(file, path.clone(), w, h, fps).context("write the AVI's header")?),
            Format::Mp4 => Sink::Mp4(Mp4::new(file, path.clone(), w, h, fps).context("write the MP4's header")?),
        };
        let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        let count = (cores / 2).clamp(2, 4);
        let pool: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
        // Encoded frames go to the writer in their numbered order; the
        // channel is short, so a slow encode has the writer repeat a frame
        // rather than pile memory up.
        let (done_tx, done_rx) = mpsc::sync_channel::<(u64, Vec<u8>)>(count * 2 + 2);
        let (ctl, ctl_rx) = mpsc::channel::<bool>();
        let paused = Arc::new(AtomicBool::new(false));
        let taken = Arc::new(AtomicU64::new(0));
        let dropped = Arc::new(AtomicU64::new(0));
        let xs = spans(source.0, w);
        let ys = spans(source.1, h);
        let mut tx = Vec::with_capacity(count);
        let mut encoders = Vec::with_capacity(count);
        for at in 0..count {
            let (job_tx, job_rx) = mpsc::sync_channel::<Job>(1);
            tx.push(job_tx);
            let done_tx = done_tx.clone();
            let pool = Arc::clone(&pool);
            let (xs, ys) = (xs.clone(), ys.clone());
            let (sw, sh) = source;
            let quality = settings.quality;
            encoders.push(
                std::thread::Builder::new()
                    .name(format!("record encode {at}"))
                    .spawn(move || {
                        while let Ok(job) = job_rx.recv() {
                            let mut rgb = Vec::new();
                            shrink(&job.rgba, sw as usize, sh as usize, &xs, &ys, &mut rgb);
                            match encode(&rgb, w, h, quality) {
                                Ok(jpeg) => {
                                    // The writer taking no more is
                                    // nothing: the frame is lost and the page
                                    // does not wait on it.
                                    let _ = done_tx.send((job.seq, jpeg));
                                }
                                Err(e) => {
                                    let _ = skyward_kmsg(&format!("record: a frame was not encoded: {e:#}"));
                                }
                            }
                            // The buffer goes back to the pool: a frame
                            // is 12 MB, and the system is not asked for one
                            // every time.
                            if let Ok(mut pool) = pool.lock()
                                && pool.len() < POOL
                            {
                                pool.push(job.rgba);
                            }
                        }
                    })
                    .context("start a thread to encode the frames")?,
            );
        }
        drop(done_tx);
        let writer = std::thread::Builder::new()
            .name("record mux".into())
            .spawn(move || mux(sink, done_rx, ctl_rx, fps))
            .context("start the thread that writes the file")?;
        Ok(Recorder {
            tx,
            ctl,
            paused,
            pool,
            taken,
            dropped,
            writer: Some(writer),
            encoders,
            source,
            size: (w, h),
            path,
        })
    }

    /// The file this take is written to.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// How wide and tall the recording is.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Pause or resume this take. While paused the page copies no canvas and
    /// the writer fills no slot; resumed, the timeline picks up where it
    /// stopped, so the file has neither a gap nor one picture held still.
    pub fn set_paused(&self, on: bool) {
        self.paused.store(on, Ordering::Relaxed);
        let _ = self.ctl.send(on);
    }

    /// Called once a page has drawn a frame: the canvas is copied whole into
    /// a reclaimed buffer and handed to an encoder that is free. Copying a
    /// block of memory is an order of magnitude faster than dropping alpha a
    /// pixel at a time, and the shrinking and the encoding happen on other
    /// threads, so the page spends two or three milliseconds here.
    pub fn push(&self, canvas: &Canvas) {
        if self.paused.load(Ordering::Relaxed) {
            return;
        }
        let (w, h) = (canvas.w, canvas.h);
        if (w, h) != self.source || canvas.data.len() != (w as usize) * (h as usize) * 4 {
            return;
        }
        let mut job = self.spare(canvas.data.len());
        job.rgba.copy_from_slice(&canvas.data);
        let seq = self.taken.load(Ordering::Relaxed);
        job.seq = seq;
        // Hand it to the encoders in turn: whichever is free takes it, and
        // where all are busy the frame is dropped.
        let count = self.tx.len();
        for k in 0..count {
            let at = (seq as usize + k) % count;
            match self.tx[at].try_send(job) {
                Ok(()) => {
                    self.taken.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                Err(TrySendError::Full(back)) | Err(TrySendError::Disconnected(back)) => job = back,
            }
        }
        self.dropped.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut pool) = self.pool.lock()
            && pool.len() < POOL
        {
            pool.push(job.rgba);
        }
    }

    /// A buffer big enough from the pool, or a new one.
    fn spare(&self, len: usize) -> Job {
        let mut rgba = self.pool.lock().ok().and_then(|mut pool| pool.pop()).unwrap_or_default();
        if rgba.len() != len {
            rgba = vec![0u8; len];
        }
        Job { seq: 0, rgba }
    }

    /// Finish: take no more frames, wait for the encoders to be done, and
    /// for the writer to fill in the index.
    pub fn finish(mut self) -> Result<Summary> {
        self.tx.clear();
        for e in self.encoders.drain(..) {
            let _ = e.join();
        }
        let summary = match self.writer.take() {
            Some(w) => w.join().map_err(|_| anyhow::anyhow!("the thread that writes the file panicked"))??,
            None => anyhow::bail!("the file was already written"),
        };
        Ok(Summary { dropped: self.dropped.load(Ordering::Relaxed) + summary.dropped, ..summary })
    }
}

impl Drop for Recorder {
    /// Dropped without `finish`: the channels are closed and the writer
    /// finishes on its own. The threads are not waited for here -- where this
    /// is dropped may be the page's own way out, and waiting would only make
    /// that harder.
    fn drop(&mut self) {
        self.tx.clear();
    }
}

/// One RGBA frame shrunk into the `xs` × `ys` grid, each cell an average,
/// written as RGB. Averaging is far clearer than dropping pixels when text is
/// shrunk: dropping breaks a stroke, averaging only blurs it.
fn shrink(src: &[u8], sw: usize, _sh: usize, xs: &[(u32, u32)], ys: &[(u32, u32)], out: &mut Vec<u8>) {
    let (dw, dh) = (xs.len(), ys.len());
    out.clear();
    out.resize(dw * dh * 3, 0);
    if xs.iter().all(|&(_, n)| n == 1) && ys.iter().all(|&(_, n)| n == 1) {
        // The whole picture: alpha dropped, written by index, which is
        // faster than extending a pixel at a time.
        let mut at = 0;
        for px in src.as_chunks::<4>().0 {
            out[at] = px[0];
            out[at + 1] = px[1];
            out[at + 2] = px[2];
            at += 3;
        }
        return;
    }
    let mut at = 0;
    for &(y0, ny) in ys {
        for &(x0, nx) in xs {
            let mut sum = [0u32; 3];
            for y in y0..y0 + ny {
                let row = ((y as usize) * sw + x0 as usize) * 4;
                for x in 0..nx as usize {
                    let p = row + x * 4;
                    sum[0] += src[p] as u32;
                    sum[1] += src[p + 1] as u32;
                    sum[2] += src[p + 2] as u32;
                }
            }
            let n = (nx * ny).max(1);
            out[at] = (sum[0] / n) as u8;
            out[at + 1] = (sum[1] / n) as u8;
            out[at + 2] = (sum[2] / n) as u8;
            at += 3;
        }
    }
}

/// Which stretch of the source `src` each cell of `dst` takes: where it
/// starts and how many it covers.
fn spans(src: u32, dst: u32) -> Vec<(u32, u32)> {
    let dst = dst.max(1);
    (0..dst)
        .map(|d| {
            let a = (d as u64 * src as u64 / dst as u64) as u32;
            let b = ((d as u64 + 1) * src as u64).div_ceil(dst as u64) as u32;
            let b = b.max(a + 1).min(src);
            (a, b - a)
        })
        .collect()
}

/// RGB as a JPEG. The `image` encoder samples at 4:2:2, which keeps the
/// edges of text clear enough.
fn encode(rgb: &[u8], width: u32, height: u32, quality: u8) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(rgb.len() / 8);
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality);
    encoder.encode(rgb, width, height, image::ExtendedColorType::Rgb8).context("encode a JPEG frame")?;
    Ok(out)
}

/// What the two containers are asked alike.
enum Sink {
    Avi(Avi),
    Mp4(Mp4),
}

impl Sink {
    fn path(&self) -> PathBuf {
        match self {
            Sink::Avi(a) => a.path(),
            Sink::Mp4(m) => m.path(),
        }
    }

    /// Whether it can take one more frame.
    fn has_room(&self) -> bool {
        match self {
            Sink::Avi(a) => a.bytes() < MOST_BYTES,
            Sink::Mp4(m) => !m.full(),
        }
    }

    fn add(&mut self, jpeg: &[u8]) -> Result<()> {
        match self {
            Sink::Avi(a) => a.add(jpeg),
            Sink::Mp4(m) => m.add(jpeg),
        }
    }

    fn frames(&self) -> u64 {
        match self {
            Sink::Avi(a) => a.frames,
            Sink::Mp4(m) => m.frames(),
        }
    }

    fn finish(&mut self) -> Result<u64> {
        match self {
            Sink::Avi(a) => a.finish(),
            Sink::Mp4(m) => m.finish(),
        }
    }
}

/// The writer: lays the encoded frames into the container along the
/// timeline.
///
/// The timeline is fixed: one slot a frame, and where the frame numbered for a
/// slot has not come the last one is written again, so the player sees the
/// length that was recorded -- a slow encode is fewer movements, not fast
/// forward.
///
/// Falling behind is not filled in: when an install takes the disk, one 200 KB
/// repeated frame only makes the lag longer -- that is how the 118 seconds of
/// one unmoving picture came about. Falling more than [`LATE_SLOTS`] behind
/// moves the timeline to the present: the picture is still recorded, and the
/// file is shorter.
fn mux(mut sink: Sink, rx: Receiver<(u64, Vec<u8>)>, ctl: Receiver<bool>, fps: u32) -> Result<Summary> {
    let step = Duration::from_millis(1000) / fps;
    let step_secs = 1.0 / fps as f64;
    let mut newest: Option<Vec<u8>> = None;
    let mut last: Vec<u8> = Vec::new();
    let mut closed = false;
    let mut written = 0u64;
    let mut stale = 0u64;
    let mut skipped = 0u64;
    let mut full = false;
    let mut started: Option<Instant> = None;
    // Pausing: `paused_at` is when it stopped and `held` how long it has
    // been stopped in all. The timeline is "now - start - paused", so a
    // resumed take writes on from where it stopped, with no gap.
    let mut paused_at: Option<Instant> = None;
    let mut held = Duration::ZERO;
    let mut paused_for = 0u64;
    loop {
        while let Ok(on) = ctl.try_recv() {
            match (on, paused_at) {
                (true, None) => paused_at = Some(Instant::now()),
                (false, Some(at)) => {
                    held += at.elapsed();
                    paused_at = None;
                    paused_for += 1;
                }
                _ => {}
            }
        }
        // Take everything the channel holds first, keeping only the newest:
        // where encoding is faster than the disk, the picture may skip a
        // slot, but the queue must not fill -- filled, the encoders stop
        // there and every frame after is the old one.
        loop {
            match rx.try_recv() {
                Ok((_, jpeg)) => {
                    if newest.replace(jpeg).is_some() {
                        stale += 1;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    closed = true;
                    break;
                }
            }
        }
        if started.is_none() {
            // The timeline starts at the first frame, not at the opening of
            // the file: a page just opened and not yet drawn should not record
            // slots of nothing.
            if newest.is_none() {
                if closed {
                    break;
                }
                match rx.recv_timeout(Duration::from_millis(40)) {
                    Ok((_, jpeg)) => newest = Some(jpeg),
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => closed = true,
                }
                continue;
            }
            started = Some(Instant::now());
        }
        let started = started.expect("set above");
        let now = Instant::now();
        // Paused: no slot is written and the timeline does not move on. The
        // frame in hand stays, and is written first on resuming.
        if paused_at.is_some() {
            // Finishing while paused: this frame is not waited for, or
            // `finish` would wait forever for this thread to return.
            if closed {
                break;
            }
            match rx.recv_timeout(Duration::from_millis(20)) {
                Ok((_, jpeg)) => {
                    if newest.replace(jpeg).is_some() {
                        stale += 1;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => closed = true,
            }
            continue;
        }
        let played = (now - started).saturating_sub(held);
        let elapsed = played.as_secs_f64();
        // Too far behind: the timeline moves to the present, and those
        // slots are not filled.
        let behind = (elapsed / step_secs) as u64;
        if behind > written + LATE_SLOTS {
            skipped += behind - written;
            written = behind;
        }
        if closed && newest.is_none() {
            break;
        }
        let due = started + held + step * written as u32;
        if now < due {
            let wait = due.saturating_duration_since(now).min(Duration::from_millis(20));
            match rx.recv_timeout(wait) {
                Ok((_, jpeg)) => {
                    if newest.replace(jpeg).is_some() {
                        stale += 1;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => closed = true,
            }
            continue;
        }
        // This slot is due: the new frame where there is one, and the last
        // frame written again where there is not.
        let jpeg = match newest.take() {
            Some(jpeg) => {
                last = jpeg;
                Some(&last)
            }
            None if !last.is_empty() => Some(&last),
            None => None,
        };
        if let Some(jpeg) = jpeg {
            if !full && sink.has_room() {
                sink.add(jpeg)?;
            } else if !full {
                full = true;
                let _ = skyward_kmsg("record: the file is as big as its container can be; the frames after this are left out");
            }
        }
        written += 1;
        if full {
            std::thread::sleep(step);
        }
    }
    let path = sink.path();
    let frames = sink.frames();
    let bytes = sink.finish()?;
    let millis = (written as u128 * 1000 / fps as u128) as u64;
    let _ = skyward_kmsg(&format!(
        "record: {frames} frames, {:.1} s, {:.1} MB, {:.1} MB/s in {} ({stale} newer frames replaced, {skipped} slots skipped, {paused_for} pauses)",
        millis as f64 / 1000.0,
        bytes as f64 / 1e6,
        bytes as f64 / 1e6 / (millis as f64 / 1000.0).max(0.001),
        path.display(),
    ));
    Ok(Summary { path, frames, bytes, millis, dropped: stale })
}

/// One line to the kernel log. `skyward-sys` is not a dependency of this
/// crate -- the UI has to build on the host too -- so the recording writes
/// `/dev/kmsg` itself, and does without where it cannot.
fn skyward_kmsg(line: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    let mut f = std::fs::OpenOptions::new().write(true).open("/dev/kmsg")?;
    writeln!(f, "<6>skyward: {line}")
}

/// An AVI file: the length fields are written empty and filled in at the
/// end.
struct Avi {
    out: BufWriter<File>,
    path: PathBuf,
    fps: u32,
    pos: u64,
    riff_at: u64,
    hdrl_at: u64,
    strl_at: u64,
    movi_list_at: u64,
    movi_at: u64,
    avih_at: u64,
    strh_at: u64,
    frames: u64,
    biggest: u32,
    bytes: u64,
    index: Vec<(u32, u32)>,
}

impl Avi {
    fn new(file: File, path: PathBuf, w: u32, h: u32, fps: u32) -> Result<Avi> {
        let mut avi = Avi {
            out: BufWriter::with_capacity(1 << 20, file),
            path,
            fps,
            pos: 0,
            riff_at: 0,
            hdrl_at: 0,
            strl_at: 0,
            movi_list_at: 0,
            movi_at: 0,
            avih_at: 0,
            strh_at: 0,
            frames: 0,
            biggest: 0,
            bytes: 0,
            index: Vec::new(),
        };
        avi.riff_at = 0;
        avi.put(b"RIFF")?;
        avi.put(&0u32.to_le_bytes())?;
        avi.put(b"AVI ")?;

        avi.hdrl_at = avi.pos;
        avi.put(b"LIST")?;
        avi.put(&0u32.to_le_bytes())?;
        avi.put(b"hdrl")?;

        avi.avih_at = avi.pos;
        avi.put(b"avih")?;
        avi.put(&56u32.to_le_bytes())?;
        let step = (1_000_000.0 / fps as f64).round() as u32;
        avi.put(&step.to_le_bytes())?; // dwMicroSecPerFrame
        avi.put(&0u32.to_le_bytes())?; // dwMaxBytesPerSec, filled in at the end
        avi.put(&0u32.to_le_bytes())?; // dwPaddingGranularity
        avi.put(&0x10u32.to_le_bytes())?; // AVIF_HASINDEX
        avi.put(&0u32.to_le_bytes())?; // dwTotalFrames, filled in at the end
        avi.put(&0u32.to_le_bytes())?; // dwInitialFrames
        avi.put(&1u32.to_le_bytes())?; // dwStreams
        avi.put(&0u32.to_le_bytes())?; // dwSuggestedBufferSize, filled in at the end
        avi.put(&w.to_le_bytes())?;
        avi.put(&h.to_le_bytes())?;
        avi.put(&[0u8; 16])?; // dwReserved[4]

        avi.strl_at = avi.pos;
        avi.put(b"LIST")?;
        avi.put(&0u32.to_le_bytes())?;
        avi.put(b"strl")?;

        avi.strh_at = avi.pos;
        avi.put(b"strh")?;
        avi.put(&56u32.to_le_bytes())?;
        avi.put(b"vids")?;
        avi.put(b"MJPG")?;
        avi.put(&0u32.to_le_bytes())?; // dwFlags
        avi.put(&0u16.to_le_bytes())?; // wPriority
        avi.put(&0u16.to_le_bytes())?; // wLanguage
        avi.put(&0u32.to_le_bytes())?; // dwInitialFrames
        avi.put(&1u32.to_le_bytes())?; // dwScale
        avi.put(&fps.to_le_bytes())?; // dwRate
        avi.put(&0u32.to_le_bytes())?; // dwStart
        avi.put(&0u32.to_le_bytes())?; // dwLength, filled in at the end
        avi.put(&0u32.to_le_bytes())?; // dwSuggestedBufferSize, filled in at the end
        avi.put(&u32::MAX.to_le_bytes())?; // dwQuality
        avi.put(&0u32.to_le_bytes())?; // dwSampleSize
        avi.put(&0i16.to_le_bytes())?; // rcFrame.left
        avi.put(&0i16.to_le_bytes())?; // rcFrame.top
        avi.put(&(w as i16).to_le_bytes())?; // rcFrame.right
        avi.put(&(h as i16).to_le_bytes())?; // rcFrame.bottom

        avi.put(b"strf")?;
        avi.put(&40u32.to_le_bytes())?;
        avi.put(&40u32.to_le_bytes())?; // biSize
        avi.put(&w.to_le_bytes())?; // biWidth
        avi.put(&h.to_le_bytes())?; // biHeight
        avi.put(&1u16.to_le_bytes())?; // biPlanes
        avi.put(&24u16.to_le_bytes())?; // biBitCount
        avi.put(b"MJPG")?; // biCompression
        avi.put(&(w.saturating_mul(h).saturating_mul(3)).to_le_bytes())?; // biSizeImage
        avi.put(&0u32.to_le_bytes())?;
        avi.put(&0u32.to_le_bytes())?;
        avi.put(&0u32.to_le_bytes())?;
        avi.put(&0u32.to_le_bytes())?;

        avi.movi_list_at = avi.pos;
        avi.put(b"LIST")?;
        avi.put(&0u32.to_le_bytes())?;
        avi.movi_at = avi.pos;
        avi.put(b"movi")?;
        Ok(avi)
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn bytes(&self) -> u64 {
        self.bytes
    }

    /// One more frame into `movi`: a `00dc` chunk, with a zero byte after an
        /// odd length.
    fn add(&mut self, jpeg: &[u8]) -> Result<()> {
        let at = self.pos;
        self.put(b"00dc")?;
        self.put(&(jpeg.len() as u32).to_le_bytes())?;
        self.put(jpeg)?;
        if jpeg.len() % 2 == 1 {
            self.put(&[0u8])?;
        }
        self.index.push(((at - self.movi_at) as u32, jpeg.len() as u32));
        self.frames += 1;
        self.bytes += jpeg.len() as u64;
        self.biggest = self.biggest.max(jpeg.len() as u32);
        Ok(())
    }

    fn put(&mut self, bytes: &[u8]) -> Result<()> {
        self.out.write_all(bytes).context("write the AVI")?;
        self.pos += bytes.len() as u64;
        Ok(())
    }

    /// Fill in the lengths and the index, and say how many bytes the file
    /// holds. The frame count is of those actually written -- the take of
    /// 2026-11-21 said 5373 frames in its header and had 5017 in `idx1`, and
    /// this is why.
    fn finish(&mut self) -> Result<u64> {
        let index_at = self.pos;
        let index = std::mem::take(&mut self.index);
        let frames = index.len() as u64;
        self.put(b"idx1")?;
        self.put(&((index.len() * 16) as u32).to_le_bytes())?;
        for (offset, len) in &index {
            self.put(b"00dc")?;
            self.put(&0x10u32.to_le_bytes())?; // AVIIF_KEYFRAME
            self.put(&offset.to_le_bytes())?;
            self.put(&len.to_le_bytes())?;
        }
        let end = self.pos;
        // The length of the movi LIST: after 'movi' up to before idx1.
        let movi_len = (index_at - (self.movi_list_at + 8)) as u32;
        let hdrl_len = (self.movi_list_at - (self.hdrl_at + 8)) as u32;
        // The length of the strl LIST: after 'strl' up to before LIST
        // movi.
        let strl_len = (self.movi_list_at - (self.strl_at + 8)) as u32;
        self.out.flush().context("flush the AVI")?;
        self.patch32(self.riff_at + 4, (end - 8) as u32)?;
        self.patch32(self.hdrl_at + 4, hdrl_len)?;
        self.patch32(self.strl_at + 4, strl_len)?;
        self.patch32(self.movi_list_at + 4, movi_len)?;
        // avih: the frame count, the most bytes a second, the suggested
        // buffer.
        let every = (self.fps as u64 * (self.bytes.max(1) / frames.max(1)) as u64).min(u32::MAX as u64);
        self.patch32(self.avih_at + 8 + 4, every as u32)?;
        self.patch32(self.avih_at + 8 + 16, frames as u32)?;
        self.patch32(self.avih_at + 8 + 28, self.biggest)?;
        // strh: the length and the suggested buffer.
        self.patch32(self.strh_at + 8 + 32, frames as u32)?;
        self.patch32(self.strh_at + 8 + 36, self.biggest)?;
        self.out.flush().context("flush the AVI")?;
        Ok(end)
    }

    fn patch32(&mut self, at: u64, value: u32) -> Result<()> {
        self.out.seek(SeekFrom::Start(at)).context("seek in the AVI")?;
        self.out.write_all(&value.to_le_bytes()).context("patch the AVI")?;
        self.out.seek(SeekFrom::Start(self.pos)).context("seek back in the AVI")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A finished AVI names in its header the frames, size and index that
    /// were written.
    #[test]
    fn a_finished_avi_names_its_frames_and_its_size() {
        let path = PathBuf::from("/tmp/skyward-record-test.avi");
        let mut avi = Avi::new(File::create(&path).unwrap(), path, 1080, 2400, 30).unwrap();
        avi.put(b"00dc").unwrap();
        avi.put(&4u32.to_le_bytes()).unwrap();
        avi.put(&[0xff, 0xd8, 0xff, 0xd9]).unwrap();
        avi.index.push((4, 4));
        avi.frames += 1;
        avi.bytes += 4;
        avi.biggest = 4;
        let bytes = avi.finish().unwrap();
        let text = std::fs::read("/tmp/skyward-record-test.avi").unwrap();
        assert_eq!(bytes as usize, text.len());
        assert_eq!(&text[..4], b"RIFF");
        assert!(text.windows(4).any(|w| w == b"idx1"));
        let frames = u32::from_le_bytes(text[0x30..0x34].try_into().unwrap());
        assert_eq!(frames, 1);
    }

    /// Every size is a share of the panel, and both its sides are even.
    #[test]
    fn every_scale_is_an_even_share_of_the_panel() {
        let panel = (1200, 2608);
        assert_eq!(Settings::of(panel, 0, 1, 1).width, 1200);
        assert_eq!(Settings::of(panel, 0, 1, 1).height, 2608);
        assert_eq!(Settings::of(panel, 1, 1, 1).width, 900);
        assert_eq!(Settings::of(panel, 1, 1, 1).height, 1956);
        assert_eq!(Settings::of(panel, 2, 1, 1).width, 600);
        assert_eq!(Settings::of(panel, 2, 1, 1).height, 1304);
        assert_eq!(Settings::of(panel, 3, 1, 1).width, 450);
        assert_eq!(Settings::of(panel, 3, 1, 1).height, 978);
    }

    /// Shrinking averages whole blocks, and the colour comes out right.
    #[test]
    fn shrinking_averages_whole_blocks() {
        // 4×2 all red, shrunk to 2×1: four to a cell, still red.
        let src = [255u8, 0, 0, 255].repeat(8);
        let (xs, ys) = (spans(4, 2), spans(2, 1));
        let mut out = Vec::new();
        shrink(&src, 4, 2, &xs, &ys, &mut out);
        assert_eq!(out, vec![255, 0, 0, 255, 0, 0]);
        // Black on the left and white on the right, shrunk to 1×1: half
        // grey.
        let mut src = [0u8, 0, 0, 255].repeat(4);
        src.extend([255u8, 255, 255, 255].repeat(4));
        let mut out = Vec::new();
        shrink(&src, 4, 2, &spans(4, 1), &spans(2, 1), &mut out);
        assert_eq!(out, vec![127, 127, 127]);
    }
}