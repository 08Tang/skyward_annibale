//! The charge page: how fast the phone takes power from its charger.
//!
//! Three ways to charge. The default leaves the charger to do as it will,
//! which is what the recovery has always done: the kernel negotiates with
//! the charger and the phone takes what it is given. Smart keeps the power
//! under a ceiling of its own that falls as the battery warms, so a phone
//! left charging overnight does not sit hot all night. Custom takes the
//! power asked for and, while its guard is on, drops to a trickle above one
//! temperature and stops charging above another.
//!
//! The power asked for is a ceiling, never a demand: the charger and the
//! cable still decide what is actually delivered, and a ceiling above what
//! they can give changes nothing. A phone whose charger negotiates no more
//! than five volts charges no faster for a ceiling of a hundred watts.

use super::*;

/// The least and most power custom charging may ask for, in watts. The most
/// is what the phone is rated for; the least is the smallest step that still
/// charges faster than the trickle the guard falls back to.
pub const CHARGE_WATTS_LEAST: f32 = 1.0;
pub const CHARGE_WATTS_MOST: f32 = 100.0;

/// The warmth the guard's two lines may be set between, in tenths of a
/// degree: below the first a battery charges as it likes, and above the last
/// a battery is being hurt. `CHARGE_GAP_TENTHS` is the least room left
/// between them, so the trickle and the stop are never the same warmth.
pub const CHARGE_SLOW_LEAST: i32 = 300;
pub const CHARGE_STOP_MOST: i32 = 480;
pub const CHARGE_GAP_TENTHS: i32 = 10;

/// How the phone should take power from its charger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChargeMode {
    /// As the charger gives it, which is what the recovery has always done.
    #[default]
    Default,
    /// A ceiling that falls as the battery warms.
    Smart,
    /// The power asked for, under a guard of the user's own.
    Custom,
}

/// The ceiling smart charging keeps under, at each step of the battery's
/// warmth: the power in watts, and the temperature in tenths of a degree
/// above which that ceiling holds.
///
/// The steps are what a phone charging overnight wants: full power while the
/// battery is cool, then less as it warms, and least once it is warm enough
/// that the heat of charging would keep it there. The last step is not
/// nothing: a phone that stops charging entirely over one night is a phone
/// that is flat in the morning.
const SMART_STEPS: [(f32, i32); 4] = [(100.0, 350), (50.0, 380), (30.0, 400), (15.0, 430)];

impl ChargeMode {
    /// The ceiling smart charging keeps under at `tenths` of a degree.
    pub fn smart_watts(tenths: i32) -> f32 {
        for (watts, up_to) in SMART_STEPS {
            if tenths < up_to {
                return watts;
            }
        }
        SMART_STEPS[SMART_STEPS.len() - 1].0
    }
}

/// What the charge page holds, and what the runtime has read back from the
/// phone about it.
#[derive(Clone, Debug, PartialEq)]
pub struct Charging {
    pub mode: ChargeMode,
    /// The ceiling custom charging keeps under, in watts.
    pub watts: f32,
    /// Whether the guard watches the battery's warmth.
    pub guarded: bool,
    /// The warmth, in tenths of a degree, above which the guard drops to
    /// [`Charging::slow_watts`].
    pub slow_at: i32,
    /// The warmth, in tenths of a degree, above which the guard stops
    /// charging altogether.
    pub stop_at: i32,
    /// What the runtime last made of the phone's charger: what it read from
    /// the charge nodes, and what it wrote back.
    pub state: ChargeNow,
}

/// What the runtime reads from the phone's charger, and what it last did
/// about it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChargeNow {
    /// The power the charger is delivering now, in tenths of a watt, as the
    /// runtime last read it. None while nothing is plugged in, or while the
    /// phone cannot say.
    pub now: Option<u32>,
    /// The most the charger has been seen to give, in tenths of a watt: what
    /// the page shows as the ceiling a custom setting is held to.
    pub most: Option<u32>,
    /// What the guard last did, so the page can say why charging is slow.
    pub guard: Guard,
    /// The charge nodes the runtime could not write, if any, and why.
    pub trouble: Option<String>,
}

/// What the guard is doing to the charge right now.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Guard {
    /// Not holding anything back.
    #[default]
    Clear,
    /// Over the slow threshold: charging at a trickle.
    Slowed,
    /// Over the stop threshold: not charging at all.
    Stopped,
}

impl Default for Charging {
    fn default() -> Self {
        // A hundred watts, which is what the phone is rated for, and a guard
        // that leaves a margin below the temperatures a battery is hurt at.
        Self { mode: ChargeMode::Default, watts: 100.0, guarded: true, slow_at: 420, stop_at: 450, state: ChargeNow::default() }
    }
}

impl Charging {
    /// The ceiling this setting keeps under, in watts, or None where the
    /// charger is left to do as it will.
    pub fn ceiling(&self, tenths: Option<i32>) -> Option<f32> {
        match self.mode {
            ChargeMode::Default => None,
            ChargeMode::Smart => Some(ChargeMode::smart_watts(tenths.unwrap_or(0))),
            ChargeMode::Custom => Some(self.watts),
        }
    }

    /// What the guard should be doing at `tenths` of a degree, given this
    /// setting. A setting whose guard is off holds nothing back, and neither
    /// does one that leaves the charger alone.
    pub fn guard_at(&self, tenths: Option<i32>) -> Guard {
        if !self.guarded || self.mode == ChargeMode::Default {
            return Guard::Clear;
        }
        let Some(tenths) = tenths else { return Guard::Clear };
        if tenths >= self.stop_at {
            Guard::Stopped
        } else if tenths >= self.slow_at {
            Guard::Slowed
        } else {
            Guard::Clear
        }
    }

    /// The power to charge at, in watts, with the guard's own limits applied.
    pub fn wanted(&self, tenths: Option<i32>) -> Option<f32> {
        let ceiling = self.ceiling(tenths)?;
        Some(match self.guard_at(tenths) {
            Guard::Clear => ceiling,
            // A trickle, not a stop: enough to hold the level it reached.
            Guard::Slowed => ceiling.min(5.0),
            Guard::Stopped => 0.0,
        })
    }
}

impl App {
    /// Take in what the runtime read from the charger.
    pub fn set_charge_state(&mut self, state: ChargeNow) {
        if self.state.charge.state != state {
            self.state.charge.state = state;
            self.refresh();
        }
    }

    /// Set how the phone should charge, and ask the runtime to put it in
    /// place.
    pub fn set_charge(&mut self, charge: Charging) {
        if self.state.charge != charge {
            self.state.charge = charge;
            self.refresh();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_leaves_the_charger_alone() {
        let charge = Charging::default();
        assert_eq!(charge.ceiling(Some(500)), None, "no ceiling");
        assert_eq!(charge.wanted(Some(500)), None, "and nothing asked for");
        assert_eq!(charge.guard_at(Some(600)), Guard::Clear, "and nothing held back, however warm");
    }

    #[test]
    fn smart_falls_as_the_battery_warms() {
        let charge = Charging { mode: ChargeMode::Smart, ..Charging::default() };
        assert_eq!(charge.ceiling(Some(300)), Some(100.0), "cool: the whole of it");
        assert_eq!(charge.ceiling(Some(360)), Some(50.0), "warming: half");
        assert_eq!(charge.ceiling(Some(390)), Some(30.0), "warmer: less");
        assert_eq!(charge.ceiling(Some(440)), Some(15.0), "warm: least, but not nothing");
    }

    #[test]
    fn custom_asks_for_what_it_is_set_to_until_the_guard_speaks() {
        let charge = Charging { mode: ChargeMode::Custom, watts: 60.0, slow_at: 400, stop_at: 450, ..Charging::default() };
        assert_eq!(charge.wanted(Some(300)), Some(60.0), "cool: what it asked for");
        assert_eq!(charge.guard_at(Some(399)), Guard::Clear);
        assert_eq!(charge.wanted(Some(400)), Some(5.0), "over the slow line: a trickle");
        assert_eq!(charge.wanted(Some(450)), Some(0.0), "over the stop line: nothing");
    }

    #[test]
    fn a_guard_turned_off_holds_nothing_back() {
        let charge = Charging { mode: ChargeMode::Custom, watts: 60.0, guarded: false, slow_at: 300, stop_at: 310, ..Charging::default() };
        assert_eq!(charge.guard_at(Some(900)), Guard::Clear, "however warm it gets");
        assert_eq!(charge.wanted(Some(900)), Some(60.0));
    }

    #[test]
    fn a_phone_that_cannot_say_how_warm_it_is_is_charged_as_asked() {
        let charge = Charging { mode: ChargeMode::Custom, watts: 60.0, ..Charging::default() };
        assert_eq!(charge.guard_at(None), Guard::Clear, "no warmth read: nothing held back");
        assert_eq!(charge.wanted(None), Some(60.0));
    }
}
