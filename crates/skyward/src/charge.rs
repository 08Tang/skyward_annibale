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
    let volts = read_u64(USB, "voltage_now").filter(|&uv| uv >= LEAST_UV);
    let protocol = read_node(COMMON, "real_type").filter(|s| !s.is_empty() && s != "Unknown");
    if !present(volts, protocol.as_deref()) {
        return None;
    }
    let amps = read_i64(USB, "current_now").map(|ua| ua.unsigned_abs()).filter(|&ua| ua != 0);
    // The charger's own current is often left at zero — it reports what the
    // port negotiated, not what is flowing — so the battery's own current is
    // what the power is worked out from, and the charger's side only stands
    // in where the battery says nothing. The battery's current is negative
    // while it charges, so the sign is dropped.
    let now = match (read_i64(BATTERY, "current_now"), read_u64(BATTERY, "voltage_now")) {
        (Some(ua), Some(uv)) if ua != 0 => Some(((ua.unsigned_abs() as f64 * uv as f64) / 1e11).round() as u32),
        _ => match (amps, volts) {
            (Some(ua), Some(uv)) => Some(((ua as f64 * uv as f64) / 1e11).round() as u32),
            _ => None,
        },
    };
    Some(ChargeNow {
        now,
        most: now,
        protocol,
        rated: read_u64(COMMON, "power_max").map(|w| w as u32).filter(|&w| w > 0),
        volts: volts.or_else(|| read_u64(BATTERY, "voltage_now").filter(|&uv| uv >= LEAST_UV)),
        amps,
    })
}

/// Whether a charger is attached, as the phone's own signals say.
///
/// The battery's `status` is the obvious thing to ask and is the one thing
/// that cannot be trusted here. This phone's battery driver probes before
/// the charger registers its power supply — the kernel log has
/// `business_battery_psy_init: get usb psy fail` at 0.99 s against
/// `business_charger_usb_psy_init: usb psy success` at 1.45 s — so it never
/// takes the reference it needs, `usb_online` answers `-1` for the whole
/// boot, and `status` reads `Discharging` with a charger on. Asking it would
/// leave the page blank on a phone that is charging perfectly well.
///
/// What is left is what the charger itself reports: a voltage on its own
/// side above the level a dead cable would show, a protocol it has
/// negotiated, or its own word that something is there. A charger that is
/// attached but has not settled yet reads as absent for a second, and the
/// next reading picks it up.
fn present(volts: Option<u64>, protocol: Option<&str>) -> bool {
    volts.is_some() || protocol.is_some() || read_u64(USB, "present").is_some_and(|p| p != 0)
}

fn read_node(dir: &str, name: &str) -> Option<String> {
    fs::read_to_string(Path::new(dir).join(name)).ok().map(|s| s.trim().to_owned())
}

/// Say what every node the page reads answers, once, so a phone whose page
/// comes up empty says which node went quiet rather than leaving it to be
/// guessed at.
pub fn describe() {
    let said = |dir: &str, name: &str| read_node(dir, name).unwrap_or_else(|| "?".to_owned());
    let line = format!(
        "charging: usb {} {} {} {} {}; battery {} {} {}; common {} {}",
        said(USB, "voltage_now"),
        said(USB, "current_now"),
        said(USB, "present"),
        said(USB, "online"),
        said(USB, "type"),
        said(BATTERY, "voltage_now"),
        said(BATTERY, "current_now"),
        said(BATTERY, "status"),
        said(COMMON, "real_type"),
        said(COMMON, "power_max"),
    );
    // What the nodes say is worth having even where the log will not take it,
    // so a failure to write is dropped rather than stopping the page.
    let _ = skyward_sys::kmsg(&line);
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

    #[test]
    fn a_charger_is_there_when_its_own_side_shows_a_voltage() {
        assert!(present(Some(16_905_000), None), "a voltage on the charger's side is a charger");
    }

    #[test]
    fn a_charger_that_has_negotiated_a_protocol_is_there_even_with_no_voltage_yet() {
        assert!(present(None, Some("PD_PPS")), "a protocol named is a charger talking");
    }

    #[test]
    fn a_phone_with_nothing_attached_reads_as_no_charger() {
        assert!(!present(None, None), "no voltage and no protocol: nothing is there");
    }
}
