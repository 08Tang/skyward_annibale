//! The phone's readings taken on a thread of their own, so the screen keeps
//! up: the battery's driver takes milliseconds to answer each read.

use anyhow::{Context, Result};
use skyward_config::DeviceConfig;
use skyward_info::{Battery, DeviceInfo};
use skyward_sys::worker::{self, Told};
use skyward_ui::app::ChargeNow;
use std::os::fd::RawFd;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

/// What the charger was doing as the readings thread read it: `None` where
/// nothing is plugged in. Reading it walks the charger's and the battery's own
/// nodes, which take milliseconds to answer, so it is taken on this thread
/// rather than where the page is drawn.
pub type Charge = Option<ChargeNow>;

/// The battery is read this often, and at once when asked; all the
/// readings every so many times instead.
const BATTERY_EVERY: Duration = Duration::from_secs(1);
const ALL_EVERY: u32 = 30;

pub enum Reading {
    /// The battery, what the charger is doing, and when reading them started.
    ///
    /// The charger is read on this thread with the battery rather than on the
    /// screen's own: reading it walks seven nodes of the charger's and the
    /// battery's, which take milliseconds to answer, and reading them where
    /// the page is drawn was a stall each time a charge figure moved.
    Battery(Result<Option<Battery>>, Charge, Instant),
    /// All the readings, and when taking them started.
    All(Box<Result<DeviceInfo>>, Instant),
}

pub struct Readings {
    taken: Told<Reading>,
    asks: Sender<()>,
}

impl Readings {
    /// Start taking the readings of the phone `cfg` describes, the first a
    /// second from now. The thread ends once the screen lets them go.
    pub fn start(cfg: &DeviceConfig) -> Result<Readings> {
        let (tell, taken) = worker::telling().context("make the readings' wake pipe")?;
        let (asks, asked) = mpsc::channel::<()>();
        let cfg = cfg.clone();
        thread::Builder::new()
            .name("readings".into())
            .spawn(move || {
                let mut ticks = 0u32;
                loop {
                    let at_once = match asked.recv_timeout(BATTERY_EVERY) {
                        Ok(()) => {
                            while asked.try_recv().is_ok() {}
                            true
                        }
                        Err(RecvTimeoutError::Timeout) => false,
                        Err(RecvTimeoutError::Disconnected) => return,
                    };
                    if !at_once {
                        ticks += 1;
                    }
                    let at = Instant::now();
                    let reading = if !at_once && ticks.is_multiple_of(ALL_EVERY) {
                        Reading::All(Box::new(skyward_info::collect(&cfg)), at)
                    } else {
                        let charge = crate::charge::reading();
                        Reading::Battery(skyward_info::battery_now(), charge, at)
                    };
                    if !tell.tell(reading) {
                        return;
                    }
                }
            })
            .context("start the readings thread")?;
        Ok(Readings { taken, asks })
    }

    /// Readable while readings wait.
    pub fn fd(&self) -> RawFd {
        self.taken.fd()
    }

    /// Read the battery at once.
    pub fn ask(&self) {
        self.asks.send(()).expect("the readings thread runs while its readings are taken");
    }

    /// The readings taken since the last call.
    pub fn taken(&self) -> Vec<Reading> {
        self.taken.take()
    }
}
