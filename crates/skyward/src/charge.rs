//! Reading the charger: what it delivers, the protocol it negotiated, and
//! the most it has given this charge.
//!
//! Nothing here writes. The nodes that would hold a charge limit are policy
//! votes on this phone rather than figures — `iin_limit` reads
//! `buck eff_client:input_suspend 0`, and a number written to it is refused
//! with `EINVAL` — so the recovery cannot make the phone charge faster or
//! slower. What it can do is say what the charger is doing, which is what
//! the page is for.

use std::fs;
use std::path::Path;

use skyward_ui::app::ChargeNow;

/// What the charger delivers, as the kernel tells it.
const USB: &str = "/sys/class/power_supply/usb";
/// The battery itself.
const BATTERY: &str = "/sys/class/power_supply/battery";
/// The charger's own side: the protocol it negotiated, and what it says it
/// can give.
const COMMON: &str = "/sys/class/xm_power/charger/charger_common";

/// The least voltage a reading is believed at, in microvolts. The nodes are
/// read while the charger is settling, and a reading of a few millivolts is
/// the node answering before it has anything to say rather than a real
/// measurement.
const LEAST_UV: u64 = 3_000_000;

/// Read the charger: the power going into the battery, the voltage and
/// current it comes at, the protocol negotiated, and what the charger says
/// it is rated for.
///
/// Nothing plugged in reads as None, which is what tells the page to forget
/// what the last charge gave: a charger that is present but has no figure
/// for its power yet is a gap in the reading, while an absent one is the end
/// of the charge.
///
/// The power is read from the charger's own side where the phone tells it
/// there, since that is what the charger delivers; where it says nothing,
/// the battery's own current and voltage stand in.
pub fn reading() -> Option<ChargeNow> {
    if !plugged() {
        return None;
    }
    let volts = read_u64(USB, "voltage_now").filter(|&uv| uv >= LEAST_UV);
    let amps = read_i64(USB, "current_now").map(|ua| ua.unsigned_abs()).filter(|&ua| ua != 0);
    let now = match (amps, volts) {
        (Some(ua), Some(uv)) => Some(((ua as f64 * uv as f64) / 1e11).round() as u32),
        _ => match (read_i64(BATTERY, "current_now"), read_u64(BATTERY, "voltage_now")) {
            (Some(ua), Some(uv)) => Some(((ua.unsigned_abs() as f64 * uv as f64) / 1e11).round() as u32),
            _ => None,
        },
    };
    Some(ChargeNow {
        now,
        most: now,
        protocol: read_node(COMMON, "real_type").filter(|s| !s.is_empty()),
        rated: read_u64(COMMON, "power_max").map(|w| w as u32).filter(|&w| w > 0),
        volts: volts.or_else(|| read_u64(BATTERY, "voltage_now").filter(|&uv| uv >= LEAST_UV)),
        amps,
    })
}

/// Whether a charger is plugged in, as the battery's own status says.
fn plugged() -> bool {
    matches!(read_node(BATTERY, "status").as_deref(), Some("Charging" | "Full" | "Not charging"))
        || read_u64(USB, "present").is_some_and(|p| p != 0)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_node_read_back_is_the_value_that_was_written() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("real_type");
        fs::write(&path, "PD_PPS\n").unwrap();
        assert_eq!(read_node(dir.path().to_str().unwrap(), "real_type").as_deref(), Some("PD_PPS"), "trimmed of its newline");
    }

    #[test]
    fn a_node_that_is_not_there_reads_as_nothing() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_node(dir.path().to_str().unwrap(), "absent"), None);
        assert_eq!(read_u64(dir.path().to_str().unwrap(), "absent"), None);
    }

    #[test]
    fn a_node_that_is_not_a_number_reads_as_nothing() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("iin_limit"), "buck eff_client:input_suspend 0\n").unwrap();
        assert_eq!(read_u64(dir.path().to_str().unwrap(), "iin_limit"), None, "a policy vote is not a figure");
    }
}
