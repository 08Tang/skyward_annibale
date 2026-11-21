//! Charging: how much power the phone asks of its charger, and the guard
//! that keeps the battery from getting hot over it.
//!
//! The recovery has always let the charger do as it will: the kernel
//! negotiates with the charger and the phone takes what it is given. That is
//! the default, and it is the one setting here that writes nothing at all.
//!
//! The others ask for a ceiling. On Xiaomi's Qualcomm phones the ceiling is
//! the input current the charger interface allows, in microamps, which the
//! power asked for is divided by the voltage the charger is delivering to
//! find. The ceiling is written to `iin_limit`; the guard's slow line goes to
//! the wired charger's thermal limit, and its stop line suspends the input
//! altogether. Nothing here can make a charger give more than it has: a
//! ceiling above what the cable and the charger deliver changes nothing, and
//! the phone charges no faster for it.
//!
//! Every write is put back when the setting is left, so a phone that leaves
//! the recovery is not left charging under a limit this page set.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use skyward_ui::app::{ChargeMode, ChargeNow, ChargeSetting, Guard};

/// Where the charger interface's nodes are, as Xiaomi's phones carry them.
const INTERFACE: &str = "/sys/class/xm_power/charger/charge_interface";
/// The wired charger's thermal limit, and its siblings.
const THERMAL: &str = "/sys/class/xm_power/charger/charger_thermal";
/// What the charger delivers, as the kernel tells it.
const USB: &str = "/sys/class/power_supply/usb";
/// The battery itself.
const BATTERY: &str = "/sys/class/power_supply/battery";

/// The voltage to divide by where the charger will not say what it delivers:
/// the five volts a charger gives before it is asked for more.
const FALLBACK_UV: u64 = 5_000_000;

/// The most input current to ask for, in microamps. The interface refuses a
/// value above what its charger can carry, and the refusal is not passed on
/// as a smaller limit but as an error, so the ceiling is held to a figure
/// every wired charger of this phone's generation takes: five amps, which is
/// what a hundred watts at twenty volts comes to. A limit above that is
/// refused outright, and the phone goes on charging exactly as it did.
const MOST_UA: u64 = 5_000_000;

/// The least voltage a reading is believed at, in microvolts. The nodes are
/// read while the charger is settling, and a reading of a few millivolts is
/// the node answering before it has anything to say rather than a real
/// measurement; dividing by it would ask for an absurd current.
const LEAST_UV: u64 = 3_000_000;

/// The charge nodes as they were found, so what was written can be put back.
#[derive(Clone, Debug, Default)]
pub struct Undo {
    /// The input current limit as it was, if it was written.
    pub iin: Option<String>,
    /// The wired thermal limit as it was, if it was written.
    pub wired: Option<String>,
    /// The input suspend as it was, if it was written.
    pub suspend: Option<String>,
}

/// What the charger is doing now, as the nodes tell it.
#[derive(Clone, Debug, Default)]
pub struct Reading {
    /// The power going into the battery, in tenths of a watt.
    pub watts: Option<u32>,
    /// The voltage the charger delivers, in microvolts.
    pub voltage: Option<u64>,
}

/// Read the charger: the power the charger is delivering, and the voltage it
/// comes at.
///
/// The power is read from the charger's own side where the phone tells it
/// there, since that is what a ceiling is judged against: the battery's own
/// current is what reaches it after the charger's losses, and a ceiling set
/// against that would be set too low. Where the charger says nothing, the
/// battery's current and voltage stand in.
pub fn reading() -> Reading {
    let voltage = read_u64(USB, "voltage_now").filter(|&uv| uv >= LEAST_UV).or_else(|| read_u64(BATTERY, "voltage_now").filter(|&uv| uv >= LEAST_UV));
    let watts = match (read_i64(USB, "current_now"), voltage) {
        (Some(ua), Some(uv)) if ua != 0 => Some(((ua.unsigned_abs() as f64 * uv as f64) / 1e11).round() as u32),
        _ => match (read_i64(BATTERY, "current_now"), read_u64(BATTERY, "voltage_now")) {
            (Some(ua), Some(uv)) => Some(((ua.unsigned_abs() as f64 * uv as f64) / 1e11).round() as u32),
            _ => None,
        },
    };
    Reading { watts, voltage }
}

/// The input current limit to ask for `watts` of power at `uv` microvolts, in
/// microamps: what the ceiling comes to, held to what the interface takes.
fn current_for(watts: f32, uv: u64) -> u64 {
    let uv = if uv < LEAST_UV { FALLBACK_UV } else { uv };
    let ua = (watts as f64 * 1e6 / (uv as f64 / 1e6)) as u64;
    ua.min(MOST_UA)
}

/// Put `charge` in place, giving back what the nodes held before, to be put
/// back when the setting is left.
pub fn set(charge: &ChargeSetting, tenths: Option<i32>) -> Result<(Undo, ChargeNow)> {
    let mut undo = Undo::default();
    let mut state = ChargeNow::default();
    let wanted = charge.wanted(tenths);

    let Some(watts) = wanted else {
        // The default asks for nothing and writes nothing: the caller gives
        // the nodes back what they held, from the undo it kept.
        return Ok((undo, state));
    };

    let reading = reading();
    state.now = reading.watts;
    state.most = reading.watts;

    // A stop is the input suspended rather than a limit of zero, which the
    // interface reads as "no limit" on some chargers.
    if charge.guard_at(tenths) == Guard::Stopped {
        undo.suspend = write_node(INTERFACE, "input_suspend", "1").ok();
        state.guard = Guard::Stopped;
        return Ok((undo, state));
    }

    // A ceiling of zero with the guard clear cannot happen, since the guard
    // stops before the ceiling reaches nothing.
    if watts > 0.0 {
        let uv = reading.voltage.unwrap_or(FALLBACK_UV);
        let ua = current_for(watts, uv);
        // The node is written only where the figure differs from what it
        // already holds: the runtime reads the charger once a second, and a
        // node written every second is a node the charger driver has to act
        // on every second.
        let held = read_node(INTERFACE, "iin_limit");
        let already = held.as_deref().and_then(|s| s.parse::<u64>().ok()) == Some(ua);
        if !already {
            undo.iin = write_node(INTERFACE, "iin_limit", &ua.to_string()).ok();
            if undo.iin.is_none() {
                let why = match write_node(INTERFACE, "iin_limit", &ua.to_string()) {
                    Ok(_) => "the second write went through".to_owned(),
                    Err(e) => format!("{e:#}"),
                };
                let _ = skyward_sys::kmsg(&format!("charging: iin_limit would not take {ua} uA: {why}"));
            }
        }
        // The thermal limit is the slow line: above it the charger gives
        // less, which is what the guard's trickle asks for.
        if charge.guarded {
            let celsius = charge.slow_at / 10;
            let held = read_node(THERMAL, "wired_ctrl_limit");
            if held.as_deref().and_then(|s| s.parse::<i32>().ok()) != Some(celsius) {
                undo.wired = write_node(THERMAL, "wired_ctrl_limit", &celsius.to_string()).ok();
            }
        }
    }
    state.guard = charge.guard_at(tenths);
    Ok((undo, state))
}

/// Give the charge nodes back what they held, and suspend nothing.
pub fn restore(undo: &Undo) {
    if let Some(v) = &undo.suspend {
        let _ = write_node(INTERFACE, "input_suspend", v);
    } else {
        let _ = write_node(INTERFACE, "input_suspend", "0");
    }
    if let Some(v) = &undo.iin {
        let _ = write_node(INTERFACE, "iin_limit", v);
    }
    if let Some(v) = &undo.wired {
        let _ = write_node(THERMAL, "wired_ctrl_limit", v);
    }
}

/// Write `value` to the node `name` under `dir`, giving back what it held.
fn write_node(dir: &str, name: &str, value: &str) -> Result<String> {
    let path = Path::new(dir).join(name);
    let was = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    fs::write(&path, format!("{value}\n")).with_context(|| format!("write {}", path.display()))?;
    Ok(was.trim().to_owned())
}

fn read_node(dir: &str, name: &str) -> Option<String> {
    fs::read_to_string(Path::new(dir).join(name)).ok().map(|s| s.trim().to_owned())
}

fn read_i64(dir: &str, name: &str) -> Option<i64> {
    read_node(dir, name)?.parse().ok()
}

fn read_u64(dir: &str, name: &str) -> Option<u64> {
    read_node(dir, name)?.parse().ok()
}

/// Whether the phone carries the charge nodes this page writes to: a phone
/// without them can still be charged, but not at a ceiling of its own.
pub fn supported() -> bool {
    Path::new(INTERFACE).is_dir() && Path::new(INTERFACE).join("iin_limit").exists()
}

/// Say in the log what the charge nodes hold and what they will take, once,
/// when the page first writes. A phone whose nodes refuse a figure the page
/// offers needs its own figures known before the page can offer the right
/// ones, and the log is where they are found.
pub fn describe() {
    let held = |dir: &str, name: &str| match read_node(dir, name) {
        Some(v) => format!("{name} = {v}"),
        None => format!("{name} = (none)"),
    };
    let said = format!(
        "charging: {}; {}; {}; {}; {}; usb {:?} uV {:?} uA; battery {:?} uV {:?} uA",
        held(INTERFACE, "iin_limit"),
        held(INTERFACE, "input_suspend"),
        held(INTERFACE, "charge_enable"),
        held(THERMAL, "wired_ctrl_limit"),
        held("/sys/class/xm_power/charger/quick_charge", "current_ration"),
        read_u64(USB, "voltage_now"),
        read_i64(USB, "current_now"),
        read_u64(BATTERY, "voltage_now"),
        read_i64(BATTERY, "current_now"),
    );
    let _ = skyward_sys::kmsg(&said);
}

/// Whether the phone carries the charge nodes this page writes to.
pub fn trouble(charge: &ChargeSetting) -> bool {
    charge.mode != ChargeMode::Default && !supported()
}

/// Where a node lives, for the tests to read back what was written.
#[cfg(test)]
fn node(dir: &str, name: &str) -> std::path::PathBuf {
    Path::new(dir).join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_asked_for_is_the_power_over_the_voltage() {
        assert_eq!(current_for(10.0, 5_000_000), 2_000_000, "ten watts at five volts is two amps");
        assert_eq!(current_for(18.0, 9_000_000), 2_000_000, "and eighteen at nine");
        assert_eq!(current_for(100.0, 20_000_000), 5_000_000, "a hundred at twenty is five");
    }

    #[test]
    fn a_charger_that_will_not_say_is_taken_as_five_volts() {
        assert_eq!(current_for(10.0, 0), 2_000_000);
    }

    #[test]
    fn the_current_asked_for_is_held_to_what_an_interface_takes() {
        assert_eq!(current_for(100.0, 5_000_000), MOST_UA, "a hundred at five is more than any phone takes");
    }

    #[test]
    fn a_guard_that_stops_suspends_the_input_rather_than_asking_for_nothing() {
        let charge = ChargeSetting { mode: ChargeMode::Custom, watts: 50.0, slow_at: 400, stop_at: 440, ..ChargeSetting::default() };
        assert_eq!(charge.guard_at(Some(450)), Guard::Stopped, "over the stop line");
        assert_eq!(charge.wanted(Some(450)), Some(0.0), "and nothing asked for");
    }

    #[test]
    fn a_node_read_back_is_the_value_that_was_written() {
        let dir = tempfile::tempdir().unwrap();
        let path = node(dir.path().to_str().unwrap(), "iin_limit");
        fs::write(&path, "500000\n").unwrap();
        let was = write_node(dir.path().to_str().unwrap(), "iin_limit", "2000000").unwrap();
        assert_eq!(was, "500000", "what it held");
        assert_eq!(read_node(dir.path().to_str().unwrap(), "iin_limit").unwrap(), "2000000", "and what is there now");
    }
}
