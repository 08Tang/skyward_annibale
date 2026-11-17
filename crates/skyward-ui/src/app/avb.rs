//! The AVB page's state and the answers the runtime gives it.

use super::*;

/// Which AVB check the page asks to switch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvbCheck {
    Verity,
    Verification,
}

/// What the runtime read from the top-level vbmeta image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvbInfo {
    pub verity: bool,
    pub verification: bool,
    pub rollback_index: u64,
    pub unlocked: Option<bool>,
    pub backup: bool,
}

impl App {
    pub fn set_avb(&mut self, answer: Result<AvbInfo, String>) {
        self.state.avb = Some(answer);
        if self.screen() == Screen::Avb {
            self.refresh();
        }
    }

    pub fn avb_ended(&mut self, notice: String, now: u64) {
        self.state.avb_notice = Some(notice);
        self.wake(now);
        self.refresh();
    }

    /// Start changing an AVB check, so the page cannot ask again while it
    /// is being written.
    pub fn avb_started(&mut self, check: AvbCheck) {
        self.state.avb_notice = Some(
            self.state
                .lang
                .pick(match check {
                    AvbCheck::Verity => [
                        "正在修改 dm-verity…",
                        "正在修改 dm-verity…",
                        "Changing dm-verity…",
                    ],
                    AvbCheck::Verification => [
                        "正在修改启动验证…",
                        "正在修改啟動驗證…",
                        "Changing boot verification…",
                    ],
                })
                .into(),
        );
        self.refresh();
    }
}
