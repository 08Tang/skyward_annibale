//! Reading the charger, and the one thing that can be told to it: that the
//! battery passed its check.
//!
//! Nothing here sets a charge limit. The nodes that would hold one are policy
//! votes on this phone rather than figures — `iin_limit` reads
//! `buck eff_client:input_suspend 0`, and a number written to it is refused
//! with `EINVAL` — so the recovery cannot make the phone charge slower, and
//! the page says what the charger is doing rather than pretending to steer it.
//!
//! What can be written is the battery's own word about itself. The kernel asks
//! the fuel gauge whether the battery passed its check before it lets a charge
//! draw more than 2 A, and on this phone that answer never comes in recovery:
//! the charging service needs the charger's own chip over `/dev/hidraw`, and no
//! such device is ever enumerated here. So the kernel caps the current at
//! `ichg:[batt_auth][2000]`, the protocol never reaches PD PPS, and the charge
//! sits at about 8 W where the stock system takes it to 40 W and beyond.
//!
//! The two nodes below are that answer. Writing both makes the fuel-gauge
//! driver itself raise the event the kernel is waiting for — the store
//! function for `slave_authentic` checks `authentic`, checks `slave_authentic`,
//! checks that the charger's power matches what the fuel gauge expects, and
//! then posts event 40, which is the `batt_auth pass` the log shows on a stock
//! boot. Nothing else is forged: the driver does the posting.
//!
//! It is a risk taken on, and the page says so. The recovery's own thermal
//! chain is broken — `mca_thermal: init wire voter failed`, so
//! `mca_charger_thermal_handle_limit` answers `charger is not online` and never
//! backs the charge off — which is why the page watches the battery's own
//! temperature and drops the charge back itself when it gets too hot.

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
/// The fuel gauge's own side: whether the battery passed its check, and the
/// temperature the page watches while it is told that it did.
const GAUGE: &str = "/sys/class/xm_power/fuelgauge/strategy_fg";

/// The temperature the charge is dropped back at, in tenths of a degree: 45 °C.
///
/// The stock system backs a wired charge off well before this — its own
/// thermal chain steps the current down through `wired_ctrl_limit` — but that
/// chain does not work here, so this is the only backstop there is. It is set
/// where a phone is warm in the hand but not hot, since the point is to stay
/// clear of the temperatures that age a battery rather than to see how close
/// the limit can be walked.
pub const TOO_HOT: i32 = 450;

/// The temperature the charge is allowed to start again at, in tenths of a
/// degree: 40 °C. Below `TOO_HOT` by enough that a battery sitting at the
/// threshold cannot switch the charge on and off every second.
pub const COOL_ENOUGH: i32 = 400;

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

/// Whether the phone carries the nodes that can tell the kernel the battery
/// passed its check. A phone without them is not offered the switch, since
/// there would be nothing for it to do.
pub fn offered() -> bool {
    Path::new(GAUGE).join("authentic").exists() && Path::new(GAUGE).join("slave_authentic").exists()
}

/// Whether the battery is being told it passed its check.
pub fn fast() -> bool {
    read_u64(GAUGE, "authentic").is_some_and(|v| v != 0)
}

/// The battery's temperature, in tenths of a degree.
pub fn temperature() -> Option<i32> {
    read_i64(BATTERY, "temp").or_else(|| read_i64(GAUGE, "temp"))
}

/// Tell the kernel the battery passed its check, so that the charge may draw
/// more than 2 A.
///
/// `authentic` is written first and `slave_authentic` second, and the order
/// matters: the fuel-gauge driver does its checking in the store function for
/// `slave_authentic`, and it will not post the event unless `authentic` is
/// already set. Writing them the other way round does nothing at all.
pub fn allow_fast() -> Result<(), String> {
    write_node(GAUGE, "authentic", "1")?;
    write_node(GAUGE, "slave_authentic", "1")?;
    let _ = skyward_sys::kmsg(&format!(
        "charging: told the gauge the battery passed, and it now reads authentic {}, slave {}",
        read_node(GAUGE, "authentic").unwrap_or_else(|| "?".into()),
        read_node(GAUGE, "slave_authentic").unwrap_or_else(|| "?".into()),
    ));
    Ok(())
}

/// Take the word back, so that the charge is held at what the kernel allows a
/// battery it has not checked.
///
/// The switch on the page cannot be turned off once it is on, and this is not
/// reachable from the page at all: it is here for the temperature watch, which
/// is the one thing that should be able to drop the charge back.
pub fn stop_fast() -> Result<(), String> {
    write_node(GAUGE, "slave_authentic", "0")?;
    write_node(GAUGE, "authentic", "0")?;
    Ok(())
}

fn write_node(dir: &str, name: &str, value: &str) -> Result<(), String> {
    let path = Path::new(dir).join(name);
    fs::write(&path, value).map_err(|e| format!("{}: {e}", path.display()))
}

/// Whether a battery at this temperature is hot enough that the charge should
/// be dropped back. A battery that says nothing is not called hot: the reading
/// is missing rather than the battery being cold, and dropping a charge on a
/// missing reading would make the switch turn itself off for no reason.
pub fn too_hot(temp: Option<i32>) -> bool {
    temp.is_some_and(|t| t >= TOO_HOT)
}

/// Whether a battery that was dropped back for heat is cool enough to be
/// allowed to charge quickly again.
pub fn cool_enough(temp: Option<i32>) -> bool {
    temp.is_some_and(|t| t <= COOL_ENOUGH)
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

    #[test]
    fn a_battery_at_the_limit_is_too_hot_but_one_just_under_it_is_not() {
        assert!(too_hot(Some(TOO_HOT)), "at the limit is hot");
        assert!(too_hot(Some(TOO_HOT + 1)), "over the limit is hot");
        assert!(!too_hot(Some(TOO_HOT - 1)), "under the limit is not");
        assert!(!too_hot(None), "a battery that says nothing is not called hot");
    }

    #[test]
    fn a_charge_that_was_dropped_stays_dropped_until_the_battery_is_well_under_the_limit() {
        assert!(cool_enough(Some(COOL_ENOUGH)), "at the lower mark it may start again");
        assert!(!cool_enough(Some(COOL_ENOUGH + 1)), "one step over the lower mark is still too warm");
        assert!(cool_enough(Some(TOO_HOT - 1)), "between the marks the charge stays off");
    }
}
