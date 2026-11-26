use crate::app::{
    ADDABLE, Action, AvbCheck, BACKUP_PARTS, BackupAt, BackupEnd, BackupJob, Check, ChargeMode, ClashChoice, Confirm, DATA_ERASED, DATA_PARTITION, DOWNLOADS, Damage, Data, Dock,
    DownloadEnd,
    EraseStop, FASTBOOT_CHOICES, Fetch, Field, FileKind, FileOp, FileTask, Fold, Folder, Frame, FsAction, FsChoice, FsRow, Have, HeroArt, Hit, ImageEnd, ImageFlow, ImageJob, Install, JobEnd, Keys, PackageKind, Pick, Pinned, Place, Region, Rows,
    RecordSetting, SCREEN_OFF, Screen, SearchScope, Serving, SkywardCheck, SkywardEnd, SkywardJob, SkywardStage, Slider, SortKey, Source, State, Switch, SystemsWork, TARGETS, THERMAL_LIMIT_LEAST, THERMAL_LIMIT_MOST, TIME_ZONES, Target, Report,
    UsbDrive, UsbLink, WIPES, Wipe, installs, starting, MEDIA_PART, UTC, listed, DATA_PART,
    AutoRoot, KernelSuEnd, KernelSuJob, KernelSuStage, is_newer,};
use crate::canvas::Canvas;
use crate::color::Rgba;
use crate::editor;
use crate::files;
use crate::fmt::{self, Verb};
use crate::font::Fonts;
use crate::geom::Rect;
use crate::icons::{self, Icon};
use crate::lang::{Lang, Text, same};
use crate::search;
use crate::term::{self, Term};
use crate::texts;
use crate::theme::Theme;
use crate::tr;
use crate::widgets::{self, Caption, Detail, Fact, Outline, Row, Spec, SystemCard, SystemMark, Tone, Trail};
use skyward_info::{DeviceInfo, RUNNING};
use skyward_install::{Outcome, Place as FlashPlace, Reflash, Refusal, Stage};
use skyward_net::download::Progress;
use skyward_net::supplicant::Security;
use skyward_net::wlan::{Bands, Network, Power, Stage as Joining};
use skyward_systems::{Held, Who};
use skyward_vab::{Slot, Standing};
use std::path::Path;

// The colours the stock Settings fills its icons' tiles with, the same in
// light and dark mode.
const BLUE: Rgba = Rgba::rgb(0x34, 0x82, 0xff);
const GREEN: Rgba = Rgba::rgb(0x1d, 0xcd, 0x3a);
const PURPLE: Rgba = Rgba::rgb(0x77, 0x67, 0xf9);
const SLATE: Rgba = Rgba::rgb(0x8c, 0x9d, 0xb0);
const ORANGE: Rgba = Rgba::rgb(0xff, 0x69, 0x00);
const RED: Rgba = Rgba::rgb(0xfa, 0x38, 0x2e);

/// What every page draws from: style, typefaces, the device readings, and
/// what the pages hold, from the choices made on them to where the data
/// partition stands.
pub struct Ctx<'a> {
    pub t: &'a Theme,
    pub f: &'a Fonts,
    pub info: &'a DeviceInfo,
    pub state: &'a State,
    /// The shell the terminal page shows, once one has started.
    pub term: Option<&'a Term>,
    /// The part of the page to draw; all of it if none is given. The page
    /// is laid out whole either way.
    pub band: Option<Band>,
}

/// Rows `rows` of a page, and of them the columns `cols`, each from the
/// first to one past the last.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Band {
    pub rows: (u32, u32),
    pub cols: (u32, u32),
}

pub fn render(screen: Screen, cx: &Ctx) -> Frame {
    match screen {
        Screen::Decrypt => decrypt(cx),
        Screen::Home => home(cx),
        Screen::Settings => settings(cx),
        Screen::Reboot => reboot(cx),
        Screen::Wipe => wipe(cx),
        Screen::Mount => mount(cx),
        Screen::Install => install(cx),
        Screen::Choose(place) => choose(cx, place),
        Screen::Installing => installing(cx),
        Screen::About => about(cx),
        Screen::Specs => specs(cx),
        Screen::ScreenOff => screen_off(cx),
        Screen::Search => search(cx),
        Screen::Backup => backup(cx),
        Screen::Restore => restore(cx),
        Screen::RestoreBackup => restore_backup(cx),
        Screen::BackupJob => backup_job(cx),
        Screen::Advanced => advanced(cx),
        Screen::Modules => modules(cx),
        Screen::ModuleInfo => module_info(cx),
        Screen::Root => root(cx),
        Screen::KernelSu => kernel_su(cx),
        Screen::KernelSuKmi => kernel_su_kmi(cx),
        Screen::Magisk => magisk(cx),
        Screen::DataApps => data_apps(cx),
        Screen::SystemApps => system_apps(cx),
        Screen::DataAppInfo => data_app_info(cx),
        Screen::Files => files(cx),
        Screen::FileInfo => file_info(cx),
        Screen::Permissions => permissions(cx),
        Screen::Editor => editor(cx),
        Screen::Terminal => terminal(cx),
        Screen::Relabel => relabel(cx),
        Screen::Filesystems => filesystems(cx),
        Screen::Partitions => partitions(cx),
        Screen::ImageBackup => image_backup(cx),
        Screen::PowerOff => power_off(cx),
        Screen::Tasks => tasks(cx),
    Screen::Charge => charge(cx),
        Screen::Filesystem => filesystem(cx),
        Screen::PartWork => part_work(cx),
        Screen::Language => language(cx),
        Screen::Record => record(cx),
        Screen::TimeZone => time_zone(cx),
        Screen::TimeZones(region) => time_zones(cx, region),
        Screen::Logs => logs(cx),
        Screen::Update => update(cx),
        Screen::Serve => serve(cx),
        Screen::Fetch => fetch(cx),
        Screen::Avb => avb(cx),
        Screen::SkywardUpdate => skyward_update(cx),
        Screen::Feedback => feedback(cx),
        Screen::License => license(cx),
        Screen::FontLicense => font_license(cx),
        Screen::Slot => slot(cx),
        Screen::Systems => systems(cx),
        Screen::Fastboot => fastboot(cx),
        Screen::Erase => erase(cx),
        Screen::CantStart => cant_start(cx),
        Screen::Unhandled => unhandled(cx),
        Screen::Rename => rename(cx),
        Screen::Clash => clash(cx),
        Screen::FileJob => file_job(cx),
        Screen::Wlan => wlan(cx),
        Screen::WlanPassword => wlan_password(cx),
        Screen::WlanNetwork => wlan_network(cx),
        Screen::FilesOtg => files_otg(cx),
        Screen::WlanAdd => wlan_add(cx),
    }
}

/// Label and colour of each swipe-to-confirm control: the filesystem
/// page's as `s` says what it is to do.
pub fn confirm_label(which: Confirm, s: &State) -> (&'static str, bool) {
    use texts::confirm;
    const INSTALL: Text = [
        "滑动以安装",
        "滑動以安裝",
        "Slide to install",
        "Slide to install",
        "Slide to install",
        "Slide to install",
        "Slide to install",
        "Slide to install",
        "Slide to install",
        "Slide to install",
        "Slide to install",
        "Slide to install",
        "Slide to install",
        "Slide to install",
        "Slide to install",
    ];
    let (label, danger) = match which {
        // 锁屏自己画它，这张只给按住电源键时的滑块用。
        Confirm::Unlock => (&["滑动解锁", "滑動解鎖", "Slide to unlock"][..], false),
        Confirm::Filesystem => match s.fs_action {
            FsAction::Repair => (&confirm::REPAIR[..], true),
            FsAction::Remake(_) => (&confirm::CHANGE[..], true),
        },
        Confirm::Reboot => (&["滑动以重启", "滑動以重新啟動", "Slide to reboot"][..], false),
        Confirm::PowerOff => (&["滑动以关机", "滑動以關機", "Slide to power off"][..], true),
        Confirm::Wipe => (&["滑动以清除", "滑動以清除", "Slide to wipe"][..], true),
        Confirm::Flash => (&["滑动以刷入", "滑動以刷入", "Slide to flash"][..], true),
        Confirm::Backup => (&["滑动以备份", "滑動以備份", "Slide to back up"][..], false),
        Confirm::Restore => (&["滑动以恢复", "滑動以還原", "Slide to restore"][..], true),
        Confirm::Slot => (&["滑动以切换", "滑動以切換", "Slide to switch"][..], true),
        Confirm::FactoryReset => (&["滑动以恢复出厂设置", "滑動以恢復原廠設定", "Slide to factory reset"][..], true),
        Confirm::DeleteEntry | Confirm::DeleteBackup => (&["滑动以删除", "滑動以刪除", "Slide to delete"][..], true),
        Confirm::KeepLock => (&["滑动以开启", "滑動以開啟", "Slide to turn on"][..], true),
        Confirm::Install => (&INSTALL[..], false),
        Confirm::InstallWipe => (&["滑动以安装并清除", "滑動以安裝並清除", "Slide to install and wipe"][..], true),
        Confirm::RunScript => (&INSTALL[..], true),
        Confirm::Merge => (&["滑动以合并", "滑動以合併", "Slide to merge"][..], true),
        Confirm::KernelSu => (&["滑动以写入 KernelSU", "滑動以寫入 KernelSU", "Slide to write KernelSU"][..], false),
        Confirm::Magisk => (&["滑动以写入 Magisk", "滑動以寫入 Magisk", "Slide to write Magisk"][..], false),
        Confirm::Relabel => (&["滑动以修复", "滑動以修復", "Slide to fix"][..], true),
        Confirm::SwitchSystem => (&["滑动以切换并重启", "滑動以切換並重新啟動", "Slide to switch and reboot"][..], false),
        Confirm::RemoveSystem => (&["滑动以删除", "滑動以刪除", "Slide to remove"][..], true),
        Confirm::DeleteApp => (&["滑动以卸载", "滑動以卸載", "Slide to uninstall"][..], true),
        Confirm::InstallSystem => (&INSTALL[..], false),
        Confirm::ExportImage(_) => (&["滑动以备份", "滑動以備份", "Slide to back up"][..], false),
        Confirm::ImportImage(_) => (&["滑动以写回", "滑動以寫回", "Slide to write back"][..], true),
        Confirm::DeleteImage(_) => (&["滑动以删除", "滑動以刪除", "Slide to delete"][..], true),
    };
    (s.lang.pick(label), danger)
}

/// The home page's large title.
pub const HOME_TITLE: &str = "Skyward";
pub const SEARCH_HINT: Text = [
    "搜索功能与分区",
    "搜尋功能與分割區",
    "Search features and partitions",
    "Search features and partitions",
    "Search features and partitions",
    "Search features and partitions",
    "Search features and partitions",
    "Search features and partitions",
    "Search features and partitions",
    "Search features and partitions",
    "Search features and partitions",
    "Search features and partitions",
    "Search features and partitions",
    "Search features and partitions",
    "Search features and partitions",
    ];

/// Where the phone can restart to, and what is said under the name of one
/// that wants it.
const REBOOT_TARGETS: [(Text, Option<Text>); 6] = [
    (texts::reboot::target::SYSTEM, None),
    (texts::reboot::target::RECOVERY, None),
    (same("Fastboot"), Some(texts::reboot::target::FASTBOOT_NOTE)),
    (same("Bootloader"), None),
    (same("EDL"), Some(texts::reboot::target::EDL_NOTE)),
    (texts::reboot::target::POWER_OFF, None),
];

/// What pages are called both on the rows that open them and in their own
/// titles.
const WLAN: Text = [
    "无线网络",
    "無線網路",
    "WLAN",
    "WLAN",
    "WLAN",
    "WLAN",
    "WLAN",
    "WLAN",
    "WLAN",
    "WLAN",
    "WLAN",
    "WLAN",
    "WLAN",
    "WLAN",
    "WLAN",
];
/// Whether the WLAN opens as the recovery starts.
const WLAN_AUTO: Text = [
    "WiFi 自动连接",
    "WiFi 自動連線",
    "WLAN on at start",
    "WLAN on at start",
    "WLAN on at start",
    "WLAN on at start",
    "WLAN on at start",
    "WLAN on at start",
    "WLAN on at start",
    "WLAN on at start",
    "WLAN on at start",
    "WLAN on at start",
    "WLAN on at start",
    "WLAN on at start",
    "WLAN on at start",
];
/// Whether the password of the network joined last is kept with it.
const WLAN_KEEP: Text = [
    "记住 WiFi 密码",
    "記住 WiFi 密碼",
    "Keep the password",
    "Keep the password",
    "Keep the password",
    "Keep the password",
    "Keep the password",
    "Keep the password",
    "Keep the password",
    "Keep the password",
    "Keep the password",
    "Keep the password",
    "Keep the password",
    "Keep the password",
    "Keep the password",
];
const DECRYPT: Text = [
    "解密",
    "解密",
    "Decrypt",
    "Decrypt",
    "Decrypt",
    "Decrypt",
    "Decrypt",
    "Decrypt",
    "Decrypt",
    "Decrypt",
    "Decrypt",
    "Decrypt",
    "Decrypt",
    "Decrypt",
    "Decrypt",
];
const ADVANCED: Text = [
    "高级工具",
    "進階工具",
    "Advanced",
    "Advanced",
    "Advanced",
    "Advanced",
    "Advanced",
    "Advanced",
    "Advanced",
    "Advanced",
    "Advanced",
    "Advanced",
    "Advanced",
    "Advanced",
    "Advanced",
];
const SETTINGS: Text = [
    "选项设置",
    "選項設定",
    "Options",
    "Options",
    "Options",
    "Options",
    "Options",
    "Options",
    "Options",
    "Options",
    "Options",
    "Options",
    "Options",
    "Options",
    "Options",
];
const REBOOT: Text = [
    "重启系统",
    "重新啟動",
    "Reboot",
    "Reboot",
    "Reboot",
    "Reboot",
    "Reboot",
    "Reboot",
    "Reboot",
    "Reboot",
    "Reboot",
    "Reboot",
    "Reboot",
    "Reboot",
    "Reboot",
];
const LOGS: Text = [
    "导出日志",
    "匯出記錄",
    "Export logs",
    "Export logs",
    "Export logs",
    "Export logs",
    "Export logs",
    "Export logs",
    "Export logs",
    "Export logs",
    "Export logs",
    "Export logs",
    "Export logs",
    "Export logs",
    "Export logs",
];
const UPDATE: Text = [
    "系统更新",
    "系統更新",
    "System update",
    "System update",
    "System update",
    "System update",
    "System update",
    "System update",
    "System update",
    "System update",
    "System update",
    "System update",
    "System update",
    "System update",
    "System update",
];

const MODEL: Text = [
    "设备型号",
    "裝置型號",
    "Model",
    "Model",
    "Model",
    "Model",
    "Model",
    "Model",
    "Model",
    "Model",
    "Model",
    "Model",
    "Model",
    "Model",
    "Model",
];
const DATA_PART_NAME: Text = [
    "data 分区",
    "data 分割區",
    "data partition",
    "data partition",
    "data partition",
    "data partition",
    "data partition",
    "data partition",
    "data partition",
    "data partition",
    "data partition",
    "data partition",
    "data partition",
    "data partition",
    "data partition",
];
const UNREADABLE: Text = [
    "无法读取",
    "無法讀取",
    "Unreadable",
    "Unreadable",
    "Unreadable",
    "Unreadable",
    "Unreadable",
    "Unreadable",
    "Unreadable",
    "Unreadable",
    "Unreadable",
    "Unreadable",
    "Unreadable",
    "Unreadable",
    "Unreadable",
];
const REFLASHING: Text = [
    "正在重新刷入 Skyward",
    "正在重新刷入 Skyward",
    "Flashing Skyward again",
    "Flashing Skyward again",
    "Flashing Skyward again",
    "Flashing Skyward again",
    "Flashing Skyward again",
    "Flashing Skyward again",
    "Flashing Skyward again",
    "Flashing Skyward again",
    "Flashing Skyward again",
    "Flashing Skyward again",
    "Flashing Skyward again",
    "Flashing Skyward again",
    "Flashing Skyward again",
];
const READING_FILE: Text = [
    "正在读取文件",
    "正在讀取檔案",
    "Reading the file",
    "Reading the file",
    "Reading the file",
    "Reading the file",
    "Reading the file",
    "Reading the file",
    "Reading the file",
    "Reading the file",
    "Reading the file",
    "Reading the file",
    "Reading the file",
    "Reading the file",
    "Reading the file",
];
const RUN_ON_COMPUTER: Text = [
    "在电脑上运行",
    "在電腦上執行",
    "Run on the computer",
    "Run on the computer",
    "Run on the computer",
    "Run on the computer",
    "Run on the computer",
    "Run on the computer",
    "Run on the computer",
    "Run on the computer",
    "Run on the computer",
    "Run on the computer",
    "Run on the computer",
    "Run on the computer",
    "Run on the computer",
];
const GETTING_READY: Text = [
    "正在准备",
    "正在準備",
    "Getting ready",
    "Getting ready",
    "Getting ready",
    "Getting ready",
    "Getting ready",
    "Getting ready",
    "Getting ready",
    "Getting ready",
    "Getting ready",
    "Getting ready",
    "Getting ready",
    "Getting ready",
    "Getting ready",
];
const WIPING: Text = [
    "正在清除",
    "正在清除",
    "Wiping",
    "Wiping",
    "Wiping",
    "Wiping",
    "Wiping",
    "Wiping",
    "Wiping",
    "Wiping",
    "Wiping",
    "Wiping",
    "Wiping",
    "Wiping",
    "Wiping",
];
const KEEP_POWER: Text = [
    "清除完成前，请不要关机。",
    "清除完成前，請不要關機。",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
    "Don't power off until it's wiped.",
];
const VERIFIED: Text = [
    "已校验",
    "已驗證",
    "Verified",
    "Verified",
    "Verified",
    "Verified",
    "Verified",
    "Verified",
    "Verified",
    "Verified",
    "Verified",
    "Verified",
    "Verified",
    "Verified",
    "Verified",
];
const VERIFYING: Text = [
    "正在校验",
    "正在驗證",
    "Verifying",
    "Verifying",
    "Verifying",
    "Verifying",
    "Verifying",
    "Verifying",
    "Verifying",
    "Verifying",
    "Verifying",
    "Verifying",
    "Verifying",
    "Verifying",
    "Verifying",
];
const WAITING_TURN: Text = [
    "等待中",
    "等待中",
    "Waiting",
    "Waiting",
    "Waiting",
    "Waiting",
    "Waiting",
    "Waiting",
    "Waiting",
    "Waiting",
    "Waiting",
    "Waiting",
    "Waiting",
    "Waiting",
    "Waiting",
];
const RESTORE_FAILED: Text = [
    "恢复失败",
    "還原失敗",
    "Restore failed",
    "Restore failed",
    "Restore failed",
    "Restore failed",
    "Restore failed",
    "Restore failed",
    "Restore failed",
    "Restore failed",
    "Restore failed",
    "Restore failed",
    "Restore failed",
    "Restore failed",
    "Restore failed",
];
const ACTIONS: Text = [
    "操作",
    "操作",
    "Actions",
    "Actions",
    "Actions",
    "Actions",
    "Actions",
    "Actions",
    "Actions",
    "Actions",
    "Actions",
    "Actions",
    "Actions",
    "Actions",
    "Actions",
];
const CHECKING: Text = [
    "正在检查更新",
    "正在檢查更新",
    "Checking for updates",
    "Checking for updates",
    "Checking for updates",
    "Checking for updates",
    "Checking for updates",
    "Checking for updates",
    "Checking for updates",
    "Checking for updates",
    "Checking for updates",
    "Checking for updates",
    "Checking for updates",
    "Checking for updates",
    "Checking for updates",
];
const DOWNLOADING: Text = [
    "正在下载",
    "正在下載",
    "Downloading",
    "Downloading",
    "Downloading",
    "Downloading",
    "Downloading",
    "Downloading",
    "Downloading",
    "Downloading",
    "Downloading",
    "Downloading",
    "Downloading",
    "Downloading",
    "Downloading",
];
const CHECKING_PART: Text = [
    "正在校验已下载的部分",
    "正在驗證已下載的部分",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
    "Verifying what was downloaded",
];
const NOTES: Text = [
    "注意事项",
    "注意事項",
    "Notes",
    "Notes",
    "Notes",
    "Notes",
    "Notes",
    "Notes",
    "Notes",
    "Notes",
    "Notes",
    "Notes",
    "Notes",
    "Notes",
    "Notes",
];
const CANCEL: Text = [
    "取消",
    "取消",
    "Cancel",
    "Cancel",
    "Cancel",
    "Cancel",
    "Cancel",
    "Cancel",
    "Cancel",
    "Cancel",
    "Cancel",
    "Cancel",
    "Cancel",
    "Cancel",
    "Cancel",
];
const RENAME: Text = [
    "重命名",
    "重新命名",
    "Rename",
    "Rename",
    "Rename",
    "Rename",
    "Rename",
    "Rename",
    "Rename",
    "Rename",
    "Rename",
    "Rename",
    "Rename",
    "Rename",
    "Rename",
];
const LICENSE_TERMS: Text = [
    "许可证",
    "授權條款",
    "License",
    "License",
    "License",
    "License",
    "License",
    "License",
    "License",
    "License",
    "License",
    "License",
    "License",
    "License",
    "License",
];
const POWERS_OFF_WHEN_DONE: Text = [
    "完成后将自动关机。",
    "完成後將自動關機。",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
    "Powers off on its own once done.",
];
const REBOOTS_WHEN_DONE: Text = [
    "完成后将自动重启。",
    "完成後將自動重新啟動。",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
    "Reboots on its own once done.",
];
const SKYWARD_UPDATE: Text = [
    "版本更新",
    "版本更新",
    "Version update",
    "Version update",
    "Version update",
    "Version update",
    "Version update",
    "Version update",
    "Version update",
    "Version update",
    "Version update",
    "Version update",
    "Version update",
    "Version update",
    "Version update",
];
/// Reporting a bug to the author, with the logs that show it.
const FEEDBACK: Text = [
    "问题反馈",
    "問題回報",
    "Report a bug",
    "Report a bug",
    "Report a bug",
    "Report a bug",
    "Report a bug",
    "Report a bug",
    "Report a bug",
    "Report a bug",
    "Report a bug",
    "Report a bug",
    "Report a bug",
    "Report a bug",
    "Report a bug",
];

/// How many lines of a bug report the field shows at once: more than a
/// sentence, and few enough that the page under it is still read.
const WORDS_LINES: usize = 5;
const FILES: Text = [
    "文件管理",
    "檔案管理",
    "File manager",
    "File manager",
    "File manager",
    "File manager",
    "File manager",
    "File manager",
    "File manager",
    "File manager",
    "File manager",
    "File manager",
    "File manager",
    "File manager",
    "File manager",
];
const TERMINAL: Text = [
    "终端",
    "終端機",
    "Terminal",
    "Terminal",
    "Terminal",
    "Terminal",
    "Terminal",
    "Terminal",
    "Terminal",
    "Terminal",
    "Terminal",
    "Terminal",
    "Terminal",
    "Terminal",
    "Terminal",
];
const LANGUAGE: Text = [
    "语言",
    "語言",
    "Language",
    "Language",
    "Language",
    "Language",
    "Language",
    "Language",
    "Language",
    "Language",
    "Language",
    "Language",
    "Language",
    "Language",
    "Language",
];
const TIME_ZONE: Text = [
    "时区",
    "時區",
    "Time zone",
    "Time zone",
    "Time zone",
    "Time zone",
    "Time zone",
    "Time zone",
    "Time zone",
    "Time zone",
    "Time zone",
    "Time zone",
    "Time zone",
    "Time zone",
    "Time zone",
];
const LICENSE: Text = [
    "开源许可",
    "開放原始碼授權",
    "Open source licenses",
    "Open source licenses",
    "Open source licenses",
    "Open source licenses",
    "Open source licenses",
    "Open source licenses",
    "Open source licenses",
    "Open source licenses",
    "Open source licenses",
    "Open source licenses",
    "Open source licenses",
    "Open source licenses",
    "Open source licenses",
];
const FONTS: Text = [
    "界面字体",
    "介面字型",
    "UI fonts",
    "UI fonts",
    "UI fonts",
    "UI fonts",
    "UI fonts",
    "UI fonts",
    "UI fonts",
    "UI fonts",
    "UI fonts",
    "UI fonts",
    "UI fonts",
    "UI fonts",
    "UI fonts",
];
const DUAL_BOOT: Text = [
    "第二系统",
    "第二系統",
    "Dual boot",
    "Dual boot",
    "Dual boot",
    "Dual boot",
    "Dual boot",
    "Dual boot",
    "Dual boot",
    "Dual boot",
    "Dual boot",
    "Dual boot",
    "Dual boot",
    "Dual boot",
    "Dual boot",
];

/// A canvas for a page `h` tall in the colour `bg`, holding the rows `cx`
/// asks to be drawn.
fn sheet(cx: &Ctx, h: u32, bg: Rgba) -> Canvas {
    let w = cx.t.m.w;
    let Band { rows, cols } = cx.band.unwrap_or(Band { rows: (0, h), cols: (0, w) });
    Canvas::band(w, h, rows.0..rows.1, cols.0..cols.1, bg)
}

fn page(cx: &Ctx) -> Canvas {
    sheet(cx, cx.t.m.h, cx.t.pal.bg)
}

/// A canvas tall enough for a scrolling page; [`fit`] trims it to the content.
fn long_page(cx: &Ctx) -> Canvas {
    sheet(cx, cx.t.m.h * 3, cx.t.pal.bg)
}

/// [`long_page`] for a page that may hold more rows than three screens: the
/// sheet is `screens` screens tall, and [`fit`] trims it to the content. A
/// list of every partition on the phone runs past three.
fn long_page_of(cx: &Ctx, screens: u32) -> Canvas {
    sheet(cx, cx.t.m.h * screens.max(1), cx.t.pal.bg)
}

/// How many screens a list of `rows` takes, at `top` down the page, with
/// `room` for whatever stands above and below it, each row `row_h` tall.
fn screens_for(t: &Theme, top: f32, room: f32, rows: usize, row_h: f32) -> u32 {
    let h = top + room + row_h * rows as f32;
    (h / t.m.h as f32).ceil() as u32 + 1
}
/// How tall a long page whose content ends at `bottom` is: to the bottom
/// margin after it, and never shorter than one screen.
fn fitted(t: &Theme, bottom: f32) -> u32 {
    ((bottom + t.m.margin * 2.0).ceil() as u32).max(t.m.h)
}

/// Trim a long page to its content plus the bottom margin, and never shorter
/// than one screen.
fn fit(c: Canvas, t: &Theme, bottom: f32) -> Canvas {
    c.crop_height(fitted(t, bottom))
}

fn hits(rects: &[Rect], actions: &[Action]) -> Vec<Hit> {
    rects.iter().zip(actions).map(|(&rect, &action)| Hit { rect, action }).collect()
}

/// Recovery home, laid out like the Settings root: title, search, a device
/// card, then grouped entries. The status bar carries the readings: the
/// slot, the data partition's lock and the battery's temperature.
pub fn home(cx: &Ctx) -> Frame {
    let (t, f, i, s, l) = (cx.t, cx.f, cx.info, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    // The search bar is left room on the page and drawn over it, folding
    // away as the page scrolls up.
    let bar = Rect::new(t.m.margin, t.m.content_top, t.m.w as f32 - t.m.margin * 2.0, t.m.search_h);
    let mut y = bar.y + bar.h + t.m.after_search;
    let cards_top = y;
    let fold = Fold { bar, hint: l.pick(texts::search::HINT), span: cards_top - bar.y };
    out.push(Hit { rect: bar, action: Action::Open(Screen::Search) });

    // Under the phone's name the card holds only what asks for something to
    // be done: a battery too low to install on, an update waiting to merge,
    // and a restart into the slot the system does not run from now.
    let low = match i.battery.as_ref().and_then(skyward_install::battery_refusal) {
        Some(Refusal::Battery { level, charging }) => Some(fmt::battery_low(l, level, charging)),
        _ => None,
    };
    let next = match (i.slot, &cx.state.boot_slots) {
        (Some(now), Some(Ok(slots))) if slots.active != now => Some(fmt::next_boot(l, slots.active, now)),
        _ => None,
    };
    let mut rows = Vec::new();
    let mut opens = Vec::new();
    if let Some((title, note)) = &low {
        rows.push(Row::new(title).icon(RED, Icon::Battery).subtitle(note).trail(Trail::None));
        opens.push(None);
    }
    if let Some(note) = fmt::snapshot_note(l, &i.snapshot) {
        rows.push(Row::new(fmt::snapshot(l, &i.snapshot)).icon(ORANGE, Icon::Layers).subtitle(note).trail(Trail::Arrow));
        opens.push(Some(Action::Open(Screen::Update)));
    }
    if let Some((title, note)) = &next {
        rows.push(Row::new(title).icon(ORANGE, Icon::Power).subtitle(note).trail(Trail::Arrow));
        opens.push(Some(Action::Open(Screen::Slot)));
    }
    if cx.state.system_notice.is_some() {
        let why = l.pick(texts::home::TAP_TO_SEE_WHY);
        rows.push(Row::new(l.pick(texts::systems::SECOND_FAILED)).icon(RED, Icon::Systems).subtitle(why).trail(Trail::Arrow));
        opens.push(Some(Action::Open(Screen::Systems)));
    }
    // The account's row is the phone's own: the owner's picture and the name
    // the phone was given, with the model and the system under it, and the page
    // of details a tap away. It waits for the partition it is kept on to be
    // open; before that neither the picture nor the name is there.
    // The system version and the codename under it. The version says which
    // build the phone runs, which is what the row is asked about; the model is
    // on the page a tap opens, and inside the row it was the least telling of
    // the two. A phone whose partitions are not mounted yet has no version
    // read, so its model stands in until it does.
    let version = i.os_version.as_deref().unwrap_or(&i.model);
    let detail = format!("{} \u{b7} {}", version, i.codename);
    match &s.device_name {
        Some(name) => {
            let account = widgets::account_card(&mut c, t, f, y, name, Some(&detail), s.avatar.as_ref());
            out.push(Hit { rect: account.rows[0], action: Action::Open(Screen::About) });
            y = account.bottom + t.m.card_gap;
        }
        // Nothing of the phone itself read yet: the model alone stands in, and
        // a tap still goes to the details.
        None => {
            let card = widgets::account_card(&mut c, t, f, y, &i.model, Some(&detail), None);
            out.push(Hit { rect: card.rows[0], action: Action::Open(Screen::About) });
            y = card.bottom + t.m.card_gap;
        }
    }
    // What asks for something to be done stands on its own under it: a battery
    // too low to install on, an update waiting to merge, a restart into the
    // slot the system does not run from now.
    if !rows.is_empty() {
        let list = widgets::list(&mut c, t, f, y, &rows);
        out.extend(list.rows.iter().zip(&opens).filter_map(|(&rect, open)| open.map(|action| Hit { rect, action })));
        y = list.bottom + t.m.card_gap;
    }
    // On a phone whose WLAN the recovery brings up, the WLAN comes first
    // under the device, as on the stock settings home; with it, where the
    // phone can hold a second system, the two systems.
    // The network the phone is joined to and the file manager come first.
    let mut first_rows = Vec::new();
    let mut first_opens = Vec::new();
    if let Some(w) = &cx.state.wlan {
        first_rows.push(Row::new(l.pick(WLAN)).icon(BLUE, Icon::Wlan).trail(Trail::Value(fmt::wlan_value(l, w))));
        first_opens.push(Action::Open(Screen::Wlan));
    }
    first_rows.push(Row::new(l.pick(FILES)).icon(BLUE, Icon::Folder));
    first_opens.push(Action::Open(Screen::Files));
    // 模块管理紧挨着文件管理：装模块就是往 data 里放文件，放在这里顺手。
    first_rows.push(Row::new(l.pick(MODULES)).icon(BLUE, Icon::Layers));
    first_opens.push(Action::Open(Screen::Modules));
    // 应用管理就在模块管理下面：一样是 data 分区里的东西，一个是模块，
    // 一个是应用。
    first_rows.push(Row::new(l.pick(DATA_APPS)).icon(PURPLE, Icon::Layers));
    first_opens.push(Action::Open(Screen::DataApps));
    let first = widgets::list(&mut c, t, f, y, &first_rows);
    out.extend(hits(&first.rows, &first_opens));
    y = first.bottom + t.m.card_gap;
    // Then the three rows that put something on the phone or take it off.
    let tools = widgets::list(
        &mut c,
        t,
        f,
        y,
        &[
            Row::new(l.pick(["安装刷机", "安裝刷機", "Install"])).icon(GREEN, Icon::Install),
            Row::new(l.pick(["创建备份", "建立備份", "Backup"])).icon(BLUE, Icon::Backup),
            Row::new(l.pick(["恢复备份", "還原備份", "Restore"])).icon(PURPLE, Icon::Restore),
        ],
    );
    out.extend(hits(&tools.rows, &[Action::Open(Screen::Install), Action::Open(Screen::Backup), Action::Open(Screen::Restore)]));
    y = tools.bottom + t.m.card_gap;
    // Where the phone holds a second system, it comes first here.
    let mut system_rows = Vec::new();
    let mut system_opens = Vec::new();
    let starts = systems_value(cx);
    if cx.state.offers(Screen::Systems) {
        let row = Row::new(l.pick(DUAL_BOOT)).icon(PURPLE, Icon::Systems);
        system_rows.push(match &starts {
            Some(value) => row.trail(Trail::Value(value)),
            None => row,
        });
        system_opens.push(Action::Open(Screen::Systems));
    }
    // 充电就排在高级工具上面：它比高级工具里的东西常用，也不该藏在里面。
    // 充电时右侧显示充电器当前送来的功率，和充电页上的数字是同一个。
    let charging = match &i.battery {
        Some(b) if b.plugged => {
            let value = s.charge.now.map_or_else(|| l.pick(["充电中", "充電中", "Charging"]).to_owned(), fmt::watts);
            Some(value)
        }
        _ => None,
    };
    match &charging {
        Some(value) => system_rows.push(Row::new(l.pick(CHARGE)).icon(GREEN, Icon::Battery).trail(Trail::Value(value))),
        None => system_rows.push(Row::new(l.pick(CHARGE)).icon(SLATE, Icon::Battery)),
    }
    system_opens.push(Action::Open(Screen::Charge));
    system_rows.push(Row::new(l.pick(ADVANCED)).icon(SLATE, Icon::More));
    system_opens.push(Action::Open(Screen::Advanced));
    system_rows.push(Row::new(l.pick(SETTINGS)).icon(SLATE, Icon::Gear));
    system_opens.push(Action::Open(Screen::Settings));    let system = widgets::list(&mut c, t, f, y, &system_rows);
    out.extend(hits(&system.rows, &system_opens));
    // At the foot: rebooting, wiping and this recovery's own update.
    let mut footer_rows = Vec::new();
    let mut footer_opens = Vec::new();
    let target = l.pick(REBOOT_TARGETS[cx.state.reboot].0);
    footer_rows.push(Row::new(l.pick(REBOOT)).icon(ORANGE, Icon::Power).trail(Trail::Value(target)));
    footer_opens.push(Action::Open(Screen::Reboot));
    footer_rows.push(Row::new(l.pick(["清除数据", "清除資料", "Wipe data"])).icon(RED, Icon::Wipe));
    footer_opens.push(Action::Open(Screen::Wipe));
    if cx.state.offers(Screen::SkywardUpdate) {
        let row = Row::new(l.pick(SKYWARD_UPDATE)).icon(BLUE, Icon::Install);
        footer_rows.push(match skyward_value(cx) {
            Some(value) => row.trail(Trail::Value(value)),
            None => row,
        });
        footer_opens.push(Action::Open(Screen::SkywardUpdate));
    }
    // 问题反馈排在版本更新下面：更新之后多半就是发现了问题。
    footer_rows.push(Row::new(l.pick(FEEDBACK)).icon(BLUE, Icon::Log));
    footer_opens.push(Action::Open(Screen::Feedback));
    let footer = widgets::list(&mut c, t, f, system.bottom + t.m.card_gap, &footer_rows);
    out.extend(hits(&footer.rows, &footer_opens));
    // However little it holds, the page scrolls far enough to fold its bar
    // away.
    let height = fitted(t, footer.bottom).max(t.m.h);
    Frame { back: false, fold: Some(fold), ..Frame::new(t, c.crop_height(height), out, HOME_TITLE, cards_top) }
}

/// What the home page says beside its entry to the systems: the one that
/// starts, where a second system is installed, once they are read.
fn systems_value(cx: &Ctx) -> Option<String> {
    let l = cx.state.lang;
    match &cx.state.systems {
        Some(Ok(systems)) => Some(match (&systems.other, starting(systems)) {
            (None, _) => l.pick(texts::home::NO_SECOND_SYSTEM).into(),
            (Some(_), Who::Installed) => systems.installed.name.clone(),
            (Some(other), Who::Other) => other.name.clone(),
        }),
        _ => None,
    }
}

/// What the advanced page says beside its system update entry: a download
/// under way, a newer system found, or where an update the system installed
/// stands.
fn update_value(cx: &Ctx) -> &'static str {
    let (s, l) = (cx.state, cx.state.lang);
    if s.downloading() {
        return l.pick(texts::update::DOWNLOADING);
    }
    if s.newest().is_some_and(|r| cx.info.os_version.as_deref() != Some(r.version.as_str())) {
        return l.pick(texts::update::NEW_VERSION);
    }
    fmt::update_value(l, &s.update)
}

pub fn root(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let mut y = t.m.content_top;

    let (kernel_su, magisk) = (kernel_su_saying(&s), magisk_saying(&s));
    let rows = [
        Row::new(l.pick(texts::kernel_su::TITLE))
            .icon(BLUE, Icon::KernelSu)
            .trail(Trail::Value(&kernel_su)),
        Row::new(l.pick(texts::magisk::TITLE))
            .icon(PURPLE, Icon::Magisk)
            .trail(Trail::Value(&magisk)),
        Row::new(l.pick(MODULES)).icon(GREEN, Icon::Layers),
    ];
    let opens = [
        Action::Open(Screen::KernelSu),
        Action::Open(Screen::Magisk),
        Action::Open(Screen::Modules),
    ];
    let first = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&first.rows, &opens));
    y = first.bottom + t.m.card_gap;

    // 在线安装：这两张卡本身就是写入动作（要滑动确认），与上面两行「点进去看
    // 现状、选 KMI」是两回事，不会进同一个页面。
    y = widgets::section(&mut c, t, f, y, l.pick(texts::root::INSTALL_ONLINE));
    let (ks_label, mg_label) = (
        tr!(l, "滑动以在线写入 KernelSU", "滑動以線上寫入 KernelSU", "Slide to write KernelSU online"),
        tr!(l, "滑动以在线写入 Magisk", "滑動以線上寫入 Magisk", "Slide to write Magisk online"),
    );
    let (ks_progress, ks_text) = write_state(s.kernel_su_job.as_ref(), s.root_bytes, l, &ks_label);
    let (mg_progress, mg_text) = write_state(s.magisk_job.as_ref(), s.root_bytes, l, &mg_label);
    for (label, progress, which) in [(ks_text, ks_progress, Confirm::KernelSu), (mg_text, mg_progress, Confirm::Magisk)] {
        let track = widgets::confirm(&mut c, t, f, y, &label, false, progress);
        out.push(Hit { rect: track, action: Action::Confirm(which) });
        y = track.y + track.h + t.m.card_gap;
    }

    y = widgets::section(&mut c, t, f, y, l.pick(texts::root::TITLE));
    let auto_root = auto_root_saying(s, l);
    let rows = [
        Row::new(l.pick(texts::root::AUTO_PATCH)).trail(Trail::Value(&auto_root)),
    ];
    let picks = [Action::SetAutoRoot(s.auto_root.next())];
    let second = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&second.rows, &picks));
    y = second.bottom;
    let note = widgets::footnote(&mut c, t, f, y, l.pick(texts::root::AUTO_PATCH_NOTE));
    Frame { back: false, ..Frame::new(t, c.crop_height(fitted(t, note)), out, l.pick(texts::root::TITLE), t.m.content_top) }
}

/// KernelSU 的页：现状、要写的 KMI，以及写进当前槽的 boot 镜像。
pub fn kernel_su(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let mut y = t.m.content_top;

    let status = s.kernel_su.as_ref().and_then(|r| r.as_ref().ok());
    let why = match s.kernel_su.as_ref() {
        Some(Err(why)) => Some(why.clone()),
        _ => None,
    };
    let mut rows = Vec::new();
    let version = status
        .and_then(|st| st.version.clone())
        .unwrap_or_else(|| l.pick(texts::kernel_su::NONE).into());
    let (slot_label, lock_label, latest_label) = (
        tr!(l, "当前槽位", "目前槽位", "Current slot"),
        tr!(l, "开机锁", "開機鎖", "Bootloader"),
        tr!(l, "在线最新", "線上最新", "Latest online"),
    );
    let latest = online_saying(l, s.latest_kernelsu.as_deref(), status.and_then(|st| st.version.as_deref()), !status.is_some_and(|st| st.installed));
    match status {
        Some(st) => {
            rows.push(Row::new(l.pick(if st.installed { texts::kernel_su::INSTALLED } else { texts::kernel_su::NOT_INSTALLED })).trail(Trail::Value(&version)));
            rows.push(Row::new(&latest_label).trail(Trail::Value(&latest)));
            if let Some(target) = &st.backup_target {
                rows.push(Row::new(l.pick(texts::kernel_su::INSTALLED_TO)).trail(Trail::Value(target)));
            }
            if let Some(slot) = &st.slot {
                rows.push(Row::new(&slot_label).trail(Trail::Value(slot)));
            }
            rows.push(Row::new(&lock_label).trail(Trail::Value(l.pick(if st.unlocked { ["已解锁", "已解鎖", "Unlocked"] } else { ["已锁定", "已鎖定", "Locked"] }))));
        }
        None => rows.push(Row::new(l.pick(texts::kernel_su::NOT_READ)).trail(Trail::None)),
    }
    let list = widgets::list(&mut c, t, f, y, &rows);
    y = list.bottom + t.m.card_gap;

    // Writing it: the stage the job is at says how far it has come, and the
    // swipe under it writes the KMI the list chose.
    let said = match s.kernel_su_job.as_ref().map(|j| j.stage) {
        Some(KernelSuStage::Preparing) => l.pick(texts::kernel_su::WORKING),
        Some(KernelSuStage::BackingUp) => l.pick(texts::kernel_su::WORKING),
        Some(KernelSuStage::Patching) => l.pick(texts::kernel_su::WORKING),
        Some(KernelSuStage::Writing) => l.pick(texts::kernel_su::WORKING),
        Some(KernelSuStage::Checking) | None => l.pick(texts::kernel_su::INSTALL),
    };
    let stage_note = widgets::footnote(&mut c, t, f, y, said);
    y = stage_note + t.m.card_gap;
    let idle = tr!(l, "滑动以写入 KernelSU", "滑動以寫入 KernelSU", "Slide to write KernelSU");
    let (progress, write_label) = write_state(s.kernel_su_job.as_ref(), s.root_bytes, l, &idle);
    let track = widgets::confirm(&mut c, t, f, y, &write_label, false, progress);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::KernelSu) });
    y = track.y + track.h + t.m.card_gap;

    // The KMIs, each one the row to write it; the one the phone's kernel
    // names is marked, and the one written is too.
    if let Some(st) = status {
        y = widgets::section(&mut c, t, f, y, l.pick(texts::kernel_su::KMI));
        let kmis: Vec<Row> = st
            .kmis
            .iter()
            .enumerate()
            .map(|(k, row)| Row::new(&row.id).subtitle(&row.target).trail(Trail::Check(s.kernel_su_kmi == Some(k) || st.default == Some(k))))
            .collect();
        let picks: Vec<Action> = (0..kmis.len()).map(Action::KernelSuKmi).collect();
        let chosen = widgets::list(&mut c, t, f, y, &kmis);
        out.extend(hits(&chosen.rows, &picks));
        y = chosen.bottom + t.m.card_gap;
    }
    let note = widgets::footnote(&mut c, t, f, y, l.pick(texts::kernel_su::NOTE));
    y = note + t.m.card_gap;
    let online = widgets::footnote(&mut c, t, f, y, l.pick(texts::kernel_su::ONLINE_NOTE));
    y = online + t.m.card_gap;
    if let Some(why) = &why {
        let at = widgets::footnote(&mut c, t, f, y, why);
        y = at + t.m.card_gap;
    }

    let saved_at = widgets::section(&mut c, t, f, y, &texts::kernel_su::restore_target(l, st_target(status.and_then(|st| st.backup_target.clone()))));
    // 没有备份记录时不摆那一行可点的「已恢复」——点了必然报 no KernelSU backup
    // record（2026-11-26，取自 Ff428 的真机修正）。
    let bottom = if status.is_some_and(|st| st.has_backup) {
        let row = Row::new(l.pick(texts::kernel_su::RESTORED)).icon(SLATE, Icon::Restore);
        let list = widgets::list(&mut c, t, f, saved_at, &[row]);
        out.extend(hits(&list.rows, &[Action::RestoreKernelSu]));
        list.bottom
    } else {
        widgets::footnote(&mut c, t, f, saved_at, l.pick([
            "本机没有原厂 boot 备份，先写入一次 Root 才有得恢复。",
            "本機沒有原廠 boot 備份，先寫入一次 Root 才有得恢復。",
            "No stock boot backup on this phone; write a root once before restoring.",
        ]))
    };
    Frame { back: false, ..Frame::new(t, c.crop_height(fitted(t, bottom)), out, l.pick(texts::kernel_su::TITLE), t.m.content_top) }
}


/// 写入卡该停在哪、写什么：滑到底开始后停在底并说「正在写入」；写完停在底说
/// 「已写入」；失败回到原位，让手指知道可以再滑一次（2026-11-26）。
fn write_state(job: Option<&KernelSuJob>, bytes: Option<(u64, Option<u64>, u64)>, l: Lang, idle: &str) -> (f32, String) {
    match job {
        Some(job) if job.outcome.is_none() => {
            // 下载中把百分比与速度写在卡上，看得见进展（2026-11-26）。
            let worked = match bytes {
                Some((done, Some(total), rate)) if total > 0 => {
                    format!("{}% · {}", done.saturating_mul(100) / total, per_second(rate))
                }
                Some((done, None, rate)) => format!("{} · {}", fmt::bytes(done), per_second(rate)),
                _ => tr!(l, "准备中…", "準備中…", "Preparing…"),
            };
            (1.0, format!("{} {}", tr!(l, "正在在线写入", "正在線上寫入", "Writing online"), worked))
        }
        Some(job) if matches!(job.outcome, Some(KernelSuEnd::Failed(_))) => {
            (0.0, tr!(l, "写入未完成，可以再滑一次", "寫入未完成，可以再滑一次", "Not written; slide again"))
        }
        Some(_) => (1.0, tr!(l, "已写入", "已寫入", "Written")),
        None => (0.0, idle.to_string()),
    }
}

/// 在线那一版怎么写在页面上：比装着的这一版新就写「可升级」，一样或旧写
/// 「已是最新」，装都没装就只写版本号（2026-11-26）。
fn online_saying(l: Lang, online: Option<&str>, installed: Option<&str>, not_installed: bool) -> String {
    let Some(online) = online else {
        return tr!(l, "未获取（需要联网）", "未取得（需要連網）", "Not fetched (needs a network)");
    };
    if not_installed {
        return online.to_string();
    }
    match is_newer(installed, Some(online)) {
        Some(true) => format!("{online} · {}", tr!(l, "可升级", "可升級", "Update available")),
        Some(false) => format!("{online} · {}", tr!(l, "已是最新", "已是最新", "Up to date")),
        None => online.to_string(),
    }
}

/// 每秒多少字节，写成人看得懂的样子。
fn per_second(rate: u64) -> String {
    if rate == 0 {
        "…".to_string()
    } else {
        format!("{}/s", fmt::bytes(rate))
    }
}

/// The KMI the KernelSU page writes: the list, each one checked or not.
pub fn kernel_su_kmi(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let y = t.m.content_top;
    let st = s.kernel_su.as_ref().and_then(|r| r.as_ref().ok());
    let rows: Vec<Row> = st
        .map(|st| {
            st.kmis
                .iter()
                .enumerate()
                .map(|(k, row)| Row::new(&row.id).subtitle(&row.target).trail(Trail::Check(s.kernel_su_kmi == Some(k) || st.default == Some(k))))
                .collect()
        })
        .unwrap_or_default();
    let picks: Vec<Action> = (0..rows.len()).map(Action::KernelSuKmi).collect();
    let list = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&list.rows, &picks));
    Frame { back: false, ..Frame::new(t, c.crop_height(fitted(t, list.bottom)), out, l.pick(texts::kernel_su::KMI), t.m.content_top) }
}

/// Magisk 的页：现状与写进当前槽的 boot 镜像。
pub fn magisk(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let mut y = t.m.content_top;
    let st = s.magisk.as_ref().and_then(|r| r.as_ref().ok());
    let version = st
        .and_then(|st| st.version.clone())
        .unwrap_or_else(|| l.pick(texts::magisk::NONE).into());
    let (latest_label, latest) = (
        tr!(l, "在线最新", "線上最新", "Latest online"),
        online_saying(l, s.latest_magisk.as_deref(), st.and_then(|st| st.version.as_deref()), !st.is_some_and(|st| st.installed)),
    );
    let mut rows = Vec::new();
    match st {
        Some(st) => {
            rows.push(Row::new(l.pick(if st.installed { texts::kernel_su::INSTALLED } else { texts::kernel_su::NOT_INSTALLED })).trail(Trail::Value(&version)));
            rows.push(Row::new(&latest_label).trail(Trail::Value(&latest)));
            if let Some(target) = &st.backup_target {
                rows.push(Row::new(l.pick(texts::kernel_su::INSTALLED_TO)).trail(Trail::Value(target)));
            }
            rows.push(Row::new(&latest_label).trail(Trail::Value(&latest)));
            rows.push(Row::new(l.pick(texts::magisk::TARGET)).trail(Trail::Value(&st.target)));
            if let Some(slot) = &st.slot {
                rows.push(Row::new(l.pick(texts::kernel_su::TARGET)).trail(Trail::Value(slot)));
            }
        }
        None => rows.push(Row::new(l.pick(texts::kernel_su::NOT_READ)).trail(Trail::None)),
    }
    let list = widgets::list(&mut c, t, f, y, &rows);
    y = list.bottom + t.m.card_gap;
    let note = widgets::footnote(&mut c, t, f, y, l.pick(texts::magisk::NOTE));
    y = note + t.m.card_gap;
    let online = widgets::footnote(&mut c, t, f, y, l.pick(texts::kernel_su::ONLINE_NOTE));
    y = online + t.m.card_gap;
    let idle = tr!(l, "滑动以写入 Magisk", "滑動以寫入 Magisk", "Slide to write Magisk");
    let (progress, write_label) = write_state(s.magisk_job.as_ref(), s.root_bytes, l, &idle);
    let track = widgets::confirm(&mut c, t, f, y, &write_label, false, progress);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Magisk) });
    y = track.y + track.h + t.m.card_gap;
    let saved_at = widgets::section(&mut c, t, f, y, &texts::magisk::restore_target(l, st_target(st.map(|s| s.target.clone()))));
    let bottom = if st.is_some_and(|st| st.has_backup) {
        let row = Row::new(l.pick(texts::kernel_su::RESTORED)).icon(SLATE, Icon::Restore);
        let list = widgets::list(&mut c, t, f, saved_at, &[row]);
        out.extend(hits(&list.rows, &[Action::RestoreMagisk]));
        list.bottom
    } else {
        widgets::footnote(&mut c, t, f, saved_at, l.pick([
            "本机没有原厂 boot 备份，先写入一次 Root 才有得恢复。",
            "本機沒有原廠 boot 備份，先寫入一次 Root 才有得恢復。",
            "No stock boot backup on this phone; write a root once before restoring.",
        ]))
    };
    Frame { back: false, ..Frame::new(t, c.crop_height(fitted(t, bottom)), out, l.pick(texts::magisk::TITLE), t.m.content_top) }
}

/// What the KernelSU row of the root page says beside its name.
fn kernel_su_saying(s: &State) -> String {
    match s.kernel_su.as_ref() {
        Some(Ok(st)) => st.version.as_deref().map(short_version).unwrap_or_else(|| s.lang.pick(texts::kernel_su::NOT_INSTALLED).into()),
        Some(Err(why)) => clip_saying(why),
        None => s.lang.pick(texts::kernel_su::NOT_READ).to_owned(),
    }
}

/// `3.3.1-56-gcfc60f8d (uapi: 6) (32736)` → `3.3.1`：右侧那一栏很窄，
/// 整串写进去会把「KernelSU」挤成「K...」（2026-11-26，取自 Ff428 的真机修正）。
fn short_version(version: &str) -> String {
    version.split(['-', ' ', '(', '+']).next().unwrap_or(version).to_string()
}

/// 一行说法最多这么长，理由太长就截断，别把右侧那一栏撑满。
fn clip_saying(why: &str) -> String {
    let line = why.lines().next().unwrap_or(why).trim();
    let mut out: String = line.chars().take(28).collect();
    if out.chars().count() < line.chars().count() {
        out.push('…');
    }
    out
}

/// What the Magisk row of the root page says beside its name.
fn magisk_saying(s: &State) -> String {
    match s.magisk.as_ref() {
        Some(Ok(st)) => st.version.as_deref().map(short_version).unwrap_or_else(|| s.lang.pick(texts::kernel_su::NOT_INSTALLED).into()),
        Some(Err(why)) => clip_saying(why),
        None => s.lang.pick(texts::kernel_su::NOT_READ).to_owned(),
    }
}

/// The partition a backup belongs to, as the restore row's title names it.
fn st_target(target: Option<String>) -> String {
    match target {
        Some(t) => t,
        None => "\u{2014}".to_owned(),
    }
}

/// What the auto-patch choice on the root page says beside its name.
fn auto_root_saying(s: &State, l: Lang) -> String {
    match s.auto_root {
        AutoRoot::None => l.pick(texts::root::NONE).to_owned(),
        AutoRoot::KernelSu => l.pick(texts::kernel_su::TITLE).to_owned(),
        AutoRoot::Magisk => l.pick(texts::magisk::TITLE).to_owned(),
    }
}

/// Preferences, opening with the stock「显示与亮度」theme previews.
pub fn settings(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let themes = widgets::theme_picker(&mut c, t, f, l, top + t.m.dp(8.9));
    out.extend(hits(&themes.rows, &[Action::Theme { dark: false }, Action::Theme { dark: true }]));
    let mut y = widgets::section(&mut c, t, f, themes.bottom, l.pick(texts::settings::BRIGHTNESS));
    let off = fmt::screen_off(l, SCREEN_OFF[s.screen_off]);
    let below = Row::new(l.pick(texts::page::SCREEN_TIMEOUT)).trail(Trail::Value(&off));
    let slider = widgets::slider_card(&mut c, t, f, y, Caption::Sun, l.pick(texts::settings::SCREEN_BRIGHTNESS), s.brightness, Some(below));
    out.push(Hit { rect: slider.track, action: Action::Slider(Slider::Brightness) });
    if let Some(r) = slider.below {
        out.push(Hit { rect: r, action: Action::Open(Screen::ScreenOff) });
    }
    y = widgets::section(&mut c, t, f, slider.bottom, l.pick(texts::settings::VIBRATION));
    let below = Row::new(l.pick(texts::settings::TOUCH_VIBRATION)).trail(Trail::Switch(s.vibrate));
    let vibration = widgets::slider_card(&mut c, t, f, y, Caption::Vibrate, l.pick(texts::settings::VIBRATION_INTENSITY), s.vibration, Some(below));
    out.push(Hit { rect: vibration.track, action: Action::Slider(Slider::Vibration) });
    if let Some(r) = vibration.below {
        out.push(Hit { rect: r, action: Action::Toggle(Switch::Vibrate) });
    }
    y = widgets::section(&mut c, t, f, vibration.bottom, l.pick(["通用", "一般", "General"]));
    let general = widgets::list(
        &mut c,
        t,
        f,
        y,
        &[
            Row::new(l.pick(LANGUAGE)).icon(BLUE, Icon::Globe).trail(Trail::Value(l.name())),
            Row::new(l.pick(TIME_ZONE)).icon(ORANGE, Icon::Clock).trail(Trail::Value(l.pick(TIME_ZONES[s.time_zone].name))),
        ],
    );
    out.extend(hits(&general.rows, &[Action::Open(Screen::Language), Action::Open(Screen::TimeZone)]));
    y = widgets::section(&mut c, t, f, general.bottom, "USB");
    let usb = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(texts::settings::FILE_TRANSFER)).icon(BLUE, Icon::Usb).trail(Trail::Switch(s.file_transfer))]);
    out.extend(hits(&usb.rows, &[Action::Toggle(Switch::FileTransfer)]));
    let note = l.pick([
        "开启后，可在电脑上管理内部存储中的文件，需先完成「解密」。",
        "開啟後，可在電腦上管理內部儲存空間中的檔案，需先完成「解密」。",
        "When on, a computer can manage the files in internal storage, once it is decrypted.",
    ]);
    let bottom = widgets::footnote(&mut c, t, f, usb.bottom, note);
    // 终端不在这里：高级页已经有一个，同一个东西开两个入口反而添乱
    //（2026-11-17）。关于与更新也一样，更新在首页底部单独一栏。
    Frame::new(t, fit(c, t, bottom), out, l.pick(SETTINGS), top)
}

/// What the settings page says beside its Skyward update entry: an update
/// under way, or a newer release found.
fn skyward_value(cx: &Ctx) -> Option<&'static str> {
    let (s, l) = (cx.state, cx.state.lang);
    if s.updating_skyward() {
        // 具体百分比在更新页里；首页这一行只说它还在进行，退出来也看得见。
        return Some(l.pick(["正在更新", "正在更新", "Updating"]));
    }
    match (s.skyward_newest(), &s.skyward_job) {
        (_, Some(SkywardJob { outcome: Some(SkywardEnd::Updated(_)), .. })) => Some(l.pick(texts::skyward_update::UPDATED)),
        (Some(n), _) if n.newer && n.build.is_some() => Some(l.pick(texts::update::NEW_VERSION)),
        _ => None,
    }
}

/// How long the screen stays on with nothing touched, opened from the
/// settings.
pub fn screen_off(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    let labels = SCREEN_OFF.map(|ms| fmt::screen_off(l, ms));
    let rows: Vec<Row> = labels.iter().enumerate().map(|(k, label)| Row::new(label).trail(Trail::Check(s.screen_off == k))).collect();
    let times = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = (0..SCREEN_OFF.len()).map(Action::ScreenOff).collect();
    let y = widgets::footnote(&mut c, t, f, times.bottom, l.pick(texts::screen_off::NOTE));
    if SCREEN_OFF[s.screen_off].is_none() {
        widgets::footnote(&mut c, t, f, y, l.pick(texts::screen_off::STAYS_ON));
    }
    Frame::new(t, c, hits(&times.rows, &picks), l.pick(texts::page::SCREEN_TIMEOUT), top)
}

/// Mount manager: the slot's partitions, each with a switch to mount it
/// read-only or unmount it; one that programs are using says which, and
/// stays.
pub fn mount(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;

    let y = widgets::section_first(&mut c, t, f, top, l.pick(texts::mount::PARTITIONS));
    let bottom = match &s.mounts {
        Err(why) => {
            let y = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(texts::mount::UNREADABLE)).trail(Trail::None)]).bottom;
            widgets::footnote(&mut c, t, f, y, why)
        }
        Ok(mounts) if mounts.is_empty() => widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["暂无可挂载的分区", "沒有可掛載的分割區", "No partitions to mount"])).trail(Trail::None)]).bottom,
        Ok(mounts) => {
            let subs: Vec<String> = mounts
                .iter()
                .map(|m| match m.holders.is_empty() {
                    true => m.point.clone(),
                    false => texts::mount::in_use_by(l, &m.point, m.holders.join(l.pick(texts::list::SEPARATOR))),
                })
                .collect();
            let rows: Vec<Row> = mounts
                .iter()
                .zip(&subs)
                .map(|(m, sub)| {
                    let trail = if m.holders.is_empty() { Trail::Switch(m.mounted) } else { Trail::Text(l.pick(texts::mount::IN_USE)) };
                    Row::new(&m.base).subtitle(sub).trail(trail)
                })
                .collect();
            let list = widgets::list(&mut c, t, f, y, &rows);
            out.extend(list.rows.iter().zip(mounts).enumerate().map(|(i, (&rect, m))| {
                let action = match m.holders.is_empty() {
                    true => Action::MountToggle(i),
                    false => Action::Unavailable(l.pick(texts::mount::CANT_UNMOUNT_IN_USE)),
                };
                Hit { rect, action }
            }));
            let y = match &s.mount_failure {
                Some(failure) => widgets::footnote(&mut c, t, f, list.bottom, &fmt::mount_failure(l, failure)),
                None => list.bottom,
            };
            let about = match cx.info.slot.map(|s| s.letter()) {
                Some(slot) => tr!(
                    l,
                    "仅显示当前槽位 {slot} 在 super 中的分区，挂载后均为只读。",
                    "僅顯示目前槽位 {slot} 在 super 中的分割區，掛載後均為唯讀。",
                    "Only the current slot {slot}'s partitions in super show here, all mounted read-only."
                ),
                None => l.pick(["仅显示 super 中的分区，挂载后均为只读。", "僅顯示 super 中的分割區，掛載後均為唯讀。", "Only partitions in super show here, all mounted read-only."]).into(),
            };
            widgets::footnote(&mut c, t, f, y, &about)
        }
    };

    Frame::new(t, fit(c, t, bottom), out, l.pick(texts::page::MOUNT), top)
}

/// Who made the recovery, as the About page names them under its version.
const AUTHOR: &str = "@ 2026 Kirk Lin & TKexig";

/// About this device, modelled on the stock「我的设备」: the wordmark, the
/// version and its author over a moving colour field, and the summary,
/// hardware and link cards scrolling over it. Only the cards are drawn
/// here; the canvas is clear wherever the field shows.
pub fn about(cx: &Ctx) -> Frame {
    let (t, f) = (cx.t, cx.f);
    let m = &t.m;
    // The page is drawn whole, under the field: laid out first to learn
    // how tall it comes to, then drawn that tall.
    let (_, _, bottom) = about_cards(cx, &mut Canvas::band(m.w, m.h * 3, 0..0, 0..0, Rgba { r: 0, g: 0, b: 0, a: 0 }));
    let mut c = Canvas::new(m.w, fitted(t, bottom));
    let (out, summary_top, bottom) = about_cards(cx, &mut c);
    let canvas = fit(c, t, bottom);
    let art = HeroArt {
        wordmark: widgets::wordmark(t, f, "Skyward", m.dp(233.7), m.dp(37.8)),
        version: widgets::version_line(t, f, RUNNING),
        author: widgets::author_line(t, f, AUTHOR),
        runs: canvas.runs(),
    };
    Frame { header: false, hero: Some(art), ..Frame::new(t, canvas, out, cx.state.lang.pick(texts::page::ABOUT), summary_top) }
}

/// The phone's processor by its marketing name in `l`, from the phone's
/// settings, or else by its cores.
fn soc(l: Lang, i: &DeviceInfo) -> Option<&str> {
    i.soc.as_ref().map(|n| l.among(n.hans.as_str(), n.hant.as_str(), n.en.as_str()))
}

/// The About page's cards on `c`: what touching them does, where the
/// first starts, and where the last ends.
fn about_cards(cx: &Ctx, c: &mut Canvas) -> (Vec<Hit>, f32, f32) {
    let (t, f, i, l) = (cx.t, cx.f, cx.info, cx.state.lang);
    let m = &t.m;
    let tone = Tone::hero(t);
    let mut out = Vec::new();

    let storage = fmt::storage_used(&i.storage).unwrap_or_else(|| fmt::capacity(i.storage.total));
    let mut facts = vec![Fact { title: l.pick(MODEL), value: i.model.clone(), link: false }, Fact { title: l.pick(["存储空间", "儲存空間", "Storage"]), value: storage, link: true }];
    if let Some(v) = &i.os_version {
        facts.push(Fact { title: l.pick(texts::specs::OS_VERSION), value: v.clone(), link: false });
    }
    if let Some(r) = &i.android {
        facts.push(Fact { title: l.pick(texts::specs::ANDROID_VERSION), value: r.clone(), link: false });
    }
    let summary = widgets::facts(c, t, f, m.dp(490.7), &facts, &tone);
    out.push(Hit { rect: summary.rows[1], action: Action::Open(Screen::Specs) });

    let processor = soc(l, i).map_or_else(|| fmt::cpu(l, &i.cpu), str::to_string);
    let mut specs = vec![(l.pick(texts::specs::PROCESSOR), processor), (l.pick(texts::specs::RAM), fmt::ram(i.ram_kib))];
    if let Some(design) = i.battery.as_ref().and_then(|b| b.design) {
        specs.push((l.pick(texts::specs::BATTERY_CAPACITY), fmt::mah(design)));
    }
    if let Some(d) = i.screen {
        specs.push((l.pick(texts::specs::SCREEN_SIZE), fmt::inches(d)));
    }
    if let Some(p) = i.panel {
        specs.push((l.pick(texts::specs::RESOLUTION), fmt::panel(p)));
    }
    let y = widgets::spec_card(c, t, f, summary.bottom + m.card_gap, &fmt::model_lines(&i.model), &specs, &tone);
    let links = widgets::list_toned(c, t, f, y + m.card_gap, &[Row::new(l.pick(texts::page::SPECS)), Row::new(l.pick(texts::page::LOGS))], &tone);
    out.extend(hits(&links.rows, &[Action::Open(Screen::Specs), Action::Open(Screen::Logs)]));
    (out, summary.rows[0].y, links.bottom)
}

/// Every reading the recovery has, modelled on the stock「全部参数与信息」.
pub fn specs(cx: &Ctx) -> Frame {
    let (t, f, i, l) = (cx.t, cx.f, cx.info, cx.state.lang);
    let gap = t.m.card_gap;
    let mut c = long_page(cx);
    let mut out = Vec::new();
    // This page leaves more room under the title than the settings pages.
    let top = t.m.dp(188.0);

    let mut grid = vec![Spec { icon: Outline::Phone, label: l.pick(texts::specs::MODEL), value: fmt::model_lines(&i.model) }];
    if let Some(design) = i.battery.as_ref().and_then(|b| b.design) {
        grid.push(Spec { icon: Outline::Battery, label: l.pick(texts::specs::BATTERY_CAPACITY), value: fmt::mah(design) });
    }
    let cpu = match soc(l, i) {
        Some(soc) => format!("{soc}\n{}", fmt::cpu(l, &i.cpu)),
        None => fmt::cpu(l, &i.cpu),
    };
    grid.push(Spec { icon: Outline::Chip, label: l.pick(texts::specs::PROCESSOR), value: cpu });
    grid.push(Spec { icon: Outline::Memory, label: l.pick(texts::specs::RAM), value: fmt::ram(i.ram_kib) });
    if let Some(p) = i.panel {
        grid.push(Spec { icon: Outline::Resolution, label: l.pick(texts::specs::RESOLUTION), value: fmt::panel(p) });
    }
    if let Some(d) = i.screen {
        grid.push(Spec { icon: Outline::Screen, label: l.pick(texts::specs::SCREEN_SIZE), value: fmt::inches(d) });
    }
    grid.push(Spec { icon: Outline::Storage, label: l.pick(texts::specs::STORAGE_CAPACITY), value: fmt::capacity(i.storage.total) });
    grid.push(Spec { icon: Outline::Tag, label: l.pick(texts::specs::CODENAME), value: i.codename.clone() });
    let mut y = widgets::spec_grid(&mut c, t, f, top, &grid);

    if let Some(v) = &i.os_version {
        y = widgets::info_card(&mut c, t, f, y + gap, &[(l.pick(texts::specs::OS_VERSION), v.clone())]);
    }
    if let Some(r) = &i.android {
        y = widgets::info_card(&mut c, t, f, y + gap, &[(l.pick(texts::specs::ANDROID_VERSION), fmt::android(l, r, i.security_patch.as_ref()))]);
    }
    y = widgets::info_card(
        &mut c,
        t,
        f,
        y + gap,
        &[(l.pick(["Recovery 版本", "Recovery 版本", "Recovery version"]), format!("Skyward {RUNNING}")), (l.pick(["内核版本", "核心版本", "Kernel version"]), i.kernel.clone())],
    );

    let mut boot = Vec::new();
    if let Some(s) = i.slot {
        boot.push((l.pick(texts::slot::CURRENT), s.letter().to_string()));
        boot.push((l.pick(texts::page::UPDATE), fmt::snapshot(l, &i.snapshot).to_string()));
    }
    if let Some(unlocked) = i.boot.unlocked {
        let state = if unlocked { texts::specs::UNLOCKED } else { texts::specs::LOCKED };
        boot.push(("Bootloader", l.pick(state).to_string()));
    }
    if let Some(v) = &i.boot.verified {
        boot.push((l.pick(texts::specs::VERIFIED_BOOT), fmt::verified(l, v)));
    }
    boot.push((l.pick(texts::specs::DATA_PARTITION), fmt::data(l, &cx.state.data).to_string()));
    y = widgets::info_card(&mut c, t, f, y + gap, &boot);

    if let Some(b) = &i.battery {
        let mut rows = vec![(l.pick(texts::specs::LEVEL), fmt::level(l, b))];
        if let Some(v) = b.temp {
            rows.push((l.pick(texts::specs::TEMPERATURE), fmt::celsius(v)));
        }
        if let Some(v) = fmt::wear(l, b) {
            rows.push((l.pick(texts::specs::HEALTH), v));
        }
        if let Some(v) = b.cycles {
            rows.push((l.pick(texts::specs::CHARGE_CYCLES), texts::specs::cycles(l, v)));
        }
        if let Some(v) = b.voltage {
            rows.push((l.pick(texts::specs::VOLTAGE), fmt::volts(v)));
        }
        y = widgets::info_card(&mut c, t, f, y + gap, &rows);
    }

    let total = fmt::capacity(i.storage.total);
    let storage = match i.storage.free.map(fmt::gb) {
        Some(free) => texts::specs::free_of(l, free, total),
        None => texts::specs::total_unmounted(l, total),
    };
    y = widgets::info_card(&mut c, t, f, y + gap, &[(l.pick(["机身存储", "內建儲存空間", "Internal storage"]), storage)]);
    let links = widgets::list(&mut c, t, f, y + gap, &[Row::new(l.pick(LICENSE)).trail(Trail::Value("GPL-3.0")), Row::new(l.pick(FONTS)).trail(Trail::Value("MiSans"))]);
    out.extend(hits(&links.rows, &[Action::Open(Screen::License), Action::Open(Screen::FontLicense)]));
    Frame::new(t, fit(c, t, links.bottom), out, l.pick(texts::page::SPECS), top)
}

/// The data partition and its users' own files. The page opens the
/// partition as it shows, which is how it learns each user's screen lock;
/// a user whose files wait on one is asked with the control that lock is
/// answered with, from the start.
pub fn decrypt(cx: &Ctx) -> Frame {
    let (c, l) = (page(cx), cx.state.lang);
    match &cx.state.data {
        Data::Locked { user, keys, checking } => code(cx, c, *user, *keys, *checking),
        // Without a lock kept, the page the start opens on is the one to type
        // it into: nothing turns while the phone's own parts come up, and the
        // keyboard takes nothing until the lock is known.
        Data::Closed | Data::Opening if !cx.state.remember => code(cx, c, 0, Keys::Digits, true),
        Data::Closed | Data::Opening => data_status(
            cx,
            c,
            Icon::Lock,
            l.pick(["解密 data 分区", "解密 data 分割區", "Decrypt the data partition"]),
            Some(if cx.state.prepared {
                l.pick(["正在读取锁屏方式", "正在讀取螢幕鎖定方式", "Reading the screen lock"])
            } else {
                l.pick(["正在准备分区与服务", "正在準備分割區與服務", "Bringing the phone's own parts up"])
            }),
            None,
            true,
        ),
        Data::Open => {
            let t = cx.t;
            let m = &t.m;
            let mut frame = data_status(
                cx,
                c,
                Icon::Folder,
                l.pick(["data 分区已解密", "data 分割區已解密", "The data partition is decrypted"]),
                Some(l.pick(["你可以在「文件管理」中查看本机文件", "你可以在「檔案管理」中查看本機檔案", "You can see the phone's files in File manager"])),
                None,
                false,
            );
            // The lock is shown here too, and dropped from here: once it is
            // kept the decrypt page opens straight into the menu, so this is
            // where it is come back to in order to stop keeping it.
            let y = m.dp(150.0) + m.dp(98.0) + m.dp(52.0);
            let (kept, not_kept) = (l.pick(["已记住密码", "已記住密碼", "Screen lock kept"]), l.pick(["未记住密码", "未記住密碼", "Screen lock not kept"]));
            let row = widgets::list(
                &mut frame.canvas,
                t,
                cx.f,
                y,
                &[Row::new(match cx.state.remember {
                    true => kept,
                    false => not_kept,
                })
                .icon(if cx.state.remember { PURPLE } else { SLATE }, Icon::Lock)
                .trail(Trail::Switch(cx.state.remember))],
            );
            frame.hits.extend(hits(&row.rows, &[Action::Toggle(Switch::Remember)]));
            frame
        }
        Data::Failed(why) => data_status(cx, c, Icon::Lock, l.pick(["未能打开 data 分区", "無法開啟 data 分割區", "Couldn't open the data partition"]), None, Some(why), false),
        Data::Erased => data_status(cx, c, Icon::Wipe, l.pick(DATA_ERASED), Some(l.pick(["下次开机时将重新创建", "下次開機時將重新建立", "It's made anew when the phone next starts"])), None, false),
    }
}

/// How far apart, in dp, the lines of the Decrypt page's title are where
/// it takes more than one.
const TITLE_PITCH: f32 = 32.0;

/// Where the partition stands, under its sign: a line on it, or why it
/// did not open and a way to try again; then the way off the page.
fn data_status(cx: &Ctx, mut c: Canvas, icon: Icon, title: &str, note: Option<&str>, why: Option<&str>, busy: bool) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let m = &t.m;
    let x = m.w as f32 / 2.0;
    let sign_cy = m.dp(150.0);
    c.fill_circle(x, sign_cy, m.dp(30.0), t.pal.card);
    icons::glyph(&mut c, icon, x, sign_cy, m.dp(40.0), t.pal.accent);
    // A ring about the sign that turns while the partition is being opened --
    // or while the page waits for the phone's own parts, which the opening
    // needs -- so that a page that is waiting has something moving on it.
    if busy {
        let r = m.dp(48.0);
        let start = -std::f32::consts::FRAC_PI_2 + cx.state.data_phase * std::f32::consts::TAU;
        c.stroke_circle(x, sign_cy, r, m.dp(1.5), t.pal.card);
        for k in 0..4 {
            let a = 0.5 + 0.5 * (k as f32 / 4.0);
            let color = crate::color::mix(t.pal.bg, t.pal.accent, a * 0.55);
            c.stroke_arc(x, sign_cy, r, start - 0.14 * (4 - k) as f32, 0.15, m.dp(2.5), color);
        }
        c.stroke_arc(x, sign_cy, r, start, 0.9, m.dp(2.5), t.pal.accent);
    }
    c.text_center(&f.heading, title, x, sign_cy + m.dp(64.0), m.dp(24.0), t.pal.text);
    if let Some(note) = note {
        c.text_center(&f.body, note, x, sign_cy + m.dp(118.0), m.sub_px, t.pal.subtitle);
    }
    // No bar: the opening says nothing of how far along it is, and one drawn to
    // a guess of its length only ever disagreed with the work (2026-11-16).
    // The turning ring over the sign says the page is waiting, and that is all
    // that is known.
    let mut out = Vec::new();
    if let Some(why) = why {
        widgets::paragraph(&mut c, t, f, sign_cy + m.dp(88.0), why);
        let again = widgets::button(&mut c, t, f, m.h as f32 * 0.62, l.pick(texts::common::TRY_AGAIN));
        out.push(Hit { rect: again, action: Action::OpenData });
    }
    let leave_cy = m.h as f32 - m.dp(64.0);
    let leave = l.pick(if matches!(cx.state.data, Data::Open | Data::Erased) { texts::common::DONE } else { texts::decrypt::NOT_NOW });
    c.text_center(&f.heading, leave, x, leave_cy, m.row_title_px, t.pal.accent);
    out.push(Hit { rect: Rect::new(x - m.dp(80.0), leave_cy - m.dp(24.0), m.dp(160.0), m.dp(48.0)), action: Action::Skip });
    Frame { header: false, ..Frame::new(t, c, out, l.pick(texts::page::DECRYPT), 0.0) }
}

/// A user's own files wait on their screen lock: the control it is
/// answered with, which takes nothing while what was entered is checked.
fn code(cx: &Ctx, mut c: Canvas, user: u32, keys: Keys, checking: bool) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let x = m.w as f32 / 2.0;
    let mut out = Vec::new();
    let title = match (keys == Keys::Pattern, user) {
        (true, 0) => l.pick(texts::decrypt::DRAW_PATTERN).to_string(),
        (true, user) => texts::decrypt::draw_pattern_of(l, user),
        (false, 0) => l.pick(texts::decrypt::ENTER_LOCK).to_string(),
        (false, user) => texts::decrypt::enter_lock_of(l, user),
    };
    let note = l.pick(if checking { texts::decrypt::CHECKING } else { texts::decrypt::TO_SEE_FILES });
    let room = m.w as f32 - m.title_x * 2.0;
    let more = widgets::centred(&mut c, &f.heading, &title, x, m.dp(130.0), m.dp(24.0), m.dp(TITLE_PITCH), t.pal.text, room);
    widgets::centred(&mut c, &f.body, note, x, m.dp(166.0) + more, m.sub_px, m.sub_pitch, t.pal.subtitle, room);
    let mut pattern = None;
    let bottom = match keys {
        Keys::Letters => {
            let top = m.h as f32 - widgets::keyboard_height(t);
            out.extend(crate::ime::bar(&mut c, t, f, &s.ime, text_floor(t)));
            let caps = widgets::keyboard(&mut c, t, f, l, top, s.layer, s.key_held(), l.pick(texts::page::DECRYPT), s.ime.enabled);
            if !checking {
                out.extend(caps.into_iter().map(|(rect, cap)| Hit { rect, action: Action::Type(cap) }));
            }
            top
        }
        Keys::Pattern => {
            let span = m.dp(300.0);
            let cy = m.dp(300.0) + span / 2.0;
            widgets::pattern_lock(&mut c, t, x, cy, span, &s.pattern);
            if !checking {
                out.push(Hit { rect: Rect::new(x - span, cy - span, span * 2.0, span * 2.0), action: Action::Pattern });
                pattern = Some((x, cy, span));
            }
            m.h as f32 - m.dp(26.0)
        }
        Keys::Digits => {
            let top = m.h as f32 - widgets::keypad_height(t);
            let caps = widgets::keypad(&mut c, t, f, l, top, s.key_held());
            if !checking {
                out.extend(caps.into_iter().map(|(rect, cap)| Hit { rect, action: Action::Type(cap) }));
            }
            top
        }
    };
    // The dots are up under the words, and what is kept is said below them; a
    // pattern is its own showing of what has been drawn.
    if keys != Keys::Pattern {
        widgets::code_dots(&mut c, t, x, m.dp(212.0), s.code.chars().count());
    }
    let skip_cy = bottom - m.dp(34.0);
    c.text_center(&f.heading, l.pick(texts::decrypt::NOT_NOW), x, skip_cy, m.row_title_px, t.pal.accent);
    out.push(Hit { rect: Rect::new(x - m.dp(80.0), skip_cy - m.dp(24.0), m.dp(160.0), m.dp(48.0)), action: Action::Skip });
    // Once something has been typed, it can be kept for the starts that
    // follow: the risk of keeping it is put before it when it is.
    if !s.code.is_empty() && !checking {
        // Under the dots, clear of the pattern where that is what answers.
        let row_y = match keys {
            Keys::Pattern => m.dp(636.0),
            _ => m.dp(258.0),
        };
        // The switch is the state itself: taken across, the lock is kept; left
        // as it is, it is not, and the words say which of the two it is.
        // A pill about what is written, not a row across the page: the dots
        // above are small, and a card from edge to edge would drown them.
        let kept = l.pick(["已记住密码", "已記住密碼", "Screen lock kept"]);
        let not_kept = l.pick(["未记住密码", "未記住密碼", "Screen lock not kept"]);
        let word = match s.remember {
            true => kept,
            false => not_kept,
        };
        let px = m.sub_px;
        let pad = m.dp(18.0);
        let gap = m.dp(12.0);
        let pill_h = m.dp(48.0);
        let text = c.text_width(&f.body, word, px);
        let total = pad + text + gap + m.switch_w + pad;
        let left = x - total / 2.0;
        c.fill_round_rect(left, row_y, total, pill_h, pill_h / 2.0, t.pal.card);
        c.text_center(&f.body, word, left + pad + text / 2.0, row_y + pill_h / 2.0, px, t.pal.text);
        widgets::switch(&mut c, t, left + total - pad, row_y + pill_h / 2.0, s.remember);
        out.push(Hit { rect: Rect::new(left, row_y, total, pill_h), action: Action::Toggle(Switch::Remember) });
    }
    Frame { header: false, pattern, ..Frame::new(t, c, out, l.pick(DECRYPT), 0.0) }
}

/// Reboot target picker with a swipe-to-confirm control.
pub fn reboot(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let mut y = widgets::section_first(&mut c, t, f, top, l.pick(texts::reboot::TO));
    let rows: Vec<Row> = REBOOT_TARGETS
        .iter()
        .enumerate()
        .map(|(k, &(title, sub))| {
            let r = Row::new(l.pick(title)).trail(Trail::Check(s.reboot == k));
            match sub {
                Some(text) => r.subtitle(l.pick(text)),
                None => r,
            }
        })
        .collect();
    let targets = widgets::list(&mut c, t, f, y, &rows);
    let picks: Vec<Action> = (0..TARGETS.len()).map(Action::RebootTarget).collect();
    out.extend(hits(&targets.rows, &picks));
    y = targets.bottom;
    if let Some(slot) = cx.info.slot {
        let next = match &s.boot_slots {
            Some(Ok(slots)) if slots.active == slot => texts::reboot::current(l, slot.letter()),
            Some(Ok(slots)) => slots.active.letter().to_string(),
            Some(Err(_)) => l.pick(texts::common::UNREADABLE).to_string(),
            None => String::new(),
        };
        y = widgets::section(&mut c, t, f, y, l.pick(texts::slot::TITLE));
        let slots = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(texts::reboot::NEXT_BOOT)).trail(Trail::Value(&next))]);
        out.extend(hits(&slots.rows, &[Action::Open(Screen::Slot)]));
        y = slots.bottom;
    }
    let (label, danger) = confirm_label(Confirm::Reboot, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Reboot) });
    Frame::new(t, c, out, l.pick(texts::page::REBOOT), top)
}

/// The work going on now, and where each stands, with a row that opens the
/// page it is shown on.
///
/// The work has pages of its own, and those pages are where it is watched:
/// this page is the one place that says what is running at all, so nothing
/// started and then left is forgotten behind a page that is no longer open.
pub fn tasks(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    // Each row says what it is, where it stands, and which page shows it. The
    // words are kept beside the rows so each subtitle outlives the row it is
    // set in.
    let mut said: Vec<String> = Vec::new();
    let mut opens: Vec<Action> = Vec::new();
    if !matches!(s.install, Install::Idle) {
        said.push(install_stage(cx));
        opens.push(Action::Open(Screen::Installing));
    }
    if let Some(job) = s.backup_job.as_ref() {
        said.push(backup_stage(cx, job));
        opens.push(Action::Open(Screen::BackupJob));
    }
    if let Some(job) = s.image_job.as_ref().filter(|j| j.outcome.is_none()) {
        let share = match job.of {
            0 => 0.0,
            of => job.done.min(of) as f32 / of as f32,
        };
        said.push(format!("{} · {}%", job.partition, (share * 100.0).round() as u32));
        opens.push(Action::Open(Screen::ImageBackup));
    }
    if s.job.is_some() {
        said.push(l.pick(["正在复制、移动或删除", "正在複製、移動或刪除", "Copying, moving or deleting"]).to_string());
        opens.push(Action::Open(Screen::Files));
    }
    if s.part_work.as_ref().is_some_and(|w| w.outcome.is_none()) {
        said.push(l.pick(["正在处理分区", "正在處理分割區", "Working on a partition"]).to_string());
        opens.push(Action::Open(Screen::PartWork));
    }
    if s.relabel.as_ref().is_some_and(|j| j.outcome.is_none()) {
        said.push(l.pick(["正在写入文件标签", "正在寫入檔案標籤", "Writing the files' labels"]).to_string());
        opens.push(Action::Open(Screen::Relabel));
    }
    if let Some(job) = s.download.as_ref().filter(|j| j.outcome.is_none()) {
        said.push(match job.progress {
            Some(Progress::Downloading { done, total }) => format!("{} / {}", fmt::bytes(done), fmt::bytes(total)),
            Some(Progress::Checking { done, total }) => format!("{} / {}", fmt::bytes(done), fmt::bytes(total)),
            None => l.pick(["正在连接", "正在連線", "Connecting"]).to_string(),
        });
        opens.push(Action::Open(Screen::Update));
    }
    let names: Vec<&str> = opens
        .iter()
        .map(|action| match action {
            Action::Open(Screen::Installing) => l.pick(["安装", "安裝", "Install"]),
            Action::Open(Screen::BackupJob) => l.pick(["备份或恢复", "備份或還原", "Backup or restore"]),
            Action::Open(Screen::ImageBackup) => l.pick(IMAGE_BACKUP),
            Action::Open(Screen::Files) => l.pick(["文件", "檔案", "Files"]),
            Action::Open(Screen::PartWork) => l.pick(["分区", "分割區", "Partitions"]),
            Action::Open(Screen::Relabel) => l.pick(RELABEL),
            _ => l.pick(["下载", "下載", "Download"]),
        })
        .collect();
    let rows: Vec<Row> = names.iter().zip(&said).map(|(name, said)| Row::new(name).subtitle(said).trail(Trail::Arrow)).collect();
    let y = match rows.is_empty() {
        true => {
            let y = widgets::section_first(&mut c, t, f, top, l.pick(TASKS));
            let note = l.pick(["现在没有正在进行的工作。", "現在沒有正在進行的工作。", "Nothing is running now."]);
            widgets::footnote(&mut c, t, f, y, note)
        }
        false => {
            let y = widgets::section_first(&mut c, t, f, top, l.pick(TASKS));
            let list = widgets::list(&mut c, t, f, y, &rows);
            out.extend(hits(&list.rows, &opens));
            list.bottom
        }
    };
    Frame::new(t, fit(c, t, y), out, l.pick(TASKS), top)
}

/// Where an install stands, in a few words.
fn install_stage(cx: &Ctx) -> String {
    let (s, l) = (cx.state, cx.state.lang);
    match &s.install {
        Install::Idle => String::new(),
        Install::Starting => l.pick(["正在准备", "正在準備", "Getting ready"]).to_string(),
        Install::Opening => l.pick(["正在打开数据分区", "正在開啟資料分割區", "Opening the data partition"]).to_string(),
        Install::Locked(_) => l.pick(["等待解锁", "等待解鎖", "Waiting for the lock"]).to_string(),
        Install::Waiting => l.pick(["等待选择安装包", "等待選擇安裝檔", "Waiting for a package"]).to_string(),
        Install::Checking { .. } => l.pick(["正在检查安装包", "正在檢查安裝檔", "Checking the package"]).to_string(),
        Install::Choosing(_) | Install::Confirming { .. } | Install::ConfirmingScript => l.pick(["等待确认", "等待確認", "Waiting to be agreed to"]).to_string(),
        Install::NotSecondSystem(_) => l.pick(["此包不是双系统包", "此包不是雙系統包", "Not a second system's package"]).to_string(),
        Install::Updating { .. } => l.pick(["正在安装系统更新", "正在安裝系統更新", "Installing the update"]).to_string(),
        Install::Flashing { .. } => l.pick(["正在刷入分区镜像", "正在刷入分割區映像檔", "Flashing an image"]).to_string(),
        Install::Scripting { .. } => l.pick(["正在运行安装脚本", "正在執行安裝指令碼", "Running the install script"]).to_string(),
        _ => l.pick(["正在进行", "正在進行", "Under way"]).to_string(),
    }
}

/// Where a backup or restore stands, in a few words.
fn backup_stage(cx: &Ctx, job: &BackupJob) -> String {
    let l = cx.state.lang;
    match &job.outcome {
        Some(_) => l.pick(["已完成", "已完成", "Done"]).to_string(),
        None => match &job.at {
            Some(at) => {
                let share = match at.of {
                    0 => 0.0,
                    of => at.done.min(of) as f32 / of as f32,
                };
                let part = job.plan.get(at.item).map(|(name, _)| name.as_str()).unwrap_or("");
                format!("{part} · {}%", (share * 100.0).round() as u32)
            }
            None => l.pick(["正在进行", "正在進行", "Under way"]).to_string(),
        },
    }
}

/// Turning the phone off: what is still going that turning it off would cut
/// short, and a swipe to turn it off.
///
/// A page of its own rather than a choice on the reboot page: a phone is
/// turned off at the end of a session, and the work that a session leaves
/// running -- an install, a backup, an image being written -- is what a
/// swipe here would cut in half.
pub fn power_off(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    // The work that would be cut short, named as its own page names it.
    let mut going: Vec<&'static str> = Vec::new();
    if !matches!(s.install, Install::Idle) {
        going.push(l.pick(["安装", "安裝", "An install"]));
    }
    if s.backup_job.as_ref().is_some_and(|j| j.outcome.is_none()) {
        going.push(l.pick(["备份或恢复", "備份或還原", "A backup or restore"]));
    }
    if s.image_job.as_ref().is_some_and(|job| job.outcome.is_none()) {
        going.push(l.pick(["镜像的写入", "映像檔的寫入", "An image being written"]));
    }
    if s.job.is_some() {
        going.push(l.pick(["复制或删除文件", "複製或刪除檔案", "Files being copied or deleted"]));
    }
    if s.part_work.as_ref().is_some_and(|w| w.outcome.is_none()) {
        going.push(l.pick(["分区上的操作", "分割區上的操作", "Work on a partition"]));
    }
    if s.relabel.as_ref().is_some_and(|job| job.outcome.is_none()) {
        going.push(l.pick(["写入文件标签", "寫入檔案標籤", "Files being relabelled"]));
    }
    let y = match going.is_empty() {
        true => {
            let y = widgets::section_first(&mut c, t, f, top, l.pick(["关机", "關機", "Power off"]));
            let note = l.pick([
                "关机后手机将完全断电。按电源键可以重新开机，开机后直接进入系统。",
                "關機後手機將完全斷電。按電源鍵可以重新開機，開機後直接進入系統。",
                "The phone goes fully off. The power key turns it back on, into the system.",
            ]);
            widgets::footnote(&mut c, t, f, y, note)
        }
        false => {
            // The work is said first: a phone turned off halfway through an
            // install is a phone that may not start again.
            let y = widgets::section_first(&mut c, t, f, top, l.pick(["正在进行", "正在進行", "Still going"]));
            let rows: Vec<Row> = going.iter().map(|what| Row::new(what).tint(t.pal.danger)).collect();
            let list = widgets::list(&mut c, t, f, y, &rows);
            let note = l.pick([
                "关机将中断这些操作，未完成的部分可能让手机无法正常启动。请先返回等待它们完成。",
                "關機將中斷這些操作，未完成的部分可能讓手機無法正常啟動。請先返回等待它們完成。",
                "Turning the phone off cuts these short, and what is left half done may keep it from starting. Go back and let them finish.",
            ]);
            widgets::footnote(&mut c, t, f, list.bottom, note)
        }
    };
    let (label, danger) = confirm_label(Confirm::PowerOff, s);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::PowerOff) });
    Frame::new(t, c, out, l.pick(POWER_OFF), top)
}

/// What the charger is doing: the power it delivers, the protocol it
/// negotiated, and the battery it fills.
///
/// This page reads and does not set. On this phone the nodes that would hold
/// a charge limit are policy votes rather than figures, and a number written
/// to them is refused, so the recovery cannot change how fast the phone
/// charges — the kernel and the charger settle that between them before the
/// page is ever drawn. What the page can do is say what they settled on,
/// which is what a recovery is asked for when a charge looks slow.
pub fn charge(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let ch = &s.charge;
    let b = cx.info.battery.as_ref();

    // What the charger is giving, and the most it has given this charge: the
    // two numbers that say how fast this charge is running.
    let now = match ch.now {
        Some(tenths) => fmt::watts(tenths),
        None if b.is_some_and(|b| b.plugged) => l.pick(["读取中", "讀取中", "Reading"]).to_owned(),
        None => l.pick(["未接充电器", "未接充電器", "No charger"]).to_owned(),
    };
    let most = match ch.most {
        Some(tenths) => fmt::watts(tenths),
        None => "\u{2014}".to_owned(),
    };
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["现在", "現在", "Now"]));
    let rows = [Row::new(&now).icon(BLUE, Icon::Battery).trail(Trail::Value(&most))];
    let list = widgets::list(&mut c, t, f, y, &rows);
    let mut y = list.bottom;

    // How fast the phone is let to charge, chosen before the charger and the
    // battery are told of: it is the one thing on this page that changes what
    // the phone does rather than what it says. The programs that settle it are
    // started as the recovery starts, so a step chosen here reaches the phone
    // at the next start, and the page says so rather than letting the steps
    // look as if they had taken at once.
    {
        let steps: [(ChargeMode, Text, Text); 3] = [
            (
                ChargeMode::Plain,
                ["默认", "預設", "Default", "Default", "Default", "Default", "Default", "Default", "Default", "Default", "Default", "Default", "Default", "Default", "Default"],
                [
                    "不启动充电服务，内核把电流限在它给未认证电池的上限，5W+。",
                    "不啟動充電服務，核心把電流限在它給未認證電池的上限，5W+。",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                    "Neither service runs, so the kernel holds the current at the ceiling it sets an unchecked battery: 5W+.",
                ],
            ),
            (
                ChargeMode::Fast,
                ["快充", "快充", "Fast", "Fast", "Fast", "Fast", "Fast", "Fast", "Fast", "Fast", "Fast", "Fast", "Fast", "Fast", "Fast"],
                [
                    "启动小米充电服务，内核被告知电池已通过认证，10W+。",
                    "啟動小米充電服務，核心被告知電池已通過認證，10W+。",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                    "Xiaomi's charging service runs, so the kernel is told the battery passed its check: 10W+.",
                ],
            ),
            (
                ChargeMode::Fastest,
                ["超快充", "超快充", "Fastest", "Fastest", "Fastest", "Fastest", "Fastest", "Fastest", "Fastest", "Fastest", "Fastest", "Fastest", "Fastest", "Fastest", "Fastest"],
                [
                    "再启动小米的认证进程，充电器通过校验后可升压到 PPS 档，40W+。",
                    "再啟動小米的認證行程，充電器通過校驗後可升壓到 PPS 檔，40W+。",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                    "Xiaomi's secret keeper runs as well, so the charger may be verified and the protocol go on to the adjustable kind: 40W+.",
                ],
            ),
        ];
        y = widgets::section(&mut c, t, f, y, l.pick(["充电档位", "充電檔位", "Charge step"]));        let rows: Vec<Row> = steps
            .iter()
            .map(|(mode, name, detail)| Row::new(l.pick(*name)).subtitle(l.pick(*detail)).trail(Trail::Check(s.charge_mode == *mode)))
            .collect();
        let list = widgets::list(&mut c, t, f, y, &rows);
        y = list.bottom;
        // The step already chosen is not a hit: tapping it would do nothing,
        // and offering it as something to tap would say otherwise.
        for (i, (mode, _, _)) in steps.iter().enumerate() {
            if s.charge_mode != *mode {
                out.extend(hits(&list.rows[i..=i], &[Action::Toggle(Switch::ChargeStep(*mode))]));
            }
        }
        y = widgets::footnote(
            &mut c,
            t,
            f,
            y + t.m.dp(8.0),
            l.pick([
                "档位在下次开机时生效：它决定 recovery 启动哪些小米服务，而这些服务只在启动时决定一次。",
                "檔位在下次開機時生效：它決定 recovery 啟動哪些小米服務，而這些服務只在啟動時決定一次。",
                "A step takes effect the next time the recovery starts: it decides which of Xiaomi's services the recovery starts, and those are decided once, as it starts.",
            ]),
        );
    }

    // The thermal guard, above the charger it watches: how warm the battery
    // may get before the charge is stepped down, and starting the recovery
    // again to make the step take. It is neither offered nor obeyed on the
    // plain step, where nothing a charge draws can warm the battery enough to
    // matter, so the slider is drawn dimmed and takes no touch.
    {
        let guarded = s.charge_mode.guarded();
        y = widgets::section(&mut c, t, f, y + t.m.dp(8.0), l.pick(["温控", "溫控", "Thermal guard"]));
        let span = f32::from(THERMAL_LIMIT_MOST - THERMAL_LIMIT_LEAST);
        let level = (f32::from(s.thermal_limit - THERMAL_LIMIT_LEAST) / span).clamp(0.0, 1.0);
        // The limit is written into the label, since the card has no value of
        // its own: the switch below it is the guard itself.
        let label = format!(
            "{} · {} °C",
            l.pick(["电池温度上限", "電池溫度上限", "Battery temperature limit"]),
            s.thermal_limit
        );
        let below = Row::new(l.pick(["超出后降低充电档位", "超出後降低充電檔位", "Step the charge down when past"]))
            .trail(Trail::Switch(s.thermal_guard && guarded));
        let card = widgets::slider_card(&mut c, t, f, y + t.m.dp(8.0), Caption::Thermometer, &label, level, Some(below));
        // The limit is the same number whatever the step, so it is drawn as it
        // stands even where it cannot be changed: a slider drawn at its left
        // end would say the limit had been moved there.
        //
        // A limit is only worth setting while the guard that reads it is on,
        // so the slider takes no touch where the switch is off, or where the
        // step starts nothing the guard has any say over. The switch itself is
        // still the guard's to turn, wherever a step is guarded at all.
        let lit = guarded && s.thermal_guard;
        if !lit {
            // Laid over the whole card, the way a finger's press is laid over a
            // row it holds: it says the card takes no touch without hiding what
            // it holds.
            let top = y + t.m.dp(8.0);
            c.fill_round_rect(t.m.margin, top, t.m.w as f32 - t.m.margin * 2.0, card.bottom - top, t.m.card_radius, t.pal.row_press);
        } else {
            out.push(Hit { rect: card.track, action: Action::Slider(Slider::ThermalLimit) });
        }
        if guarded && let Some(r) = card.below {
            out.push(Hit { rect: r, action: Action::Toggle(Switch::ThermalGuard) });
        }
        y = card.bottom;
        y = widgets::footnote(
            &mut c,
            t,
            f,
            y + t.m.dp(8.0),
            if guarded {
                l.pick([
                    "开关和温度都是即时生效的：开启后温度才可调，调完就按新温度盯着，电池一到这个温度就把充电档位降一档，然后重启回 recovery 按新档位充电。默认档不启动任何服务，充电很慢，不会让电池过热，所以默认档下这一项不可调，也不生效。",
                    "開關和溫度都是即時生效的：開啟後溫度才可調，調完就按新溫度盯著，電池一到這個溫度就把充電檔位降一檔，然後重啟回 recovery 按新檔位充電。預設檔不啟動任何服務，充電很慢，不會讓電池過熱，所以預設檔下這一項不可調，也不生效。",
                    "The switch and the temperature both take at once: the limit may be set only while the guard is on, and it is watched as it is set, so a battery that reaches it has the charge stepped down there and then, starting the recovery again to charge at the new step. The default step starts no service at all, so its charge is slow enough not to overheat the battery: the guard is neither offered nor obeyed there.",
                ])
            } else {
                l.pick([
                    "默认档不启动任何服务，充电很慢，不会让电池过热，所以这一项不可调，也不生效。",
                    "預設檔不啟動任何服務，充電很慢，不會讓電池過熱，所以這一項不可調，也不生效。",
                    "The default step starts no service at all, so its charge is slow enough not to overheat the battery: this is neither offered nor obeyed there.",
                ])
            },
        );
    }

    // What the charger negotiated, and what it says it can give. These are
    // the kernel's own words, left as it writes them: a protocol name is a
    // name, and translating it would only make it harder to look up.
    let mut charger: Vec<(&str, String)> = Vec::new();
    if let Some(protocol) = &ch.protocol {
        charger.push((l.pick(["协议", "協定", "Protocol"]), protocol.clone()));
    }
    if let Some(rated) = ch.rated {
        charger.push((l.pick(["充电器标称", "充電器標稱", "Charger rated"]), fmt::watts(rated * 10)));
    }
    if let Some(uv) = ch.volts {
        charger.push((l.pick(["输入电压", "輸入電壓", "Input voltage"]), fmt::volts(uv)));
    }
    if let Some(ua) = ch.amps {
        charger.push((l.pick(["输入电流", "輸入電流", "Input current"]), fmt::amps(ua)));
    }
    if !charger.is_empty() {
        y = widgets::section(&mut c, t, f, y, l.pick(["充电器", "充電器", "Charger"]));
        y = widgets::info_card(&mut c, t, f, y, &charger);
    }

    // The battery being filled: what it is at, how warm it is, and what the
    // charge is doing to it. The warmth matters more here than anywhere else
    // on the pages, since a fast charge is what makes it.
    if let Some(b) = b {
        let mut cells = vec![(l.pick(["电量", "電量", "Level"]), fmt::level(l, b))];
        if let Some(v) = b.temp {
            cells.push((l.pick(["温度", "溫度", "Temperature"]), fmt::celsius(v)));
        }
        if let Some(w) = b.watts() {
            cells.push((l.pick(["电池功率", "電池功率", "Battery power"]), fmt::watts((w * 10.0).round() as u32)));
        }
        if let Some(v) = b.voltage {
            cells.push((l.pick(["电池电压", "電池電壓", "Battery voltage"]), fmt::volts(v)));
        }
        if let Some(v) = fmt::wear(l, b) {
            cells.push((l.pick(["健康", "健康狀態", "Health"]), v));
        }
        if let Some(v) = b.cycles {
            cells.push((l.pick(["充电循环", "充電循環", "Charge cycles"]), tr!(l, "{v} 次", "{v} 次", "{v}")));
        }
        y = widgets::section(&mut c, t, f, y, l.pick(["电池", "電池", "Battery"]));
        y = widgets::info_card(&mut c, t, f, y, &cells);
    }

    // What the page cannot do, said plainly, since a page about charging that
    // stayed silent would look as if it could make the phone charge faster.
    // The step above settles which of the phone's own programs run; the
    // protocol itself is still the charger's and the kernel's to settle.
    let note = l.pick([
        "档位只决定 recovery 启动哪些小米服务；能到多少功率仍由充电器与内核协商，并不保证达到某个数值。",
        "檔位只決定 recovery 啟動哪些小米服務；能到多少功率仍由充電器與核心協商，並不保證達到某個數值。",
        "A step settles which of Xiaomi's services the recovery runs: how much power comes of it is still the charger's and the kernel's to settle, and no particular figure is promised.",
    ]);
    y = widgets::footnote(&mut c, t, f, y + t.m.dp(10.0), note);

    Frame::new(t, fit(c, t, y), out, l.pick(CHARGE), top)
}

/// Installing, opened from the home page: the system on the phone now, and
/// where a package can come from.
pub fn install(cx: &Ctx) -> Frame {
    let (t, f, i, l) = (cx.t, cx.f, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut facts = Vec::new();
    if let Some(v) = &i.os_version {
        facts.push((l.pick(texts::install::CURRENT_SYSTEM), v.clone()));
    }
    if let Some(p) = &i.security_patch {
        facts.push((l.pick(texts::install::SECURITY_PATCH), fmt::patch(l, p)));
    }
    if let Some(s) = i.slot {
        facts.push((l.pick(texts::slot::CURRENT), texts::install::slot_detail(l, s.letter(), s.other().letter())));
    }
    let ways = l.pick(texts::install::WAYS);
    let y = if facts.is_empty() {
        widgets::section_first(&mut c, t, f, top, ways)
    } else {
        let y = widgets::section_first(&mut c, t, f, top, l.pick(texts::common::THIS_PHONE));
        let y = widgets::info_card(&mut c, t, f, y, &facts);
        widgets::section(&mut c, t, f, y, ways)
    };
    let mut rows = Vec::new();
    if let Some(note) = fmt::snapshot_note(l, &i.snapshot) {
        rows.push(Row::new(fmt::snapshot(l, &i.snapshot)).icon(ORANGE, Icon::Layers).subtitle(note).trail(Trail::None));
    }
    // Internal storage opens with the data partition, which the decrypt
    // page opens or tells why it does not.
    let data = &cx.state.data;
    let (storage, from_storage) = if data.storage_open() {
        (&["选择内部存储中的更新包或镜像", "選擇內部儲存空間中的更新檔或映像檔", "Choose a package or image in internal storage"][..], Action::Open(Screen::Choose(Place::Storage)))
    } else if *data == Data::Erased {
        (&texts::data::ERASED[..], Action::Open(Screen::Decrypt))
    } else {
        (&texts::decrypt::NEEDED[..], Action::Open(Screen::Decrypt))
    };
    // The drive's page waits for one to be plugged in, and says why one
    // that is cannot be read.
    let usb = match &cx.state.usb_drive {
        Some(UsbDrive::Shown) => texts::install::CHOOSE_ON_USB_DRIVE,
        Some(UsbDrive::Unreadable(_)) => texts::usb_drive::UNREADABLE,
        None => texts::usb_drive::WANTED,
    };
    rows.push(Row::new(install_title(l, Source::File(Place::Storage))).icon(BLUE, Icon::Phone).subtitle(l.pick(storage)));
    rows.push(Row::new(install_title(l, Source::File(Place::UsbDrive))).icon(PURPLE, Icon::UsbDrive).subtitle(l.pick(usb)));
    let sideload = l.pick(texts::install::SIDELOAD);
    rows.push(Row::new(install_title(l, Source::Computer)).icon(GREEN, Icon::Usb).subtitle(sideload));
    let methods = widgets::list(&mut c, t, f, y, &rows);
    let from_usb = Action::Open(Screen::Choose(Place::UsbDrive));
    let mut out = hits(&methods.rows[methods.rows.len() - 3..], &[from_storage, from_usb, Action::Sideload]);
    // A phone that can hold a second system installs one from here too.
    let what = l.pick(match cx.state.dual_boot {
        true => texts::install::WHAT_WITH_SECOND_SYSTEM,
        false => texts::install::WHAT,
    });
    let y = widgets::footnote(&mut c, t, f, methods.bottom, what);
    let y = widgets::section(&mut c, t, f, y, l.pick(texts::common::OPTIONS));
    let check = Row::new(l.pick(texts::install::VERIFY_SIGNATURES)).trail(Trail::Switch(cx.state.check_signature));
    let signature = widgets::list(&mut c, t, f, y, &[check]);
    out.extend(hits(&signature.rows, &[Action::Toggle(Switch::Signature)]));
    let mut y = widgets::footnote(&mut c, t, f, signature.bottom, l.pick(texts::install::VERIFY_SIGNATURES_NOTE));
    if cx.state.recovery_partitions {
        let row = Row::new(l.pick(texts::install::FLASH_SKYWARD_AGAIN)).trail(Trail::Switch(cx.state.keep_recovery));
        let options = widgets::list(&mut c, t, f, y + t.m.card_gap, &[row]);
        out.extend(hits(&options.rows, &[Action::Toggle(Switch::KeepRecovery)]));
        y = widgets::footnote(&mut c, t, f, options.bottom, l.pick(texts::install::FLASH_SKYWARD_AGAIN_NOTE));
    }
    Frame::new(t, fit(c, t, y), out, l.pick(texts::page::INSTALL), top)
}

/// How installing from `source` is named: on the install page, and as the
/// title of the page that installs.
fn install_title(l: Lang, source: Source) -> &'static str {
    use texts::install::title;
    l.pick(match source {
        Source::Computer => title::FROM_COMPUTER,
        Source::File(Place::Storage) => title::FROM_STORAGE,
        Source::File(Place::UsbDrive) => title::FROM_USB_DRIVE,
        Source::Opened { .. } => title::FROM_FILES,
        Source::Asked { security: false } => title::SYSTEM_UPDATE,
        Source::Asked { security: true } => title::SECURITY_UPDATE,
        Source::Rescue => title::RESCUE,
    })
}

/// What a place is called at the top of its folders.
fn place_name(l: Lang, place: Place) -> &'static str {
    l.pick(match place {
        Place::Storage => texts::place::STORAGE,
        Place::UsbDrive => texts::place::USB_DRIVE,
    })
}

/// What the package was found to be, at `y`: the install page says which
/// kind it is before anything is written, so that a package of one kind is
/// not taken for one of another.
fn kind_note(c: &mut Canvas, cx: &Ctx, y: f32, kind: PackageKind) -> f32 {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let what = l.pick(match kind {
        PackageKind::Update => ["识别为系统更新包", "辨識為系統更新檔", "Found to be a system update"],
        PackageKind::Script => ["识别为带安装脚本的刷机包", "辨識為帶安裝指令碼的刷機包", "Found to be a zip with an install script"],
        PackageKind::Image => ["识别为分区镜像", "辨識為分割區映像檔", "Found to be a partition image"],
        PackageKind::System => ["识别为第二系统的安装包", "辨識為第二系統的安裝檔", "Found to be a second system's package"],
    });
    widgets::footnote(c, t, f, y, what)
}

/// A one-line card on how things stand.
fn standing<'a>(title: &'a str, color: Rgba, icon: Icon, detail: &'a str) -> Row<'a> {
    Row::new(title).icon(color, icon).subtitle(detail).trail(Trail::None)
}

fn share(done: u64, total: u64) -> f32 {
    if total == 0 { 0.0 } else { (done as f64 / total as f64) as f32 }
}

fn percent(share: f32) -> u32 {
    (share * 100.0).floor() as u32
}

/// Where an update stands, and the share of its step done when known.
fn update_stage(l: Lang, stage: Option<&Stage>) -> (String, Option<f32>) {
    match stage {
        Some(Stage::Writing { done, total }) => {
            let s = share(*done, *total);
            (texts::installing::update_writing(l, percent(s)), Some(s))
        }
        Some(Stage::Verifying { index, count, partition }) => {
            (texts::installing::update_verifying(l, partition, index + 1, count), Some(*index as f32 / *count as f32))
        }
        Some(Stage::Finishing) => (l.pick(["第 3 步，共 3 步：完成安装", "第 3 步，共 3 步：完成安裝", "Step 3 of 3: finishing"]).into(), None),
        Some(Stage::Reflashing { done, total }) => (l.pick(REFLASHING).into(), Some(*done as f32 / *total as f32)),
        Some(Stage::Resizing | Stage::Flashing { .. } | Stage::Script { .. } | Stage::Signature { .. } | Stage::Allocating { .. } | Stage::Root { .. } | Stage::Storing) | None => (l.pick(GETTING_READY).into(), None),
    }
}

/// Where installing a system beside the installed one stands, and the share
/// of its step done when known.
fn system_stage(l: Lang, stage: Option<&Stage>) -> (String, Option<f32>) {
    match stage {
        Some(Stage::Allocating { done, total }) => {
            let s = share(*done, *total);
            (texts::installing::system_allocating(l, percent(s)), Some(s))
        }
        Some(Stage::Root { done, total }) => {
            let s = share(*done, *total);
            let said = match done * 2 <= *total {
                true => texts::installing::system_writing(l, percent(s)),
                false => texts::installing::system_verifying(l, percent(s)),
            };
            (said, Some(s))
        }
        Some(Stage::Storing) => (l.pick(texts::installing::SYSTEM_STORING).into(), None),
        _ => (l.pick(texts::installing::GETTING_READY).into(), None),
    }
}

/// What the card says while an install script runs, and how far it is: the
/// last line the script showed and how far it says it is, then Skyward
/// going back into the recovery partitions it wrote over.
fn script_stage<'a>(l: Lang, stage: Option<&Stage>, said: &'a [String]) -> (&'a str, Option<f32>) {
    let running = l.pick(texts::installing::RUNNING_SCRIPT);
    let last = said.iter().rev().map(|line| line.trim()).find(|line| !line.is_empty()).unwrap_or(running);
    match stage {
        Some(Stage::Reflashing { done, total }) => (l.pick(texts::installing::REFLASHING), Some(*done as f32 / *total as f32)),
        Some(Stage::Script { thousandths }) => (last, Some(f32::from(*thousandths) / 1000.0)),
        _ => (last, None),
    }
}

/// How Skyward went back into the recovery partitions the install wrote
/// over, where it did.
fn reflashed(c: &mut Canvas, cx: &Ctx, y: f32) -> f32 {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    match &cx.state.reflash {
        None => y,
        Some(Reflash::Written(written)) => {
            let said = match written.as_slice() {
                [slot] => texts::installing::reflashed_one(l, slot.letter()),
                [a, b] => texts::installing::reflashed_two(l, a.letter(), b.letter()),
                other => unreachable!("a phone has two slots, and Skyward went back into {other:?}"),
            };
            widgets::footnote(c, t, f, y, &said)
        }
        Some(Reflash::Failed(why)) => {
            let y = widgets::footnote(c, t, f, y, l.pick(texts::installing::REFLASH_FAILED));
            widgets::footnote(c, t, f, y, why)
        }
    }
}

/// What the install script showed, where there is any: its heading, and
/// under it the lines as the page's long list, the last thing on the page.
fn script_output(c: &mut Canvas, cx: &Ctx, y: f32, rows: &mut Option<Rows>) -> f32 {
    let lines = &cx.state.script_output;
    if lines.is_empty() {
        return y;
    }
    let y = widgets::section(c, cx.t, cx.f, y, cx.state.lang.pick(texts::installing::SCRIPT_OUTPUT));
    *rows = Some(Rows { edges: widgets::output_edges(c, cx.t, cx.f, y, lines), action: None });
    y
}

/// The slots an image can be flashed to, each by its letter where the
/// slot the phone runs from is known.
fn pick_title(l: Lang, pick: Pick, current: Option<Slot>) -> String {
    match (pick, current) {
        (Pick::Current, Some(slot)) => texts::flash::current_slot(l, slot.letter()),
        (Pick::Current, None) => l.pick(texts::slot::CURRENT).into(),
        (Pick::Other, Some(slot)) => texts::flash::other_slot(l, slot.other().letter()),
        (Pick::Other, None) => l.pick(texts::flash::OTHER_SLOT_UNNAMED).into(),
        (Pick::Both, _) => l.pick(texts::flash::BOTH_SLOTS).into(),
    }
}

/// Installing a package step by step: receiving what the computer sends
/// with `adb sideload`, or reading the file chosen, then installing it.
pub fn installing(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let (mut out, mut rows) = (Vec::new(), None);
    let top = t.m.content_top;
    let computer = s.source == Source::Computer;
    let bottom = match &s.install {
        Install::Idle | Install::Starting | Install::Ending if computer => {
            let row = standing(l.pick(texts::installing::SWITCHING_USB), GREEN, Icon::Usb, l.pick(texts::installing::TO_SIDELOAD));
            widgets::list(&mut c, t, f, top, &[row]).bottom
        }
        Install::Idle | Install::Starting | Install::Ending if s.source == Source::Rescue => {
            let row = standing(l.pick(texts::installing::SWITCHING_USB), GREEN, Icon::Usb, l.pick(texts::installing::TO_RESCUE));
            widgets::list(&mut c, t, f, top, &[row]).bottom
        }
        Install::Idle | Install::Starting | Install::Ending => widgets::list(&mut c, t, f, top, &[Row::new(l.pick(READING_FILE)).icon(GREEN, Icon::Install).trail(Trail::None)]).bottom,
        Install::Opening => {
            let opening = Row::new(l.pick(texts::installing::OPENING_DATA)).icon(GREEN, Icon::Lock).trail(Trail::None);
            let y = widgets::list(&mut c, t, f, top, &[opening]).bottom;
            asked_package(&mut c, cx, y)
        }
        Install::Locked(_) => {
            let row = standing(l.pick(texts::installing::CANT_READ_YET), ORANGE, Icon::Lock, l.pick(texts::decrypt::NEEDED));
            let y = widgets::list(&mut c, t, f, top, &[row]).bottom;
            let open = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(texts::page::DECRYPT))]);
            out.extend(hits(&open.rows, &[Action::Open(Screen::Decrypt)]));
            let y = widgets::footnote(&mut c, t, f, open.bottom, l.pick(texts::installing::INSTALLS_ONCE_DECRYPTED));
            asked_package(&mut c, cx, y)
        }
        Install::Waiting if s.source == Source::Rescue => {
            let row = standing(l.pick(texts::installing::WAITING), GREEN, Icon::Usb, l.pick(texts::installing::CONNECT));
            let y = widgets::list(&mut c, t, f, top, &[row]).bottom;
            let y = widgets::section(&mut c, t, f, y, l.pick(RUN_ON_COMPUTER));
            let y = widgets::info_card(
                &mut c,
                t,
                f,
                y,
                &[
                    (
                        l.pick(["adb rescue install 文件名", "adb rescue install 檔案名稱", "adb rescue install <file>"]),
                        l.pick([
                            "将文件名替换为系统更新包、刷机包或分区镜像",
                            "將檔案名稱替換為系統更新檔、刷機包或分割區映像檔",
                            "With a system update package, flashable zip or partition image as <file>",
                        ])
                        .into(),
                    ),
                    ("adb rescue getprop", l.pick(["读取本机的版本和设备信息", "讀取本機的版本和裝置資訊", "Reads this phone's versions and device details"]).into()),
                    ("adb rescue wipe userdata", l.pick(["清除本机的全部数据", "清除本機的全部資料", "Wipes all of this phone's data"]).into()),
                ],
            );
            let note = l.pick([
                "需在电脑上安装 Android SDK Platform-Tools。返回后将退出救援模式，并将 USB 切换回 adb 调试。",
                "需在電腦上安裝 Android SDK Platform-Tools。返回後將結束救援模式，並將 USB 切換回 adb 偵錯。",
                "The computer needs Android SDK Platform-Tools. Going back leaves rescue mode and switches USB back to adb debugging.",
            ]);
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Waiting => {
            let row = standing(l.pick(texts::installing::WAITING), GREEN, Icon::Usb, l.pick(texts::installing::CONNECT));
            let y = widgets::list(&mut c, t, f, top, &[row]).bottom;
            let y = widgets::section(&mut c, t, f, y, l.pick(RUN_ON_COMPUTER));
            let y = widgets::info_card(
                &mut c,
                t,
                f,
                y,
                &[(
                    l.pick(["adb sideload 文件名", "adb sideload 檔案名稱", "adb sideload <file>"]),
                    l.pick([
                        "将文件名替换为系统更新包、刷机包或分区镜像，例如 boot.img",
                        "將檔案名稱替換為系統更新檔、刷機包或分割區映像檔，例如 boot.img",
                        "With a system update package, flashable zip or partition image as <file>, such as boot.img",
                    ])
                    .into(),
                )],
            );
            let note = l.pick([
                "需在电脑上安装 Android SDK Platform-Tools。返回后将停止等待，并将 USB 切换回 adb 调试。",
                "需在電腦上安裝 Android SDK Platform-Tools。返回後將停止等待，並將 USB 切換回 adb 偵錯。",
                "The computer needs Android SDK Platform-Tools. Going back stops waiting and switches USB back to adb debugging.",
            ]);
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Checking { stage: Some(Stage::Signature { done, total }), .. } => {
            let read = share(*done, *total);
            let detail = texts::installing::read(l, percent(read));
            let y = widgets::progress_card(&mut c, t, f, top, l.pick(texts::installing::VERIFYING_SIGNATURE), &detail, Some(read));
            let note = l.pick(match s.source {
                Source::Computer => [
                    "读取完成前，请不要断开数据线。返回将取消安装，并结束电脑上的 adb sideload。",
                    "讀取完成前，請不要拔除傳輸線。返回將取消安裝，並結束電腦上的 adb sideload。",
                    "Keep the cable in until it's read. Going back cancels the install and ends adb sideload on the computer.",
                ],
                Source::Rescue => [
                    "读取完成前，请不要断开数据线。返回将取消安装，并退出救援模式。",
                    "讀取完成前，請不要拔除傳輸線。返回將取消安裝，並結束救援模式。",
                    "Keep the cable in until it's read. Going back cancels the install and leaves rescue mode.",
                ],
                Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => ["返回将取消安装。", "返回將取消安裝。", "Going back cancels the install."],
                Source::File(Place::UsbDrive) | Source::Opened { drive: true } => {
                    ["读取完成前，请不要拔出 U 盘。返回将取消安装。", "讀取完成前，請不要拔出 USB 隨身碟。返回將取消安裝。", "Keep the USB drive in until it's read. Going back cancels the install."]
                }
            });
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Checking { size, .. } => {
            let detail = texts::installing::file_size(l, fmt::bytes(*size));
            let y = widgets::list(&mut c, t, f, top, &[standing(l.pick(texts::installing::READING_FILE), GREEN, Icon::Install, &detail)]).bottom;
            match s.source {
                Source::Computer | Source::Rescue => widgets::footnote(&mut c, t, f, y, l.pick(texts::installing::KEEP_CABLE)),
                Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => y,
                Source::File(Place::UsbDrive) | Source::Opened { drive: true } => widgets::footnote(&mut c, t, f, y, l.pick(texts::installing::KEEP_USB_DRIVE)),
            }
        }
        Install::Choosing(choice) => {
            let (name, holds) = fmt::image_kind(l, choice.kind);
            let detail = texts::installing::size_and_holds(l, fmt::bytes(choice.size), &holds);
            let y = widgets::list(&mut c, t, f, top, &[standing(name, GREEN, Icon::Install, &detail)]).bottom;
            let y = kind_note(&mut c, cx, y, PackageKind::Image);
            let y = widgets::section(&mut c, t, f, y, l.pick(["分区", "分割區", "Partition"]));
            let rows: Vec<Row> = choice.partitions.iter().enumerate().map(|(k, p)| Row::new(p.name.as_str()).subtitle(fmt::partition_role(l, p)).trail(Trail::Check(k == choice.partition))).collect();
            let parts = widgets::list(&mut c, t, f, y, &rows);
            let choose: Vec<Action> = (0..choice.partitions.len()).map(Action::FlashPartition).collect();
            out.extend(hits(&parts.rows, &choose));
            // A partition in a slot has one to pick, and a logical one is
            // the slot the phone runs from, which is shown with nothing to
            // pick; super is in no slot.
            let bottom = match choice.partitions[choice.partition].place {
                FlashPlace::Slotted => {
                    let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(["槽位", "槽位", "Slot"]));
                    let titles = Pick::ALL.map(|p| pick_title(l, p, i.slot));
                    let rows: Vec<Row> = Pick::ALL.iter().zip(&titles).map(|(p, title)| Row::new(title).trail(Trail::Check(*p == choice.pick))).collect();
                    let picks = widgets::list(&mut c, t, f, y, &rows);
                    out.extend(hits(&picks.rows, &Pick::ALL.map(Action::FlashSlots)));
                    picks.bottom
                }
                FlashPlace::Logical => {
                    let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(["槽位", "槽位", "Slot"]));
                    let title = pick_title(l, Pick::Current, i.slot);
                    widgets::list(&mut c, t, f, y, &[Row::new(&title).trail(Trail::Check(true))]).bottom
                }
                FlashPlace::Single => parts.bottom,
            };
            let (label, danger) = confirm_label(Confirm::Flash, cx.state);
            let track = widgets::confirm(&mut c, t, f, bottom + t.m.dp(28.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::Flash) });
            let note = l.pick(match s.source {
                Source::Computer => [
                    "刷入将覆盖分区中的原有内容。返回将取消刷入，并结束电脑上的 adb sideload。",
                    "刷入將覆寫分割區中的原有內容。返回將取消刷入，並結束電腦上的 adb sideload。",
                    "Flashing writes over what the partition holds. Going back cancels it and ends adb sideload on the computer.",
                ],
                Source::Rescue => [
                    "刷入将覆盖分区中的原有内容。返回将取消刷入，并退出救援模式。",
                    "刷入將覆寫分割區中的原有內容。返回將取消刷入，並結束救援模式。",
                    "Flashing writes over what the partition holds. Going back cancels it and leaves rescue mode.",
                ],
                Source::File(_) | Source::Opened { .. } | Source::Asked { .. } => {
                    ["刷入将覆盖分区中的原有内容。返回将取消刷入。", "刷入將覆寫分割區中的原有內容。返回將取消刷入。", "Flashing writes over what the partition holds. Going back cancels it."]
                }
            });
            widgets::footnote(&mut c, t, f, track.y + track.h, note)
        }
        Install::Confirming { version, wipe } => {
            use texts::installing::confirm;
            let (title, color, icon, detail, which) = if *wipe {
                let detail = match (version, s.cache_partition) {
                    (Some(v), true) => confirm::version_resets_with_cache(l, v),
                    (Some(v), false) => confirm::version_resets(l, v),
                    (None, true) => l.pick(confirm::UPDATE_RESETS_WITH_CACHE).into(),
                    (None, false) => l.pick(confirm::UPDATE_RESETS).into(),
                };
                (l.pick(confirm::WIPE_NEEDED).to_string(), ORANGE, Icon::Wipe, detail, Confirm::InstallWipe)
            } else {
                let title = match version {
                    Some(v) => confirm::version(l, v),
                    None => l.pick(confirm::UPDATE).into(),
                };
                let detail = match i.slot.map(|s| s.other().letter()) {
                    Some(slot) => confirm::to_slot(l, slot),
                    None => l.pick(confirm::STARTS_AFTER_REBOOT).into(),
                };
                (title, GREEN, Icon::Install, detail, Confirm::Install)
            };
            let y = widgets::list(&mut c, t, f, top, &[standing(&title, color, icon, &detail)]).bottom;
            let y = kind_note(&mut c, cx, y, PackageKind::Update);
            let (label, danger) = confirm_label(which, cx.state);
            let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(which) });
            widgets::footnote(&mut c, t, f, track.y + track.h, l.pick(texts::installing::CANCEL_WRITES_NOTHING))
        }
        Install::Updating { version, stage, overwriting } => {
            let title = match version {
                Some(v) => texts::installing::named(l, v),
                None => l.pick(texts::installing::UPDATE).into(),
            };
            let (detail, done) = update_stage(l, stage.as_ref());
            let y = widgets::progress_card(&mut c, t, f, top, &title, &detail, done);
            let until = until_installed(l, s.source);
            let note = match i.slot.map(|s| s.other().letter()) {
                Some(slot) => texts::installing::until_and_slot(l, until, slot),
                None => until.to_string(),
            };
            let y = widgets::footnote(&mut c, t, f, y, &note);
            if *overwriting {
                widgets::footnote(&mut c, t, f, y, l.pick(texts::installing::OVERWRITING))
            } else {
                y
            }
        }
        Install::Flashing { partition, stage } => {
            let (detail, done) = match stage {
                Some(Stage::Flashing { done, total }) => {
                    let s = share(*done, *total);
                    let detail = match done * 2 < *total {
                        true => texts::flash::writing(l, percent(s)),
                        false => texts::flash::verifying(l, percent(s)),
                    };
                    (detail, Some(s))
                }
                _ => (l.pick(texts::installing::GETTING_READY).into(), None),
            };
            let y = widgets::progress_card(&mut c, t, f, top, &texts::flash::flashing(l, partition), &detail, done);
            let keep = l.pick(match s.source {
                Source::Computer | Source::Rescue => ["刷入完成前，请不要断开数据线。", "刷入完成前，請不要拔除傳輸線。", "Keep the cable in until it's flashed."],
                Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => {
                    ["刷入完成前，请不要关机。", "刷入完成前，請不要關機。", "Don't power off until it's flashed."]
                }
                Source::File(Place::UsbDrive) | Source::Opened { drive: true } => ["刷入完成前，请不要拔出 U 盘。", "刷入完成前，請不要拔出 USB 隨身碟。", "Keep the USB drive in until it's flashed."],
            });
            widgets::footnote(&mut c, t, f, y, keep)
        }
        Install::SystemChoosing(choice) => {
            let detail = texts::installing::as_second_system(l, &choice.version);
            let y = widgets::list(&mut c, t, f, top, &[standing(&choice.name, PURPLE, Icon::Systems, &detail)]).bottom;
            let y = kind_note(&mut c, cx, y, PackageKind::System);
            let y = widgets::section(&mut c, t, f, y, l.pick(["分配空间", "分配空間", "Room for it"]));
            let sizes: Vec<String> = choice.sizes.iter().map(|&b| fmt::bytes(b)).collect();
            let least = l.pick(texts::installing::LEAST_ROOM);
            let rows: Vec<Row> = sizes
                .iter()
                .enumerate()
                .map(|(k, size)| {
                    let row = Row::new(size).trail(Trail::Check(k == choice.size));
                    if k == 0 { row.subtitle(least) } else { row }
                })
                .collect();
            let list = widgets::list(&mut c, t, f, y, &rows);
            let picks: Vec<Action> = (0..choice.sizes.len()).map(Action::SystemSize).collect();
            out.extend(hits(&list.rows, &picks));
            let y = widgets::footnote(&mut c, t, f, list.bottom, l.pick(texts::installing::ROOM_NOTE));
            let (label, danger) = confirm_label(Confirm::InstallSystem, cx.state);
            let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::InstallSystem) });
            widgets::footnote(&mut c, t, f, track.y + track.h, l.pick(texts::installing::CANCEL_WRITES_NOTHING))
        }
        Install::Systeming { name, stage } => {
            let (detail, done) = system_stage(l, stage.as_ref());
            let y = widgets::progress_card(&mut c, t, f, top, &texts::installing::named(l, name), &detail, done);
            widgets::footnote(&mut c, t, f, y, until_installed(l, s.source))
        }
        Install::ConfirmingScript => {
            let name = s.package_name.as_deref().unwrap_or(l.pick(texts::installing::FLASHABLE_ZIP));
            let detail = l.pick(texts::installing::SCRIPT_WARNING);
            let mut y = widgets::list(&mut c, t, f, top, &[standing(name, ORANGE, Icon::Install, detail)]).bottom;
            y = kind_note(&mut c, cx, y, PackageKind::Script);
            if let Some(note) = fmt::snapshot_script_note(l, &i.snapshot) {
                y = widgets::footnote(&mut c, t, f, y, note);
            }
            let (label, danger) = confirm_label(Confirm::RunScript, cx.state);
            let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::RunScript) });
            widgets::footnote(&mut c, t, f, track.y + track.h, l.pick(texts::installing::CANCEL_RUNS_NOTHING))
        }
        Install::NotSecondSystem(kind) => {
            let title = l.pick(["此包不是双系统包", "此包不是雙系統包", "This is not a second system's package"]);
            let detail = match kind {
                PackageKind::Update => l.pick([
                    "这是系统更新包，会安装到另一个槽位，不会装成第二系统。返回后可重新选择。",
                    "這是系統更新檔，會安裝到另一個槽位，不會裝成第二系統。返回後可重新選擇。",
                    "It is a system update, which installs to the other slot rather than beside the installed system. Go back to choose another.",
                ]),
                PackageKind::Script => l.pick([
                    "这是带安装脚本的刷机包，会运行其中的脚本改写分区，不会装成第二系统。返回后可重新选择。",
                    "這是帶安裝指令碼的刷機包，會執行其中的指令碼改寫分割區，不會裝成第二系統。返回後可重新選擇。",
                    "It is a zip with an install script, which rewrites partitions rather than installing a second system. Go back to choose another.",
                ]),
                PackageKind::Image => l.pick([
                    "这是分区镜像，会刷入所选分区，不会装成第二系统。返回后可重新选择。",
                    "這是分割區映像檔，會刷入所選分割區，不會裝成第二系統。返回後可重新選擇。",
                    "It is a partition image, which is written to a partition rather than installing a second system. Go back to choose another.",
                ]),
                PackageKind::System => l.pick(["这是第二系统的安装包。", "這是第二系統的安裝檔。", "It is a second system's package."]),
            };
            let y = widgets::list(&mut c, t, f, top, &[standing(title, ORANGE, Icon::Systems, detail)]).bottom;
            let back = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(["选择其他文件", "選擇其他檔案", "Choose another file"])).icon(GREEN, Icon::Folder)]);
            out.extend(hits(&back.rows, &[Action::Back]));
            let y = back.bottom;
            let note = l.pick([
                "第二系统只能从「双系统」页面的「安装系统」进入安装，且只接受带有 system.toml 和 rootfs.img 的安装包。",
                "第二系統只能從「雙系統」頁面的「安裝系統」進入安裝，且只接受帶有 system.toml 和 rootfs.img 的安裝檔。",
                "A second system installs only from Dual boot's Install a system, and only from a package holding system.toml and rootfs.img.",
            ]);
            widgets::footnote(&mut c, t, f, y, note)
        }
        Install::Scripting { stage, .. } => {
            let (detail, done) = script_stage(l, stage.as_ref(), &s.script_output);
            let y = widgets::progress_card(&mut c, t, f, top, l.pick(texts::installing::SCRIPT), detail, done);
            let y = widgets::footnote(&mut c, t, f, y, until_installed(l, s.source));
            script_output(&mut c, cx, y, &mut rows)
        }
        Install::Wiping => {
            let y = widgets::progress_card(&mut c, t, f, top, l.pick(texts::wipe::WIPING), factory_reset(s), None);
            widgets::footnote(&mut c, t, f, y, l.pick(texts::wipe::KEEP_POWER))
        }
        Install::Wiped(Ok(())) => {
            let made_anew = l.pick(texts::installing::DATA_MADE_ANEW);
            let y = widgets::list(&mut c, t, f, top, &[standing(l.pick(texts::wipe::WIPED), GREEN, Icon::Wipe, made_anew)]).bottom;
            widgets::footnote(&mut c, t, f, y, l.pick(texts::installing::MORE_COMMANDS))
        }
        Install::Wiped(Err(stop)) => {
            let (what, detail) = fmt::erase_stop(l, stop);
            let mut y = widgets::list(&mut c, t, f, top, &[standing(l.pick(texts::wipe::INCOMPLETE), RED, Icon::Wipe, &what)]).bottom;
            if let Some(d) = detail {
                y = widgets::footnote(&mut c, t, f, y, &d);
            }
            widgets::footnote(&mut c, t, f, y, l.pick(texts::installing::MORE_COMMANDS))
        }
        Install::Finished { outcome, damage } => finished(&mut c, cx, top, outcome, damage.clone(), &mut out, &mut rows),
    };
    // What the system asked for says why, where it said.
    let bottom = match (s.source, &s.asked.reason) {
        (Source::Asked { .. }, Some(reason)) => widgets::info_card(&mut c, t, f, bottom + t.m.dp(12.0), &[(l.pick(texts::installing::REASON), reason.clone())]),
        _ => bottom,
    };
    Frame { rows, ..Frame::new(t, fit(c, t, bottom), out, install_title(l, s.source), top) }
}

/// The package the system asked for, by the path it gave, at `y`.
fn asked_package(c: &mut Canvas, cx: &Ctx, y: f32) -> f32 {
    let name = cx.state.package_name.clone().unwrap_or_default();
    widgets::info_card(c, cx.t, cx.f, y + cx.t.m.dp(12.0), &[(cx.state.lang.pick(texts::installing::PACKAGE), name)])
}

/// What not to do while a package from `source` is installed.
fn until_installed(l: Lang, source: Source) -> &'static str {
    l.pick(match source {
        Source::Computer | Source::Rescue => ["安装完成前，请不要断开数据线或关机。", "安裝完成前，請不要拔除傳輸線或關機。", "Keep the cable in and don't power off until it's installed."],
        Source::File(Place::Storage) | Source::Opened { drive: false } | Source::Asked { .. } => ["安装完成前，请不要关机。", "安裝完成前，請不要關機。", "Don't power off until it's installed."],
        Source::File(Place::UsbDrive) | Source::Opened { drive: true } => {
            ["安装完成前，请不要拔出 U 盘或关机。", "安裝完成前，請不要拔出 USB 隨身碟或關機。", "Keep the USB drive in and don't power off until it's installed."]
        }
    })
}

/// How an install ended, and where to go from there: another package from
/// the computer, or back to choose another file.
fn finished(c: &mut Canvas, cx: &Ctx, top: f32, outcome: &Outcome, damage: Option<Damage>, out: &mut Vec<Hit>, rows: &mut Option<Rows>) -> f32 {
    let (t, f, source, l) = (cx.t, cx.f, cx.state.source, cx.state.lang);
    let gap = t.m.card_gap;
    use texts::installed;
    let choose_another = installed::CHOOSE_ANOTHER;
    let (again, another) = match source {
        Source::Computer => (Row::new(l.pick(installed::SEND_AGAIN)).icon(GREEN, Icon::Usb), Action::Sideload),
        Source::File(_) | Source::Opened { .. } => (Row::new(l.pick(choose_another)).icon(GREEN, Icon::Folder), Action::Back),
        Source::Asked { .. } => (Row::new(l.pick(choose_another)).icon(GREEN, Icon::Folder), Action::Open(Screen::Install)),
        Source::Rescue => (Row::new(l.pick(installed::LEAVE_RESCUE)).icon(GREEN, Icon::Usb), Action::Back),
    };
    let restart = Row::new(l.pick(installed::REBOOT_TO_SYSTEM)).icon(ORANGE, Icon::Power);
    let done = l.pick(installed::INSTALLED);
    let nothing_written = l.pick(installed::NOTHING_WRITTEN);
    match outcome {
        Outcome::Installed { slot, .. } => {
            let detail = installed::to_slot(l, slot.letter());
            let y = widgets::list(c, t, f, top, &[standing(done, GREEN, Icon::Install, &detail)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[restart]);
            out.extend(hits(&next.rows, &[Action::RebootTo(Target::System)]));
            let y = reflashed(c, cx, next.bottom);
            afterwards_of(c, cx, y, true)
        }
        Outcome::Flashed { partition, slots } => {
            let detail = match slots.as_slice() {
                [] => installed::flashed_whole(l, partition),
                [slot] => installed::flashed_one(l, partition, slot.letter()),
                [a, b] => installed::flashed_two(l, partition, a.letter(), b.letter()),
                other => unreachable!("a phone has two slots, and the image went to {other:?}"),
            };
            let y = widgets::list(c, t, f, top, &[standing(l.pick(installed::FLASHED), GREEN, Icon::Install, &detail)]).bottom;
            let mut rows = vec![restart];
            let mut acts = vec![Action::RebootTo(Target::System)];
            if *partition == "recovery" {
                rows.push(Row::new(l.pick(installed::REBOOT_TO_RECOVERY)).icon(ORANGE, Icon::Power));
                acts.push(Action::RebootTo(Target::Recovery));
            }
            rows.push(again);
            acts.push(another);
            let next = widgets::list(c, t, f, y + gap, &rows);
            out.extend(hits(&next.rows, &acts));
            afterwards_of(c, cx, next.bottom, true)
        }
        Outcome::Ran { .. } => {
            let ran = l.pick(installed::SCRIPT_RAN);
            let y = widgets::list(c, t, f, top, &[standing(done, GREEN, Icon::Install, ran)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[restart, again]);
            out.extend(hits(&next.rows, &[Action::RebootTo(Target::System), another]));
            let y = reflashed(c, cx, next.bottom);
            let y = afterwards_of(c, cx, y, true);
            script_output(c, cx, y, rows)
        }
        Outcome::SystemInstalled { name } => {
            let detail =
                tr!(l, "{name} 已安装为第二系统，可在「双系统」中切换。", "{name} 已安裝為第二系統，可在「雙系統」中切換。", "{name} is installed as the second system. Switch to it from Dual boot.");
            let installed = l.pick(["安装成功", "安裝成功", "Installed"]);
            let y = widgets::list(c, t, f, top, &[standing(installed, GREEN, Icon::Systems, &detail)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[Row::new(l.pick(DUAL_BOOT)).icon(PURPLE, Icon::Systems)]);
            out.extend(hits(&next.rows, &[Action::Open(Screen::Systems)]));
            afterwards_of(c, cx, next.bottom, true)
        }
        Outcome::Cancelled => {
            let y = widgets::list(c, t, f, top, &[standing(l.pick(installed::CANCELLED), SLATE, Icon::Install, nothing_written)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[again]);
            out.extend(hits(&next.rows, &[another]));
            next.bottom
        }
        Outcome::Refused(r) => {
            let reason = fmt::refusal(l, r, &cx.info.codename, source, cx.state.zone());
            let y = widgets::list(c, t, f, top, &[standing(l.pick(installed::CANT_INSTALL), ORANGE, Icon::Install, &reason)]).bottom;
            // What the system asked for and did not go in leaves the phone
            // where it was, from which it may go back to the system.
            let (rows, acts) = match source {
                Source::Asked { .. } => (vec![again, restart], vec![another, Action::RebootTo(Target::System)]),
                _ => (vec![again], vec![another]),
            };
            let next = widgets::list(c, t, f, y + gap, &rows);
            out.extend(hits(&next.rows, &acts));
            let y = widgets::footnote(c, t, f, next.bottom, nothing_written);
            afterwards_of(c, cx, y, false)
        }
        Outcome::Failed(failure) => {
            let (what, detail) = fmt::failure(l, failure, source);
            let y = widgets::list(c, t, f, top, &[standing(l.pick(installed::FAILED), RED, Icon::Install, &what)]).bottom;
            let next = widgets::list(c, t, f, y + gap, &[again, restart]);
            out.extend(hits(&next.rows, &[another, Action::RebootTo(Target::System)]));
            // The package is sent again, or installed or flashed again.
            let sent = matches!(source, Source::Computer | Source::Rescue);
            let warning = match (damage, sent) {
                (Some(Damage::System), true) => l.pick(installed::damage::SYSTEM_SEND).into(),
                (Some(Damage::System), false) => l.pick(installed::damage::SYSTEM_INSTALL).into(),
                (Some(Damage::Partition(p)), true) => installed::damage::partition_send(l, p),
                (Some(Damage::Partition(p)), false) => installed::damage::partition_flash(l, p),
                (Some(Damage::Script), true) => l.pick(installed::damage::SCRIPT_SEND).into(),
                (Some(Damage::Script), false) => l.pick(installed::damage::SCRIPT_INSTALL).into(),
                (None, _) => l.pick(installed::damage::NONE).to_string(),
            };
            let mut y = widgets::footnote(c, t, f, next.bottom, &warning);
            if let Some(d) = detail {
                y = widgets::footnote(c, t, f, y, &d);
            }
            let y = reflashed(c, cx, y);
            let y = afterwards_of(c, cx, y, false);
            script_output(c, cx, y, rows)
        }
    }
}

/// What comes after an install, where it is not the user's to choose:
/// rescue waits for the computer's next command, and a package the system
/// asked for, once in, has the phone leave on its own.
fn afterwards_of(c: &mut Canvas, cx: &Ctx, y: f32, installed: bool) -> f32 {
    let l = cx.state.lang;
    let leaving = match cx.state.asked.shutdown_after {
        true => texts::installed::leaving::POWER_OFF,
        false => texts::installed::leaving::REBOOT,
    };
    match cx.state.source {
        Source::Rescue => widgets::footnote(c, cx.t, cx.f, y, l.pick(texts::installing::MORE_COMMANDS)),
        Source::Asked { .. } if installed => widgets::footnote(c, cx.t, cx.f, y, l.pick(leaving)),
        _ => y,
    }
}

/// What the wipe page lists of [`WIPES`] beyond everything, under the
/// partition each is in.
const WIPE_GROUPS: [(Text, &[Wipe]); 2] =
    [(DATA_PART_NAME, &[Wipe::AppData, Wipe::Media, Wipe::AppCaches, Wipe::SystemCaches, Wipe::Art, Wipe::Logs, Wipe::Updates]), (["cache 分区", "cache 分割區", "cache partition", "cache partition", "cache partition", "cache partition", "cache partition", "cache partition", "cache partition", "cache partition", "cache partition", "cache partition", "cache partition", "cache partition", "cache partition"], &[Wipe::Cache])];

/// What a factory reset erases, as the wipe page tells it of its first
/// choice, and the page that carries one out for the system tells it.
fn factory_reset(s: &State) -> &'static str {
    s.lang.pick(match s.cache_partition {
        true => texts::wipe::factory_reset::WITH_CACHE,
        false => texts::wipe::factory_reset::WITHOUT_CACHE,
    })
}

/// A choice of the wipe page as it lists it: its title, and exactly what it
/// erases.
fn wipe_row(w: Wipe, s: &State) -> (&'static str, String) {
    let l = s.lang;
    let (title, what): (Text, String) = match w {
        Wipe::All => (["全部数据", "全部資料", "All data", "All data", "All data", "All data", "All data", "All data", "All data", "All data", "All data", "All data", "All data", "All data", "All data"], capitalized(l, factory_reset(s))),
        Wipe::AppData => (
            ["应用数据", "應用程式資料", "App data", "App data", "App data", "App data", "App data", "App data", "App data", "App data", "App data", "App data", "App data", "App data", "App data"],
            l.pick([
                "/data/data、/data/user、/data/user_de，以及内部存储中的 Android/data、Android/media，应用将恢复到初始状态",
                "/data/data、/data/user、/data/user_de，以及內部儲存空間中的 Android/data、Android/media，應用程式將恢復到初始狀態",
                "/data/data, /data/user, /data/user_de, and Android/data and Android/media in internal storage. Apps go back to how they were first installed",
            ])
            .into(),
        ),
        Wipe::Media => (
            ["内部存储", "內部儲存空間", "Internal storage", "Internal storage", "Internal storage", "Internal storage", "Internal storage", "Internal storage", "Internal storage", "Internal storage", "Internal storage", "Internal storage", "Internal storage", "Internal storage", "Internal storage"],
            l.pick(["/data/media，即 /sdcard 中的照片、视频和文件", "/data/media，即 /sdcard 中的相片、影片和檔案", "/data/media: the photos, videos and files in /sdcard"]).into(),
        ),
        Wipe::AppCaches => (
            ["应用缓存", "應用程式快取", "App caches", "App caches", "App caches", "App caches", "App caches", "App caches", "App caches", "App caches", "App caches", "App caches", "App caches", "App caches", "App caches"],
            l.pick([
                "各应用数据中的 cache、code_cache，以及 Android/data 中的 cache，应用将按需重新生成",
                "各應用程式資料中的 cache、code_cache，以及 Android/data 中的 cache，應用程式將視需要重新產生",
                "cache and code_cache in each app's data, and cache in Android/data. Apps make them again as they need them",
            ])
            .into(),
        ),
        Wipe::SystemCaches => (
            ["系统缓存", "系統快取", "System caches", "System caches", "System caches", "System caches", "System caches", "System caches", "System caches", "System caches", "System caches", "System caches", "System caches", "System caches", "System caches"],
            l.pick([
                "/data/system/package_cache、/data/resource-cache、/data/cache，下次开机时重新生成",
                "/data/system/package_cache、/data/resource-cache、/data/cache，下次開機時重新產生",
                "/data/system/package_cache, /data/resource-cache and /data/cache, made again at the next start",
            ])
            .into(),
        ),
        Wipe::Art => (
            ["ART 缓存", "ART 快取", "ART cache", "ART cache", "ART cache", "ART cache", "ART cache", "ART cache", "ART cache", "ART cache", "ART cache", "ART cache", "ART cache", "ART cache", "ART cache"],
            l.pick([
                "/data/dalvik-cache、/data/misc/apexdata/com.android.art/dalvik-cache、/data/ota，下次开机时重新编译，启动时间较长",
                "/data/dalvik-cache、/data/misc/apexdata/com.android.art/dalvik-cache、/data/ota，下次開機時重新編譯，啟動時間較長",
                "/data/dalvik-cache, /data/misc/apexdata/com.android.art/dalvik-cache and /data/ota, compiled again at the next start, which then takes longer",
            ])
            .into(),
        ),
        Wipe::Logs => {
            let own = match (l, s.phone_logs) {
                (_, 0) => String::new(),
                (Lang::En, 1) => ", and this phone's own log folder".into(),
                (_, n) => tr!(l, "，以及本机的 {n} 个日志文件夹", "，以及本機的 {n} 個記錄資料夾", ", and this phone's own {n} log folders"),
            };
            let what = tr!(
                l,
                "/data/tombstones、/data/anr、/data/system/dropbox、/data/misc/logd 等系统日志和崩溃记录，各用户的错误报告{own}",
                "/data/tombstones、/data/anr、/data/system/dropbox、/data/misc/logd 等系統記錄和當機記錄，各使用者的錯誤報告{own}",
                "System logs and crash records, such as /data/tombstones, /data/anr, /data/system/dropbox and /data/misc/logd, each user's bug reports{own}"
            );
            (["日志", "記錄", "Logs", "Logs", "Logs", "Logs", "Logs", "Logs", "Logs", "Logs", "Logs", "Logs", "Logs", "Logs", "Logs"], what)
        }
        Wipe::Updates => (
            ["下载的更新包", "下載的更新檔", "Downloaded updates", "Downloaded updates", "Downloaded updates", "Downloaded updates", "Downloaded updates", "Downloaded updates", "Downloaded updates", "Downloaded updates", "Downloaded updates", "Downloaded updates", "Downloaded updates", "Downloaded updates", "Downloaded updates"],
            l.pick(["/data/ota_package 中的系统更新包", "/data/ota_package 中的系統更新檔", "System update packages in /data/ota_package"]).into(),
        ),
        Wipe::Cache => (
            ["全部内容", "全部內容", "Everything", "Everything", "Everything", "Everything", "Everything", "Everything", "Everything", "Everything", "Everything", "Everything", "Everything", "Everything", "Everything"],
            l.pick(["格式化 cache 分区，保留 recovery 文件夹中的日志", "格式化 cache 分割區，保留 recovery 資料夾中的記錄", "Formats the cache partition, keeping the logs in its recovery folder"])
                .into(),
        ),
    };
    (l.pick(title), what)
}

/// What to erase: everything, as a factory reset does, or parts of it on
/// their own, the cache partition among them where the phone has one; then
/// a swipe erases what is chosen.
pub fn wipe(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let chosen = |w: Wipe| s.wipe[WIPES.iter().position(|&x| x == w).expect("every choice is one of WIPES")];
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(texts::wipe::RESET));
    let (name, sub) = wipe_row(Wipe::All, s);
    let all = widgets::list(&mut c, t, f, y, &[Row::new(name).subtitle(&sub).trail(Trail::Check(chosen(Wipe::All)))]);
    out.extend(hits(&all.rows, &[Action::WipeItem(Wipe::All)]));
    let mut y = widgets::footnote(&mut c, t, f, all.bottom, l.pick(texts::wipe::STAYS));
    for (title, items) in WIPE_GROUPS {
        let shown: Vec<Wipe> = items.iter().copied().filter(|&w| w != Wipe::Cache || s.cache_partition).collect();
        if shown.is_empty() {
            continue;
        }
        let below = widgets::section(&mut c, t, f, y, l.pick(title));
        let texts: Vec<(&str, String)> = shown.iter().map(|&w| wipe_row(w, s)).collect();
        let rows: Vec<Row> = shown.iter().zip(&texts).map(|(&w, (name, sub))| Row::new(name).subtitle(sub).trail(Trail::Check(chosen(w)))).collect();
        let list = widgets::list(&mut c, t, f, below, &rows);
        out.extend(hits(&list.rows, &shown.iter().map(|&w| Action::WipeItem(w)).collect::<Vec<_>>()));
        y = list.bottom;
    }
    let y = widgets::footnote(&mut c, t, f, y, l.pick(texts::wipe::NOTE));
    let (label, danger) = confirm_label(Confirm::Wipe, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Wipe) });
    Frame::new(t, fit(c, t, track.y + track.h), out, l.pick(texts::page::WIPE), top)
}

/// A page laid out above the on-screen keyboard: where the keyboard's top
/// is, and its keys as touch targets.
fn keyboard_hits(c: &mut Canvas, t: &Theme, f: &Fonts, s: &State, enter: &str, out: &mut Vec<Hit>) -> f32 {
    let top = t.m.h as f32 - widgets::keyboard_height(t);
    out.extend(crate::ime::bar(c, t, f, &s.ime, text_floor(t)));
    let keys = widgets::keyboard(c, t, f, s.lang, top, s.layer, s.key_held(), enter, s.ime.enabled);
    out.extend(keys.into_iter().map(|(rect, cap)| Hit { rect, action: Action::Type(cap) }));
    top
}

/// Where the keyboard's top edge is on this theme's screen.
fn kb_top(t: &Theme) -> f32 {
    t.m.h as f32 - widgets::keyboard_height(t)
}

/// The keys and the spelling bar as one strip held at the foot of the screen.
///
/// A page that holds its keys in a dock rather than drawing them on the page
/// cannot use [`keyboard_hits`], and drawing only the keys would leave it with
/// no way to type Chinese: the bar is part of the strip, above the keys, and
/// its hits come back in the strip's own coordinates with the keys'.
fn keys_dock(t: &Theme, f: &Fonts, s: &State, enter: &str) -> Dock {
    let m = &t.m;
    let bar = crate::ime::height(t);
    let mut canvas = Canvas::new(m.w, (bar + widgets::keyboard_height(t)).ceil() as u32);
    let mut hits = crate::ime::bar(&mut canvas, t, f, &s.ime, 0.0);
    hits.extend(
        widgets::keyboard(&mut canvas, t, f, s.lang, bar, s.layer, s.key_held(), enter, s.ime.enabled)
            .into_iter()
            .map(|(rect, cap)| Hit { rect, action: Action::Type(cap) }),
    );
    Dock { canvas, hits }
}

/// The lowest line a page's content may use: the keyboard's top edge less the
/// strip the spelling bar takes.
fn text_floor(t: &Theme) -> f32 {
    kb_top(t) - crate::ime::height(t)
}

/// Finding a feature or partition: the field where the title would be, the
/// button that leaves the search beside it, what it finds, and the
/// keyboard.
pub fn search(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    // A long page: a query that finds many features lists more than one
    // screen holds. The keyboard is held at the foot of the screen as a dock
    // rather than drawn on the page, so it stays put while the page scrolls.
    //
    // The sheet is a fixed five screens whatever the query finds, so that a
    // page drawn again in part is as tall as it was: a sheet that grew and
    // shrank with every key typed could not be drawn again a band at a time.
    // Five is more rows than any query can find, a wrapped subtitle and all,
    // so nothing is cut off; what the page scrolls to is where its rows stop.
    let mut c = long_page_of(cx, 5);
    let mut out = Vec::new();
    let (at, cancel) = widgets::search_layout(t);
    // What the field is looking through says what it asks for: a page's apps
    // or modules, or the whole of what the phone offers.
    let hint = match s.search_scope {
        SearchScope::Apps => l.pick(["搜索应用", "搜尋應用程式", "Search apps"]),
        SearchScope::Modules => l.pick(["搜索模块", "搜尋模組", "Search modules"]),
        SearchScope::Files => l.pick(["搜索文件", "搜尋檔案", "Search files"]),
        SearchScope::Features => l.pick(SEARCH_HINT),
    };
    // The field and the button that leaves the search are held at the top of
    // the screen rather than drawn on the page: the page scrolls, and a field
    // that went up with it would be gone as soon as there was anything to
    // scroll to. A page with no `<` holds its strip at the top of the screen.
    let strip_top = at.y;
    let mut pinned_canvas = Canvas::new(m.w, (at.h + m.dp(12.0)).ceil() as u32);
    pinned_canvas.clear(t.pal.bg);
    let field = widgets::search_field(&mut pinned_canvas, t, f, at.x, at.w, at.h / 2.0, &s.search, hint);
    widgets::cancel_button(&mut pinned_canvas, t, Rect::new(cancel.x, 0.0, cancel.w, cancel.h));
    let pinned = Pinned { canvas: pinned_canvas, top: strip_top, floor: strip_top, hits: vec![Hit { rect: Rect::new(cancel.x, 0.0, cancel.w, cancel.h), action: Action::CloseSearch }] };
    let top = strip_top + at.h + m.dp(12.0);
    let bottom;
    if s.search.trim().is_empty() {
        let how = match s.search_scope {
            SearchScope::Apps => l.pick(["输入应用名称或包名筛选，例如输入 chrome 可找到 Chrome。", "輸入應用程式名稱或套件名稱篩選，例如輸入 chrome 可找到 Chrome。", "Type an app's name or package, such as chrome to find Chrome."]),
            SearchScope::Modules => l.pick(["输入模块名称或目录名筛选。", "輸入模組名稱或目錄名篩選。", "Type a module's name or folder to filter them."]),
            // The file manager filters its own folder as the name is typed, so
            // there is nothing to say here beyond how to get back to it.
            SearchScope::Files => l.pick(["在这个文件夹里找，输入名称的一部分即可。", "在這個資料夾裡找，輸入名稱的一部分即可。", "Find a name in this folder: part of it is enough."]),
            SearchScope::Features => l.pick([
                "输入功能或分区名称，支持拼音全拼和首字母，例如输入 bf 可找到「备份」。",
                "輸入功能或分割區名稱，支援英文和漢語拼音的全拼與首字母，例如輸入 bf 可找到「備份」。",
                "Type the name of a feature or partition, such as backup to find Backup.",
            ]),
        };
        bottom = widgets::footnote(&mut c, t, f, top, how);
    } else {
        // A page's entries are filtered, and the row opens the one it names;
        // the whole of what the phone offers is searched, and the row opens
        // the page.
        let query = s.search.trim().to_lowercase();
        match s.search_scope {
            SearchScope::Features => {
                let found = search::find_on(&s.search, l, |screen| s.offers(screen));
                if found.is_empty() {
                    let typed = s.search.trim();
                    let none = tr!(l, "未找到\"{typed}\"", "找不到「{typed}」", "Nothing found for “{typed}”");
                    bottom = widgets::list(&mut c, t, f, top, &[Row::new(&none).trail(Trail::None)]).bottom;
                } else {
                    let rows: Vec<Row> = found.iter().map(|e| Row::new(l.pick(e.title)).subtitle(l.pick(e.place))).collect();
                    let list = widgets::list(&mut c, t, f, top, &rows);
                    // Every row answers, the ones under the keyboard too: the
                    // page scrolls them up above it.
                    out.extend(list.rows.iter().zip(&found).map(|(&rect, e)| Hit { rect, action: Action::Open(e.screen) }));
                    bottom = list.bottom;
                }
            }
            scope => {
                let (rows, acts) = filtered(s, scope, &query);
                if rows.is_empty() {
                    let typed = s.search.trim();
                    let none = tr!(l, "未找到\"{typed}\"", "找不到「{typed}」", "Nothing found for “{typed}”");
                    bottom = widgets::list(&mut c, t, f, top, &[Row::new(&none).trail(Trail::None)]).bottom;
                } else {
                    let list = widgets::list(&mut c, t, f, top, &rows);
                    out.extend(list.rows.iter().zip(&acts).map(|(&rect, &action)| Hit { rect, action }));
                    bottom = list.bottom;
                }
            }
        }
    }
    let find = l.pick(["搜索", "搜尋", "Search"]);
    // The keyboard is held at the foot of the screen rather than drawn on the
    // page: a page that scrolls would take the keys with it.
    let dock = keys_dock(t, f, s, find);
    // The field's clear button is on the strip too, in the strip's own
    // coordinates.
    let mut pinned = pinned;
    if let Some(rect) = field.clear {
        pinned.hits.push(Hit { rect, action: Action::ClearSearch });
    }
    // The page is as tall as three screens whatever it finds, so that drawing
    // part of it again draws the same page: a canvas that grew and shrank with
    // every key typed would not. What it scrolls to is where the rows stop,
    // which `content_end` says: the last row's foot and the margin under it.
    // Taking the keys' own top as a floor as well would let a short list
    // scroll, and a long one stop before its last row.
    let end = bottom + m.margin;
    Frame { back: false, header: false, caret: Some(field.caret), pinned: Some(pinned), dock: Some(dock), content_end: Some(end), ..Frame::new(t, c, out, find, 0.0) }
}

/// Backing up: what to take, where it goes, and how.
pub fn backup(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["备份内容", "備份內容", "Contents"]));
    let system = fmt::snapshot_backup_note(l, &i.snapshot).unwrap_or(l.pick(["当前槽位在 super 中的分区", "目前槽位在 super 中的分割區", "The current slot's partitions in super"]));
    let data = l.pick(match s.data == Data::Open {
        true => [
            "应用、账户和设置，不包括内部存储、锁屏密码和指纹",
            "應用程式、帳戶和設定，不包括內部儲存空間、螢幕鎖定密碼和指紋",
            "Apps, accounts and settings, without internal storage, the screen lock or fingerprints",
        ],
        false => [
            "应用、账户和设置，不包括内部存储、锁屏密码和指纹，需先完成「解密」",
            "應用程式、帳戶和設定，不包括內部儲存空間、螢幕鎖定密碼和指紋，需先完成「解密」",
            "Apps, accounts and settings, without internal storage, the screen lock or fingerprints. Decrypt first",
        ],
    });
    let boot = l.pick(["boot、init_boot、vendor_boot、dtbo、vbmeta 和 recovery", "boot、init_boot、vendor_boot、dtbo、vbmeta 和 recovery", "boot, init_boot, vendor_boot, dtbo, vbmeta and recovery"]);
    let modem = l.pick(["IMEI、基带设置和传感器校准数据", "IMEI、基頻設定和感應器校正資料", "IMEI, modem settings and sensor calibration"]);
    let media = l.pick(match s.data == Data::Open {
        true => ["内部存储中的照片、视频和音乐", "內部儲存空間中的相片、影片和音樂", "Photos, videos and music in internal storage"],
        false => ["内部存储中的照片、视频和音乐，需先完成「解密」", "內部儲存空間中的相片、影片和音樂，需先完成「解密」", "Photos, videos and music in internal storage. Decrypt first"],
    });
    let subs = [boot, system, data, media, modem];
    let rows: Vec<Row> = BACKUP_PARTS.iter().zip(subs).enumerate().map(|(k, (name, sub))| Row::new(l.pick(*name)).subtitle(sub).trail(Trail::Check(s.backup[k]))).collect();
    let parts = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&parts.rows, &[0, 1, 2, 3, 4].map(Action::BackupPart)));
    let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(texts::backup::SAVE_TO));
    let storage = match (s.data.storage_open(), i.storage.free.map(fmt::gb)) {
        (true, Some(free)) => texts::backup::free(l, free),
        (true, None) => l.pick(texts::backup::FREE_UNKNOWN).to_string(),
        (false, _) if s.data == Data::Erased => l.pick(texts::data::ERASED).to_string(),
        (false, _) => l.pick(texts::decrypt::NEEDED).to_string(),
    };
    let usb = l.pick(match &s.usb_drive {
        Some(UsbDrive::Shown) => texts::backup::CONNECTED,
        Some(UsbDrive::Unreadable(_)) => texts::usb_drive::UNREADABLE,
        None => texts::usb_drive::WANTED,
    });
    let places = widgets::list(
        &mut c,
        t,
        f,
        y,
        &[
            Row::new(place_name(l, Place::Storage)).subtitle(&storage).trail(Trail::Check(s.backup_to == Place::Storage)),
            Row::new(place_name(l, Place::UsbDrive)).subtitle(usb).trail(Trail::Check(s.backup_to == Place::UsbDrive)),
        ],
    );
    out.extend(hits(&places.rows, &[Action::BackupTo(Place::Storage), Action::BackupTo(Place::UsbDrive)]));
    let y = widgets::section(&mut c, t, f, places.bottom, l.pick(["选项", "選項", "Options"]));
    let options = widgets::list(
        &mut c,
        t,
        f,
        y,
        &[
            Row::new(l.pick(["压缩", "壓縮", "Compress"])).subtitle(l.pick(["备份更小，耗时更长", "備份更小，耗時更長", "Smaller, and slower to make"])).trail(Trail::Switch(s.compress)),
            Row::new(l.pick(["完成后校验", "完成後驗證", "Verify when done"]))
                .subtitle(l.pick(["确认备份文件完整", "確認備份檔案完整", "Checks the backup's files are whole"]))
                .trail(Trail::Switch(s.verify)),
        ],
    );
    out.extend(hits(&options.rows, &[Action::Toggle(Switch::Compress), Action::Toggle(Switch::Verify)]));
    let y = widgets::footnote(&mut c, t, f, options.bottom, l.pick(texts::backup::NOTE));
    let (label, danger) = confirm_label(Confirm::Backup, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Backup) });
    Frame::new(t, fit(c, t, track.y + track.h), out, l.pick(texts::page::BACKUP), top)
}

/// The backups found, newest first, to restore one; those that cannot be
/// read are listed after them, to be deleted.
pub fn restore(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(texts::restore::BACKUPS));
    let bottom = if s.backups.is_empty() {
        widgets::list(&mut c, t, f, y, &[Row::new(l.pick(texts::restore::NONE)).trail(Trail::None)]).bottom
    } else {
        let lines: Vec<(String, String)> = s
            .backups
            .iter()
            .map(|b| match &b.held {
                Ok(h) => (fmt::datetime(l, h.made, s.zone()), format!("{} · {} · {}", fmt::parts(l, &h.parts), fmt::bytes(h.size), place_name(l, b.place))),
                Err(_) => (b.name.clone(), format!("{} · {}", l.pick(texts::common::UNREADABLE), place_name(l, b.place))),
            })
            .collect();
        let rows: Vec<Row> = lines.iter().map(|(title, sub)| Row::new(title).subtitle(sub)).collect();
        let list = widgets::list(&mut c, t, f, y, &rows);
        out.extend(list.rows.iter().enumerate().map(|(k, &rect)| Hit { rect, action: Action::OpenBackup(k) }));
        list.bottom
    };
    let bottom = widgets::footnote(&mut c, t, f, bottom, l.pick(texts::restore::NOTE));
    Frame::new(t, fit(c, t, bottom), out, l.pick(texts::page::RESTORE), top)
}

/// One backup: when and where it was made, and which of its parts to
/// restore. One made on another phone is not restored here, and one that
/// cannot be read can only be deleted.
pub fn restore_backup(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let (entry, chosen) = s.restore.as_ref().expect("the restore page shows a backup");
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let location = l.pick(texts::common::LOCATION);
    let y = match &entry.held {
        Err(why) => {
            let facts = [(l.pick(texts::restore::FOLDER), entry.name.clone()), (location, place_name(l, entry.place).to_string())];
            let y = widgets::info_card(&mut c, t, f, top, &facts);
            widgets::footnote(&mut c, t, f, y, &texts::restore::unreadable(l, why))
        }
        Ok(held) => {
            let mut facts = vec![
                (l.pick(texts::restore::MADE), fmt::datetime(l, held.made, s.zone())),
                (l.pick(texts::common::SIZE), fmt::bytes(held.size)),
                (location, place_name(l, entry.place).to_string()),
            ];
            if let Some(system) = &held.system {
                facts.push((l.pick(texts::restore::SYSTEM_VERSION), system.clone()));
            }
            if let Some(slot) = held.slot {
                facts.push((l.pick(texts::slot::TITLE), slot.letter().to_string()));
            }
            let y = widgets::info_card(&mut c, t, f, top, &facts);
            if held.device != i.codename {
                widgets::footnote(&mut c, t, f, y, &texts::backup::refused::other_phone(l, &held.device))
            } else {
                let y = widgets::section(&mut c, t, f, y, l.pick(texts::restore::CONTENTS));
                let kept: Vec<usize> = (0..BACKUP_PARTS.len()).filter(|&k| held.parts[k]).collect();
                let rows: Vec<Row> = kept.iter().map(|&k| Row::new(l.pick(BACKUP_PARTS[k])).trail(Trail::Check(chosen[k]))).collect();
                let list = widgets::list(&mut c, t, f, y, &rows);
                out.extend(list.rows.iter().zip(&kept).map(|(&rect, &k)| Hit { rect, action: Action::RestorePart(k) }));
                let mut y = widgets::footnote(&mut c, t, f, list.bottom, l.pick(texts::restore::OVERWRITES));
                if chosen[DATA_PART] {
                    y = widgets::footnote(&mut c, t, f, y, l.pick(texts::restore::DATA_NOTE));
                }
                if chosen[MEDIA_PART] {
                    y = widgets::footnote(&mut c, t, f, y, l.pick(texts::restore::MEDIA_NOTE));
                }
                // The boot partitions' vbmeta checks the system's: one of
                // another version without the other may not start.
                if let (Some(was), Some(now)) = (&held.system, &i.os_version)
                    && was != now
                    && chosen[0] != chosen[1]
                {
                    y = widgets::footnote(&mut c, t, f, y, &texts::restore::other_version(l, was, now));
                }
                let (label, danger) = confirm_label(Confirm::Restore, cx.state);
                let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
                out.push(Hit { rect: track, action: Action::Confirm(Confirm::Restore) });
                track.y + track.h
            }
        }
    };
    let delete = widgets::list(&mut c, t, f, y + t.m.dp(28.0), &[Row::new(l.pick(texts::restore::DELETE)).tint(t.pal.danger).trail(Trail::None)]);
    out.extend(hits(&delete.rows, &[Action::AskDelete]));
    let mut bottom = delete.bottom;
    if s.asking_delete {
        let y = widgets::footnote(&mut c, t, f, delete.bottom, l.pick(texts::restore::ASK_DELETE));
        let (label, danger) = confirm_label(Confirm::DeleteBackup, cx.state);
        let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::DeleteBackup) });
        bottom = track.y + track.h;
    }
    Frame::new(t, fit(c, t, bottom), out, l.pick(texts::restore::TITLE), top)
}

/// The share of its pass the part under way has done.
fn part_share(a: &BackupAt) -> f32 {
    if a.of == 0 { 1.0 } else { share(a.done, a.of).min(1.0) }
}

/// How far backing up or restoring has come over all, 0 to 1, and the
/// bytes of the parts the pass under way has gone through, the part under
/// way by its share. Each part counts by its bytes in each pass: a restore
/// checks all its parts first, and a backup checks them last.
fn job_progress(job: &BackupJob, a: &BackupAt) -> (f32, u64) {
    let total: u64 = job.plan.iter().map(|(_, n)| n).sum();
    let before: u64 = job.plan[..a.item].iter().map(|(_, n)| n).sum();
    let passed = before as f64 + f64::from(part_share(a)) * job.plan[a.item].1 as f64;
    let second = job.checks && a.checking != job.restore;
    let passes = if job.checks { 2.0 } else { 1.0 };
    let over = (if second { total as f64 } else { 0.0 } + passed) / (passes * total.max(1) as f64);
    (over as f32, passed as u64)
}

/// What a backup or a restore does to a part as it goes through them:
/// takes it, puts it back, or checks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pass {
    BackUp,
    Restore,
    Verify,
}

impl Pass {
    /// The pass that writes: a restore's, or a backup's.
    fn writing(restore: bool) -> Pass {
        if restore { Pass::Restore } else { Pass::BackUp }
    }

    /// The pass a job is on at `a`.
    fn at(job: &BackupJob, a: &BackupAt) -> Pass {
        if a.checking { Pass::Verify } else { Pass::writing(job.restore) }
    }

    /// Under way: 正在备份, Backing up.
    fn doing(self) -> Text {
        match self {
            Pass::BackUp => texts::backup::job::BACKING_UP,
            Pass::Restore => texts::backup::job::RESTORING,
            Pass::Verify => texts::backup::job::VERIFYING,
        }
    }

    /// Done to a part: 已备份, Backed up.
    fn done(self) -> Text {
        match self {
            Pass::BackUp => texts::backup::job::BACKED_UP,
            Pass::Restore => texts::backup::job::RESTORED,
            Pass::Verify => texts::backup::job::VERIFIED,
        }
    }

    /// Under way on the part `name`: 正在备份 boot.
    fn doing_part(self, l: Lang, name: &str) -> String {
        match self {
            Pass::BackUp => texts::backup::job::backing_up_part(l, name),
            Pass::Restore => texts::backup::job::restoring_part(l, name),
            Pass::Verify => texts::backup::job::verifying_part(l, name),
        }
    }

    /// How much of the whole is done: 已备份 1.2GB，共 5.0GB.
    fn done_of(self, l: Lang, passed: &str, total: &str) -> String {
        match self {
            Pass::BackUp => texts::backup::job::backed_up_of(l, passed, total),
            Pass::Restore => texts::backup::job::restored_of(l, passed, total),
            Pass::Verify => texts::backup::job::verified_of(l, passed, total),
        }
    }
}

/// What a backup, or a restore, has done to a part, and does to it, in
/// each language.
fn job_words(restore: bool) -> (Text, Text) {
    match restore {
        true => (["已恢复", "已還原", "Restored", "Restored", "Restored", "Restored", "Restored", "Restored", "Restored", "Restored", "Restored", "Restored", "Restored", "Restored", "Restored"], ["正在恢复", "正在還原", "Restoring", "Restoring", "Restoring", "Restoring", "Restoring", "Restoring", "Restoring", "Restoring", "Restoring", "Restoring", "Restoring", "Restoring", "Restoring"]),
        false => (["已备份", "已備份", "Backed up", "Backed up", "Backed up", "Backed up", "Backed up", "Backed up", "Backed up", "Backed up", "Backed up", "Backed up", "Backed up", "Backed up", "Backed up"], ["正在备份", "正在備份", "Backing up", "Backing up", "Backing up", "Backing up", "Backing up", "Backing up", "Backing up", "Backing up", "Backing up", "Backing up", "Backing up", "Backing up", "Backing up"]),
    }
}
/// Where part `k` of a backup or restore stands, as its row says it.
fn part_state(l: Lang, job: &BackupJob, k: usize) -> String {
    use std::cmp::Ordering;
    use texts::backup::job as said;
    let (did, doing) = job_words(job.restore);
    let wrote = Pass::writing(job.restore).done();
    let ended = job.outcome.is_some();
    let a = match (&job.outcome, job.at) {
        (Some(BackupEnd::Saved(_)), _) => {
            return l.pick(if job.checks { VERIFIED } else { did }).to_string();
        }
        (Some(BackupEnd::Restored { .. }), _) => return l.pick(did).to_string(),
        (_, None) => {
            return l.pick(if ended { &["未开始", "未開始", "Not started"][..] } else { &WAITING_TURN[..] }).to_string();
        }
        (_, Some(a)) => a,
    };
    let state = match k.cmp(&a.item) {
        Ordering::Less if a.checking => VERIFIED,
        Ordering::Less => did,
        Ordering::Equal if ended => ["未完成", "未完成", "Unfinished", "Unfinished", "Unfinished", "Unfinished", "Unfinished", "Unfinished", "Unfinished", "Unfinished", "Unfinished", "Unfinished", "Unfinished", "Unfinished", "Unfinished"],
        Ordering::Equal => {
            return format!("{} {}%", l.pick(if a.checking { VERIFYING } else { doing }), percent(part_share(&a)));
        }
        // Ahead of the pass under way: a backup checks its parts once all
        // are written, and a restore writes them once all are checked.
        Ordering::Greater => match (a.checking, job.restore, ended) {
            (true, false, false) => wrote,
            (true, false, true) => said::NOT_VERIFIED,
            (false, false, false) | (true, true, false) => texts::common::WAITING,
            (false, false, true) => said::NOT_BACKED_UP,
            (false, true, false) => said::VERIFIED,
            (_, true, true) => said::NOT_RESTORED,
        },
    };
    l.pick(state).to_string()
}

/// Backing up or restoring: how far it has come over all, each part and
/// where it stands, and a way to stop while stopping breaks nothing. Once
/// it has ended, how, and where to go.
pub fn backup_job(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let job = s.backup_job.as_ref().expect("the backup page shows work");
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    use texts::backup::job as said;
    let icon = if job.restore { Icon::Restore } else { Icon::Backup };
    let writing = job.at.is_some_and(|a| !a.checking);
    let mut y = match &job.outcome {
        None => {
            let (title, detail, done) = match job.at {
                Some(a) => {
                    let (over, passed) = job_progress(job, &a);
                    let total = fmt::bytes(job.plan.iter().map(|(_, n)| n).sum());
                    let pass = Pass::at(job, &a);
                    (pass.doing_part(l, &job.plan[a.item].0), pass.done_of(l, &fmt::bytes(passed), &total), over)
                }
                None if job.restore => (l.pick(said::READING).to_string(), l.pick(said::CHECKING).to_string(), 0.0),
                None => (l.pick(texts::installing::GETTING_READY).to_string(), l.pick(said::SIZING).to_string(), 0.0),
            };
            widgets::progress_card(&mut c, t, f, top, &title, &detail, Some(done))
        }
        Some(end) => {
            let gone = l.pick(if job.restore {
                ["未写入任何内容。", "未寫入任何內容。", "Nothing was written."]
            } else {
                ["未保存任何备份。", "未儲存任何備份。", "No backup was saved."]
            });
            let failed = l.pick(if job.restore { &RESTORE_FAILED[..] } else { &["备份失败", "備份失敗", "Backup failed"][..] });
            let (title, color, detail) = match end {
                BackupEnd::Saved(name) => {
                    let saved = match job.place {
                        Place::Storage => said::saved_to_storage(l, name),
                        Place::UsbDrive => said::saved_to_usb_drive(l, name),
                    };
                    (l.pick(said::BACKUP_COMPLETE), GREEN, saved)
                }
                BackupEnd::Restored { .. } => (l.pick(said::RESTORE_COMPLETE), GREEN, l.pick(said::USES_RESTORED).to_string()),
                BackupEnd::Stopped => (l.pick(texts::common::STOPPED), SLATE, gone.to_string()),
                BackupEnd::Refused(r) => {
                    let title = l.pick(if job.restore { said::CANT_RESTORE } else { said::CANT_BACK_UP });
                    (title, ORANGE, fmt::backup_refusal(l, r, job.restore))
                }
                BackupEnd::Failed(_) if job.restore && writing => (failed, RED, l.pick(said::PARTLY_WRITTEN).to_string()),
                BackupEnd::Failed(_) => (failed, RED, gone.to_string()),
            };
            widgets::list(&mut c, t, f, top, &[standing(title, color, icon, &detail)]).bottom
        }
    };
    let keep = l.pick(match job.place {
        Place::Storage => ["请不要关机。", "請不要關機。", "Don't power off"],
        Place::UsbDrive => ["请不要拔出 U 盘或关机。", "請不要拔出 USB 隨身碟或關機。", "Keep the USB drive in and don't power off"],
    });
    match &job.outcome {
        None => {
            let note = match (job.restore, job.at) {
                (false, _) => {
                    tr!(l, "备份完成前，{keep}停止后将删除未完成的备份。", "備份完成前，{keep}停止後將刪除未完成的備份。", "{keep} until the backup is done. Stopping deletes the unfinished backup.")
                }
                (true, Some(a)) if !a.checking => tr!(l, "恢复完成前，{keep}", "還原完成前，{keep}", "{keep} until the restore is done."),
                (true, _) => l.pick(["校验备份完成前，不会写入任何内容。", "驗證備份完成前，不會寫入任何內容。", "Nothing is written until the backup is verified."]).to_string(),
            };
            y = widgets::footnote(&mut c, t, f, y, &note);
            // A restore that has begun writing goes on to its end.
            if !(job.restore && writing) {
                let label = l.pick(if job.stopping { texts::common::STOPPING } else { texts::common::STOP });
                let stop = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
                if !job.stopping {
                    out.extend(hits(&stop.rows, &[Action::StopBackup]));
                }
                y = stop.bottom;
            }
        }
        Some(end) => {
            match end {
                BackupEnd::Restored { unmade } if !unmade.is_empty() => {
                    let note = said::users_gone(l, unmade.join(l.pick(texts::list::SEPARATOR)));
                    y = widgets::footnote(&mut c, t, f, y, &note);
                }
                BackupEnd::Failed(why) => y = widgets::footnote(&mut c, t, f, y, why),
                _ => {}
            }
            let (rows, acts): (Vec<Row>, Vec<Action>) = match end {
                BackupEnd::Restored { .. } => (
                    vec![Row::new(l.pick(texts::installed::REBOOT_TO_SYSTEM)).icon(ORANGE, Icon::Power), Row::new(l.pick(texts::common::DONE)).trail(Trail::None)],
                    vec![Action::RebootTo(Target::System), Action::Back],
                ),
                _ => (vec![Row::new(l.pick(texts::common::DONE)).trail(Trail::None)], vec![Action::Back]),
            };
            let next = widgets::list(&mut c, t, f, y + t.m.card_gap, &rows);
            out.extend(hits(&next.rows, &acts));
            y = next.bottom;
        }
    }
    // The parts come after what can be done, which a long list would push
    // off the screen.
    if !job.plan.is_empty() {
        let heading = l.pick(if job.restore { texts::restore::CONTENTS } else { texts::backup::CONTENTS });
        let y0 = widgets::section(&mut c, t, f, y, heading);
        let states: Vec<String> = (0..job.plan.len()).map(|k| part_state(l, job, k)).collect();
        let rows: Vec<Row> = job.plan.iter().zip(&states).map(|((name, _), state)| Row::new(name).trail(Trail::Text(state))).collect();
        y = widgets::list(&mut c, t, f, y0, &rows).bottom;
    }
    let title = l.pick(match (&job.outcome, job.restore) {
        (None, restore) => Pass::writing(restore).doing(),
        (Some(BackupEnd::Saved(_)), _) => said::BACKUP_COMPLETE,
        (Some(BackupEnd::Restored { .. }), _) => said::RESTORE_COMPLETE,
        (Some(_), false) => said::BACKUP_INCOMPLETE,
        (Some(_), true) => said::RESTORE_INCOMPLETE,
    });
    Frame { back: job.outcome.is_some(), ..Frame::new(t, fit(c, t, y), out, title, top) }
}

/// Tools for the experienced.
/// The folder the file manager shows, served to a browser on the same
/// network: the folder, the switch that serves it, and where it is.
pub fn serve(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let mut y = widgets::section(&mut c, t, f, m.content_top, l.pick(["传输", "傳輸", "Transfer"]));
    let folder = &s.files.path;
    let row = Row::new(l.pick(["文件夹", "資料夾", "Folder"])).icon(BLUE, Icon::Folder).subtitle(folder).trail(Trail::Value(l.pick(["更改", "更改", "Change"])));
    let pick = widgets::list(&mut c, t, f, y, &[row]);
    out.extend(hits(&pick.rows, &[Action::ServePick]));
    y = widgets::section(&mut c, t, f, pick.bottom, l.pick(["服务", "服務", "Server"]));
    let on = matches!(s.serving, Serving::Running { .. });
    let mut line = Row::new(l.pick(["启动服务", "啟動服務", "Serve"])).icon(if on { GREEN } else { SLATE }, Icon::Globe).trail(Trail::Switch(on));
    if let Serving::Failed(why) = &s.serving {
        line = line.subtitle(why);
    }
    let run = widgets::list(&mut c, t, f, y, &[line]);
    out.extend(hits(&run.rows, &[Action::ServeToggle]));
    let below = run.bottom;
    if let Serving::Running { at, served } = &s.serving {
        let said = tr!(l, "已应答 {served} 次请求", "已回應 {served} 次請求", "{served} requests answered");
        widgets::info_card(&mut c, t, f, below + m.card_gap, &[(l.pick(["地址", "位址", "Address"]), at.clone()), (l.pick(["请求", "請求", "Requests"]), said)]);
    } else {
        let hint = l.pick([
            "在同一网络下用浏览器打开上面的地址，即可上传或下载这个文件夹里的文件。WLAN 需先连接。",
            "在同一網路下用瀏覽器開啟上面的位址，即可上傳或下載這個資料夾裡的檔案。Wi-Fi 需先連線。",
            "Open the address above in a browser on the same network to move files in and out of this folder. Wi-Fi has to be joined first.",
        ]);
        widgets::footnote(&mut c, t, f, below, hint);
    }
    Frame::new(t, c, out, l.pick(SERVE), m.content_top)
}

const SERVE: Text = [
    "无线传输",
    "無線傳輸",
    "Wireless transfer",
    "Wireless transfer",
    "Wireless transfer",
    "Wireless transfer",
    "Wireless transfer",
    "Wireless transfer",
    "Wireless transfer",
    "Wireless transfer",
    "Wireless transfer",
    "Wireless transfer",
    "Wireless transfer",
    "Wireless transfer",
    "Wireless transfer",
];
const FETCH: Text = [
    "网络下载",
    "網路下載",
    "Download",
    "Download",
    "Download",
    "Download",
    "Download",
    "Download",
    "Download",
    "Download",
    "Download",
    "Download",
    "Download",
    "Download",
    "Download",
];
/// A length of bytes in the units the pages speak.
fn mb(bytes: u64) -> String {
    match bytes {
        b if b >= 1 << 20 => format!("{:.1} MB", b as f64 / (1u64 << 20) as f64),
        b if b >= 1 << 10 => format!("{:.1} KB", b as f64 / (1u64 << 10) as f64),
        b => format!("{b} B"),
    }
}

/// A file from an address typed on the page's keyboard: the field and the
/// button, the way an address too long to type is taken from a file or from
/// another device through the page served here, and how the download stands.
pub fn fetch(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    // A long page, with the keys held in a dock at its foot: the page holds
    // the served page's address and the state of a download as well as the
    // field, which is more than a screen, and keys drawn on the page would
    // scroll away from the field they type into.
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let wide = m.w as f32 - m.margin * 2.0;
    // A little air under the title: the page's content starts at the title's
    // own edge, and a field laid on that edge looks joined to it.
    let top = m.content_top + m.dp(12.0);
    let hint = l.pick(["粘贴或输入完整网址", "貼上或輸入完整網址", "Type or paste a full address"]);
    let field = widgets::search_field(&mut c, t, f, m.margin, wide, top, &s.fetch_url, hint);
    let running = matches!(s.fetch, Fetch::Running { .. });
    let label = match running {
        true => l.pick(["停止下载", "停止下載", "Stop"]),
        false => l.pick(["开始下载", "開始下載", "Download"]),
    };
    let (bh, bw) = (m.dp(48.0), m.dp(220.0).min(wide));
    let mut y = field.field.y + field.field.h + m.dp(26.0);
    let (bx, by) = ((m.w as f32 - bw) / 2.0, y);
    c.fill_round_rect(bx, by, bw, bh, bh / 2.0, t.pal.accent);
    c.text_center(&f.heading, label, bx + bw / 2.0, by + bh / 2.0, m.row_title_px, Rgba::rgb(255, 255, 255));
    out.push(Hit {
        rect: Rect::new(bx, by, bw, bh),
        action: match running {
            true => Action::FetchStop,
            false => Action::FetchStart,
        },
    });
    y = by + bh + m.dp(16.0);
    let from_file = l.pick(["从 /sdcard/Download/link.txt 读链接", "從 /sdcard/Download/link.txt 讀連結", "Take the link from /sdcard/Download/link.txt"]);
    c.text_center(&f.body, from_file, m.w as f32 / 2.0, y + m.dp(18.0), m.sub_px, t.pal.accent);
    out.push(Hit { rect: Rect::new(m.margin, y, wide, m.dp(36.0)), action: Action::FetchFromFile });
    y += m.dp(36.0) + m.dp(10.0);
    // A link too long for this keyboard can be typed on another device, which
    // opens a page this recovery serves; the code is where that page is.
    if let Some(at) = s.fetch_portal.clone() {
        // The page scrolls, so the code may be as tall as it needs to be: a
        // square that fits the width, with the caption under it.
        let side = m.dp(168.0).min(wide);
        let x0 = (m.w as f32 - side) / 2.0;
        let quiet = 2.0f32;
        if let Ok(code) = qrcodegen::QrCode::encode_text(&at, qrcodegen::QrCodeEcc::Medium) {
            let n = code.size();
            let cell = side / (n as f32 + quiet * 2.0);
            c.fill_round_rect(x0 - m.dp(12.0), y - m.dp(6.0), side + m.dp(24.0), side + m.dp(24.0), m.dp(14.0), Rgba::rgb(255, 255, 255));
            for row in 0..n {
                for col in 0..n {
                    if code.get_module(col, row) {
                        let px = x0 + (col as f32 + quiet) * cell;
                        let py = y + m.dp(6.0) + (row as f32 + quiet) * cell;
                        c.fill_rect(px, py, cell + 0.5, cell + 0.5, Rgba::rgb(12, 12, 16));
                    }
                }
            }
        }
        y += side + m.dp(20.0);
        let how = l.pick([
            "用另一台手机扫这个码，在网页里输入网址，发回来就填进上面的框",
            "用另一台手機掃這個碼，在網頁裡輸入網址，傳回來就填進上面的框",
            "Scan this on another device, type the address there, and it fills the field above",
        ]);
        y = widgets::footnote(&mut c, t, f, y, how);
    }
    let end = match &s.fetch {
        Fetch::Idle => {
            let how = l.pick([
                "文件保存到 /sdcard/Download，名字取网址里最后一段。",
                "檔案儲存到 /sdcard/Download，名字取網址裡最後一段。",
                "Files are saved in /sdcard/Download, named after the last part of the address.",
            ]);
            widgets::footnote(&mut c, t, f, y, how)
        }
        Fetch::Running { done, total } => {
            let share = total.map(|t| (*done as f32 / (t.max(1) as f32)).clamp(0.0, 1.0));
            let detail = match (total, share) {
                (Some(total), Some(share)) => {
                    let (done_s, total_s, pct) = (mb(*done), mb(*total), percent(share));
                    tr!(l, "{done_s} / {total_s}（{pct}%）", "{done_s} / {total_s}（{pct}%）", "{done_s} of {total_s} ({pct}%)")
                }
                _ => mb(*done),
            };
            widgets::progress_card(&mut c, t, f, y, l.pick(["正在下载", "正在下載", "Downloading"]), &detail, share)
        }
        Fetch::Done { path, bytes } => {
            let (where_to, size_s) = (path.clone(), mb(*bytes));
            let detail = tr!(l, "已保存 {where_to}（{size_s}）", "已儲存 {where_to}（{size_s}）", "Saved {where_to} ({size_s})");
            widgets::info_card(&mut c, t, f, y, &[(l.pick(["完成", "完成", "Done"]), detail)])
        }
        Fetch::Failed(why) => widgets::info_card(&mut c, t, f, y, &[(l.pick(["失败", "失敗", "Failed"]), why.clone())]),
        Fetch::Stopped => widgets::info_card(&mut c, t, f, y, &[(l.pick(["已停止", "已停止", "Stopped"]), String::new())]),
    };
    let title = l.pick(FETCH);
    let enter = match running {
        true => l.pick(["停止下载", "停止下載", "Stop"]),
        false => l.pick(["开始下载", "開始下載", "Download"]),
    };
    // The keys and the spelling bar are held in a dock at the foot of the
    // screen, so the field, the code and the state of a download scroll under
    // them rather than taking the whole screen with them.
    let dock = keys_dock(t, f, s, enter);
    if let Some(rect) = field.clear {
        out.push(Hit { rect, action: Action::ClearFetchUrl });
    }
    let c = fit(c, t, end);
    Frame { dock: Some(dock), caret: Some(field.caret), ..Frame::new(t, c, out, title, top) }
}

pub fn advanced(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    // A long page: fifteen rows reach past one screen, and a page cut to one
    // screen has nothing to scroll, so the last rows would be out of reach.
    let mut c = long_page(cx);
    let top = t.m.content_top;
    // Every day's work first -- the data partition, the partitions, the files'
    // labels and a shell -- then what reaches the network, then what checks or
    // records the phone, and last what repairs it.
    let tools = widgets::list(
        &mut c,
        t,
        f,
        top,
        &[
            Row::new(l.pick(DECRYPT)).icon(BLUE, Icon::Lock).trail(Trail::Value(fmt::data(l, &cx.state.data))),
            Row::new(l.pick(["挂载", "掛載", "Mount"])).icon(SLATE, Icon::Mount),
            Row::new(l.pick(TERMINAL)).icon(SLATE, Icon::Terminal),
            Row::new(l.pick(PARTITIONS)).icon(RED, Icon::Mount),
            Row::new(l.pick(IMAGE_BACKUP)).icon(BLUE, Icon::Backup),
            Row::new(l.pick(FILESYSTEMS)).icon(RED, Icon::Mount),
            Row::new(l.pick(AVB)).icon(GREEN, Icon::Shield),
            Row::new(l.pick(["Root", "Root", "Root"])).icon(ORANGE, Icon::KernelSu),
            Row::new(l.pick(SERVE)).icon(BLUE, Icon::Globe),
            Row::new(l.pick(FETCH)).icon(GREEN, Icon::Globe),
            Row::new(l.pick(UPDATE)).icon(GREEN, Icon::Layers).trail(Trail::Value(update_value(cx))),
            Row::new(l.pick(LOGS)).icon(ORANGE, Icon::Log),
            Row::new(l.pick(["U 盘管理", "USB 隨身碟管理", "USB drive"])).icon(PURPLE, Icon::UsbDrive),
            Row::new(l.pick(RELABEL)).icon(PURPLE, Icon::Shield),
            Row::new(l.pick(POWER_OFF)).icon(SLATE, Icon::Power),
            Row::new(l.pick(TASKS)).icon(BLUE, Icon::Layers),
        ],
    );
    let out = hits(
        &tools.rows,
        &[
            Action::Open(Screen::Decrypt),
            Action::Open(Screen::Mount),
            Action::Open(Screen::Terminal),
            Action::Open(Screen::Partitions),
            Action::Open(Screen::ImageBackup),
            Action::Open(Screen::Filesystems),
            Action::Open(Screen::Avb),
            Action::Open(Screen::Root),
            Action::Open(Screen::Serve),
            Action::Open(Screen::Fetch),
            Action::Open(Screen::Update),
            Action::Open(Screen::Logs),
            Action::FilesOtg,
            Action::Open(Screen::Relabel),
            Action::Open(Screen::PowerOff),
            Action::Open(Screen::Tasks),
        ],
    );
    Frame::new(t, fit(c, t, tools.bottom), out, l.pick(ADVANCED), top)
}

/// The modules the phone's own system loads — KernelSU's and Magisk's keep them
/// the same way — each with the switch that turns it on or off.
pub fn modules(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut rows: Vec<Row> = Vec::new();
    let mut acts: Vec<Action> = Vec::new();
    match &s.modules {
        None => rows.push(Row::new(l.pick(["正在读取…", "正在讀取…", "Reading…"]))),
        Some(Err(why)) => {
            let mut row = Row::new(l.pick(["读取失败", "讀取失敗", "Not read"]));
            row.sub = Some(why.as_str());
            rows.push(row);
        }
        Some(Ok(list)) if list.is_empty() => rows.push(Row::new(l.pick(["没有安装模块", "沒有安裝模組", "No modules installed"]))),
        Some(Ok(list)) => {
            for (k, m) in list.iter().enumerate() {
                let mut row = Row::new(&m.name).icon(if m.enabled { BLUE } else { SLATE }, Icon::Layers).trail(Trail::Switch(m.enabled));
                // 副标题：版本与作者；关了的、标了待删除的也说清楚。
                let off = l.pick(["已关闭", "已關閉", "Off"]);
                let removing = l.pick(["下次启动时移除", "下次啟動時移除", "removed next start"]);
                row.sub = Some(match (m.enabled, m.removing, m.note.is_empty()) {
                    (false, true, _) => removing,
                    (false, false, _) => off,
                    (true, true, _) => removing,
                    (true, false, true) => m.about.as_str(),
                    (true, false, false) => m.note.as_str(),
                });
                rows.push(row);
                acts.push(Action::ModuleOpen(k));
            }
        }
    }
    // 角落里那颗「仓库」：去取模块清单。放在返回箭头右边，免得压住它。
    let mut out = Vec::new();
    let pill_h = t.m.dp(widgets::PILL_DP);
    let pill_x = t.m.back_x + t.m.back_w + t.m.dp(14.0);
    let pill_y = t.m.back_cy - pill_h / 2.0;
    // 一枚「安装」：去文件管理器挑 zip。
    let install = l.pick(["安装", "安裝", "Install"]);
    let wid = widgets::pill(&mut c, t, f, pill_x, pill_y, install, t.pal.search, t.pal.text);
    out.push(Hit { rect: Rect::new(pill_x, pill_y, wid, pill_h), action: Action::Open(Screen::Files) });
    // 一枚「搜索模块」：只看这个列表，不搜功能。放在「安装」右边，靠右对齐。
    let search_label = l.pick(["搜索模块", "搜尋模組", "Search modules"]);
    let w = widgets::pill_width(&c, t, f, search_label);
    let x = t.m.w as f32 - t.m.margin - w;
    let wd = widgets::pill(&mut c, t, f, x, pill_y, search_label, t.pal.search, t.pal.text);
    out.push(pill_hit(t, x, pill_y, wd, pill_h, Action::SearchModules));
    let listed = widgets::list(&mut c, t, f, top, &rows);
    let rows_hits = hits(&listed.rows, &acts);
    out.extend(rows_hits);
    // 装得再多也能滚到，页脚不贴着最后一行。
    let bottom = widgets::footnote(&mut c, t, f, listed.bottom, l.pick(["点一个模块看它是什么，或者把它关掉", "點一個模組看它是什麼，或者把它關掉", "Open a module to see what it is, or turn it off here"]));
    Frame::new(t, fit(c, t, bottom), out, l.pick(MODULES), top)
}

/// One module: what it says of itself, and the two things that can be done
/// with it — switched off or on, and taken away at the next start.
pub fn module_info(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let at = s.module_at.unwrap_or(0);
    let picked = match &s.modules {
        Some(Ok(list)) => list.get(at),
        _ => None,
    };
    let Some(m) = picked else {
        return Frame::new(t, c, Vec::new(), l.pick(MODULES), top);
    };
    let state_note = match (m.enabled, m.removing) {
        (true, true) => l.pick(["已开启 · 下次启动时移除", "已開啟 · 下次啟動時移除", "On · removed next start"]),
        (true, false) => l.pick(["已开启", "已開啟", "On"]),
        (false, _) => l.pick(["已关闭", "已關閉", "Off"]),
    };
    let mut facts: Vec<(&str, String)> = vec![(l.pick(["版本", "版本", "Version"]), m.note.clone())];
    if !m.about.is_empty() {
        facts.push((l.pick(["说明", "說明", "About"]), m.about.clone()));
    }
    facts.push((l.pick(["状态", "狀態", "State"]), state_note.to_string()));
    facts.push((l.pick(["位置", "位置", "Where"]), format!("/data/adb/modules/{}", m.id)));
    let y = widgets::info_card(&mut c, t, f, top, &facts);
    // 开关一张卡片，卸载另一张：卸载要重启才生效，也不该和开关并排放着。
    let switch_label = if m.enabled { l.pick(["关闭模块", "關閉模組", "Turn it off"]) } else { l.pick(["开启模块", "開啟模組", "Turn it on"]) };
    let turned = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(switch_label).icon(BLUE, Icon::Layers)]);
    let remove_label = if m.removing { l.pick(["取消卸载", "取消卸載", "Keep it"]) } else { l.pick(["卸载模块", "卸載模組", "Uninstall"]) };
    let removed = widgets::list(&mut c, t, f, turned.bottom + t.m.card_gap, &[Row::new(remove_label).icon(RED, Icon::Wipe).tint(t.pal.danger)]);
    let mut out = hits(&turned.rows, &[Action::ModuleToggle(at)]);
    out.extend(hits(&removed.rows, &[Action::ModuleRemove(at)]));
    let note = match m.removing {
        true => l.pick(["这个模块会在下次启动时被移除。再点一次就留下它。", "這個模組會在下次啟動時被移除。再點一次就留下它。", "This module is taken away at the next start. Choose it again to keep it."]),
        false => l.pick(["卸载要重启后才生效，重启前可以取消。", "卸載要重啟後才生效，重啟前可以取消。", "Uninstalling takes effect at the next start, and can be undone until then."]),
    };
    let bottom = widgets::footnote(&mut c, t, f, removed.bottom, note);
    Frame::new(t, fit(c, t, bottom), out, l.pick(MODULES), top)
}

const MODULES: Text = [
    "模块管理",
    "模組管理",
    "Modules",
    "Modules",
    "Modules",
    "Modules",
    "Modules",
    "Modules",
    "Modules",
    "Modules",
    "Modules",
    "Modules",
    "Modules",
    "Modules",
    "Modules",
];
const DATA_APPS: Text = [
    "应用管理",
    "應用程式管理",
    "Apps",
    "Apps",
    "Apps",
    "Apps",
    "Apps",
    "Apps",
    "Apps",
    "Apps",
    "Apps",
    "Apps",
    "Apps",
    "Apps",
    "Apps",
];
const SYSTEM_APPS: Text = [
    "系统应用",
    "系統應用程式",
    "System apps",
    "System apps",
    "System apps",
    "System apps",
    "System apps",
    "System apps",
    "System apps",
    "System apps",
    "System apps",
    "System apps",
    "System apps",
    "System apps",
    "System apps",
];
/// What the page says it is for: it lists, and says where the same apps are
/// handled by something that can actually handle them.
const SYSTEM_APPS_FOOTNOTE: [&str; 3] = [
    "这里只列出系统应用，不在这里停用或卸载：这两件事要系统运行着才做得到，请用系统设置里的应用管理。",
    "這裡只列出系統應用程式，不在這裡停用或卸載：這兩件事要系統運行著才做得到，請用系統設定裡的應用程式管理。",
    "These are listed only: neither is switched off nor taken away here, since both need the system running. The system's own app settings do that.",
];
const RELABEL: Text = [
    "修复 SELinux 上下文",
    "修復 SELinux 上下文",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
    "Fix SELinux contexts",
];
const FILESYSTEMS: Text = [
    "文件系统",
    "檔案系統",
    "Filesystems",
    "Filesystems",
    "Filesystems",
    "Filesystems",
    "Filesystems",
    "Filesystems",
    "Filesystems",
    "Filesystems",
    "Filesystems",
    "Filesystems",
    "Filesystems",
    "Filesystems",
    "Filesystems",
];
const PARTITIONS: Text = [
    "分区管理",
    "分割區管理",
    "Partitions",
    "Partitions",
    "Partitions",
    "Partitions",
    "Partitions",
    "Partitions",
    "Partitions",
    "Partitions",
    "Partitions",
    "Partitions",
    "Partitions",
    "Partitions",
    "Partitions",
];
const IMAGE_BACKUP: Text = [
    "镜像备份",
    "映像檔備份",
    "Image backup",
    "Image backup",
    "Image backup",
    "Image backup",
    "Image backup",
    "Image backup",
    "Image backup",
    "Image backup",
    "Image backup",
    "Image backup",
    "Image backup",
    "Image backup",
    "Image backup",
];
const POWER_OFF: Text = [
    "关机",
    "關機",
    "Power off",
    "Power off",
    "Power off",
    "Power off",
    "Power off",
    "Power off",
    "Power off",
    "Power off",
    "Power off",
    "Power off",
    "Power off",
    "Power off",
    "Power off",
];
const TASKS: Text = [
    "任务",
    "任務",
    "Tasks",
    "Tasks",
    "Tasks",
    "Tasks",
    "Tasks",
    "Tasks",
    "Tasks",
    "Tasks",
    "Tasks",
    "Tasks",
    "Tasks",
    "Tasks",
    "Tasks",
];
const CHARGE: Text = [
    "充电设置",
    "充電設定",
    "Charging",
    "Charging",
    "Charging",
    "Charging",
    "Charging",
    "Charging",
    "Charging",
    "Charging",
    "Charging",
    "Charging",
    "Charging",
    "Charging",
    "Charging",
];
const AVB: Text = [
    "AVB 校验",
    "AVB 驗證",
    "AVB verification",
    "AVB verification",
    "AVB verification",
    "AVB verification",
    "AVB verification",
    "AVB verification",
    "AVB verification",
    "AVB verification",
    "AVB verification",
    "AVB verification",
    "AVB verification",
    "AVB verification",
    "AVB verification",
];
/// A pill's tap target: the pill and a little about it, so that a finger that
/// lands beside it still finds it.
fn pill_hit(t: &Theme, x: f32, y: f32, w: f32, h: f32, action: Action) -> Hit {
    let pad = (t.m.dp(44.0) - h).max(0.0) / 2.0;
    Hit { rect: Rect::new(x - t.m.dp(4.0), y - pad, w + t.m.dp(8.0), h + pad * 2.0), action }
}

/// The rows one page's entries make when filtered by `query`, and what each
/// one opens: an app by its name or package, a module by its name or folder.
fn filtered<'a>(s: &'a State, scope: SearchScope, query: &str) -> (Vec<Row<'a>>, Vec<Action>) {
    let mut rows = Vec::new();
    let mut acts = Vec::new();
    match scope {
        SearchScope::Apps => {
            let Some(Ok(list)) = &s.apps else { return (rows, acts) };
            for (k, a) in list.iter().enumerate() {
                if !a.label.to_lowercase().contains(query) && !a.package.to_lowercase().contains(query) {
                    continue;
                }
                let mut row = Row::new(&a.label).icon(PURPLE, Icon::Layers);
                row.sub = Some(a.package.as_str());
                rows.push(row);
                acts.push(Action::DataAppOpen(k));
            }
        }
        SearchScope::Modules => {
            let Some(Ok(list)) = &s.modules else { return (rows, acts) };
            for (k, m) in list.iter().enumerate() {
                if !m.name.to_lowercase().contains(query) && !m.id.to_lowercase().contains(query) {
                    continue;
                }
                // The same row the modules page draws, switch and all: the
                // search is a way of finding one, not another list of them.
                let mut row = Row::new(&m.name).icon(if m.enabled { BLUE } else { SLATE }, Icon::Layers).trail(Trail::Switch(m.enabled));
                row.sub = Some(if m.note.is_empty() { m.about.as_str() } else { m.note.as_str() });
                rows.push(row);
                acts.push(Action::ModuleOpen(k));
            }
        }
        // The whole of what the phone offers is searched by the page itself,
        // which has the names of the features to look through; the file
        // manager filters its own folder, which this page does not hold.
        SearchScope::Features | SearchScope::Files => {}
    }
    (rows, acts)
}

/// How many rows of `sample` fit on a page whose content starts at `top`.
///
/// Measured rather than counted from a number: a row with a subtitle under
/// its name is taller than one without, and the subtitle wraps, so the count
/// has to come from the row the page actually draws.
fn rows_that_fit(t: &Theme, f: &Fonts, top: f32, sample: &Row) -> usize {
    let probe = Canvas::band(t.m.w, 1, 0..1, 0..t.m.w, t.pal.bg);
    let h = widgets::one_height(&probe, t, f, sample);
    // Half a row is left over at the foot, so that the last one is not cut in
    // half by the bottom margin.
    let room = t.m.h as f32 - top - t.m.margin * 2.0 - h / 2.0;
    ((room / h).floor() as usize).max(1)
}

/// The pager a page turned a screen at a time draws at its top right: the
/// count, and a way back and on.
///
/// The three cells keep their places whether or not they can be used, and a
/// cell that cannot be is drawn faint and takes no tap. Moving them would
/// put the one tapped after a turn under the finger that had just turned the
/// page, and the page would run back and forth.
#[allow(clippy::too_many_arguments)]
fn pager(t: &Theme, f: &Fonts, c: &mut Canvas, out: &mut Vec<Hit>, l: Lang, y: f32, page: usize, pages: usize, turn: fn(usize) -> Action) {
    let ph = t.m.dp(widgets::PILL_DP);
    if pages <= 1 {
        let only = l.pick(["只有一页", "只有一頁", "One page"]);
        let w = widgets::pill_width(c, t, f, only);
        let x = t.m.w as f32 - t.m.margin - w;
        widgets::pill(c, t, f, x, y, only, t.pal.search, t.pal.text);
        return;
    }
    let gaps = t.m.dp(widgets::PILL_GAP_DP);
    let counter = format!("{}/{}", page + 1, pages);
    let back = l.pick(["上一页", "上一頁", "Newer"]);
    let fwd = l.pick(["下一页", "下一頁", "Older"]);
    let w_prev = widgets::pill_width(c, t, f, back);
    let w_next = widgets::pill_width(c, t, f, fwd);
    let w_counter = c.text_width(&f.value, &counter, t.m.sub_px) + t.m.dp(8.0);
    let x_next = t.m.w as f32 - t.m.margin - w_next;
    let x_counter = x_next - gaps - w_counter;
    let x_prev = x_counter - gaps - w_prev;
    let faint = t.pal.subtitle;
    // The right third of the screen turns the page on: a finger that lands a
    // little wide still turns it.
    if page + 1 < pages {
        out.push(Hit { rect: Rect::new(t.m.w as f32 * 0.66, y - t.m.dp(12.0), t.m.w as f32 * 0.34 - t.m.margin, ph + t.m.dp(24.0)), action: turn(page + 1) });
    }
    if page > 0 {
        let wd = widgets::pill(c, t, f, x_prev, y, back, t.pal.search, t.pal.text);
        // The way back stops at 62% of the width, leaving the rest to the way
        // on.
        let left = x_prev - t.m.dp(16.0);
        let right = (x_prev + wd + t.m.dp(8.0)).min(t.m.w as f32 * 0.62);
        out.push(Hit { rect: Rect::new(left, y - t.m.dp(10.0), (right - left).max(wd), ph + t.m.dp(20.0)), action: turn(page - 1) });
    } else {
        widgets::pill(c, t, f, x_prev, y, back, t.pal.search, faint);
    }
    c.text_center(&f.value, &counter, x_counter + w_counter / 2.0, y + ph / 2.0, t.m.sub_px, faint);
    if page + 1 < pages {
        widgets::pill(c, t, f, x_next, y, fwd, t.pal.search, t.pal.text);
    } else {
        widgets::pill(c, t, f, x_next, y, fwd, t.pal.search, faint);
    }
}

/// The apps the owner installed, newest first, a screen at a time.
///
/// Ported from skyward-pudding v0.3.15. A screen holds what fits rather than
/// the whole list on one tall page: a phone with a hundred and eighty apps
/// makes a page tens of thousands of pixels tall, which is a canvas of over a
/// hundred megabytes for every frame.
pub fn data_apps(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let top = t.m.content_top;
    let mut c = page(cx);
    let mut out = Vec::new();
    let mut sample = Row::new("Ag").icon(PURPLE, Icon::Layers);
    sample.sub = Some("com.example.app");
    let per = rows_that_fit(t, f, top, &sample);
    let mut rows: Vec<Row> = Vec::new();
    let mut acts: Vec<Action> = Vec::new();
    let mut pages = 1usize;
    match &s.apps {
        None => rows.push(Row::new(l.pick(["正在读取…", "正在讀取…", "Reading…"]))),
        Some(Err(why)) => {
            let mut row = Row::new(l.pick(["读取失败", "讀取失敗", "Not read"]));
            row.sub = Some(why.as_str());
            rows.push(row);
        }
        Some(Ok(list)) if list.is_empty() => rows.push(Row::new(l.pick(["数据分区里没有应用", "資料分區裡沒有應用程式", "No apps in the data partition"]))),
        Some(Ok(list)) => {
            pages = list.len().div_ceil(per).max(1);
            let page = s.app_page.min(pages - 1);
            let (from, to) = (page * per, ((page + 1) * per).min(list.len()));
            for (k, a) in list.iter().enumerate().take(to).skip(from) {
                let mut row = Row::new(&a.label).icon(PURPLE, Icon::Layers);
                row.sub = Some(a.package.as_str());
                rows.push(row);
                acts.push(Action::DataAppOpen(k));
            }
        }
    }
    let page = s.app_page.min(pages.saturating_sub(1));
    // The search pill stands on the line the back arrow is on; the pager on
    // the title's.
    let ph = t.m.dp(widgets::PILL_DP);
    let py = t.m.back_cy - ph / 2.0;
    let pager_y = t.m.title_top + t.m.title_px * 0.46 - ph / 2.0;
    let mut x = t.m.w as f32 - t.m.margin;
    let search_label = l.pick(["搜索应用", "搜尋應用程式", "Search apps"]);
    let w = widgets::pill_width(&c, t, f, search_label);
    x -= w;
    let wd = widgets::pill(&mut c, t, f, x, py, search_label, t.pal.search, t.pal.text);
    out.push(pill_hit(t, x, py, wd, ph, Action::SearchApps));
    // What the system's own partitions hold is a page of its own, and this is
    // the way to it.
    let sys_label = l.pick(SYSTEM_APPS);
    let w = widgets::pill_width(&c, t, f, sys_label);
    x -= t.m.dp(widgets::PILL_GAP_DP) + w;
    let wd = widgets::pill(&mut c, t, f, x, py, sys_label, t.pal.search, t.pal.subtitle);
    out.push(pill_hit(t, x, py, wd, ph, Action::Open(Screen::SystemApps)));
    pager(t, f, &mut c, &mut out, l, pager_y, page, pages, Action::AppPage);
    let listed = widgets::list(&mut c, t, f, top, &rows);
    out.extend(hits(&listed.rows, &acts));
    Frame::new(t, c, out, l.pick(DATA_APPS), top)
}

/// The apps the system's own partitions hold, to look at and nothing more.
///
/// Neither is taken off nor switched off from here. A system app's folder is on
/// a read-only partition, and turning one off means writing the package
/// manager's own record, which is a binary format that writing by hand breaks.
/// The system's own app settings do both, and they do it with the system
/// running; there is no honest way to do it from the recovery, so the page does
/// not pretend to.
///
/// Ported from skyward-pudding v0.3.15.
pub fn system_apps(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let top = t.m.content_top;
    let mut c = page(cx);
    let mut sample = Row::new("Ag").icon(SLATE, Icon::Layers);
    sample.sub = Some("/system/app");
    let per = rows_that_fit(t, f, top, &sample);
    let mut rows: Vec<Row> = Vec::new();
    let mut pages = 1usize;
    match &s.sys_apps {
        None => rows.push(Row::new(l.pick(["正在读取…", "正在讀取…", "Reading…"]))),
        Some(list) if list.is_empty() => rows.push(Row::new(l.pick(["没有读到系统应用", "沒有讀到系統應用程式", "No system apps read"]))),
        Some(list) => {
            pages = list.len().div_ceil(per).max(1);
            let page = s.sys_page.min(pages - 1);
            let (from, to) = (page * per, ((page + 1) * per).min(list.len()));
            for a in list.iter().take(to).skip(from) {
                let mut row = Row::new(&a.label).icon(SLATE, Icon::Layers);
                row.sub = Some(a.package.as_str());
                rows.push(row);
            }
        }
    }
    let page = s.sys_page.min(pages.saturating_sub(1));
    let ph = t.m.dp(widgets::PILL_DP);
    let pager_y = t.m.title_top + t.m.title_px * 0.46 - ph / 2.0;
    let mut out = Vec::new();
    pager(t, f, &mut c, &mut out, l, pager_y, page, pages, Action::SysPage);
    let listed = widgets::list(&mut c, t, f, top, &rows);
    let bottom = widgets::footnote(&mut c, t, f, listed.bottom, l.pick(SYSTEM_APPS_FOOTNOTE));
    Frame::new(t, fit(c, t, bottom), out, l.pick(SYSTEM_APPS), top)
}

/// One app: what it is, and the swipe that takes it off.
///
/// Ported from skyward-pudding v0.3.15.
pub fn data_app_info(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let at = s.app_at.unwrap_or(0);
    let picked = match &s.apps {
        Some(Ok(list)) => list.get(at),
        _ => None,
    };
    let Some(a) = picked else {
        return Frame::new(t, c, Vec::new(), l.pick(DATA_APPS), top);
    };
    let facts: Vec<(&str, String)> = vec![
        (l.pick(["包名", "套件名稱", "Package"]), a.package.clone()),
        (l.pick(["名称", "名稱", "Name"]), a.label.clone()),
        (l.pick(["位置", "位置", "Where"]), a.dir.clone()),
        (l.pick(["占用", "佔用", "Size"]), fmt::file_size(a.size)),
        (l.pick(["安装", "安裝", "Installed"]), fmt::datetime(l, a.installed, s.zone())),
        (l.pick(["更新", "更新", "Updated"]), fmt::datetime(l, a.updated, s.zone())),
    ];
    let y = widgets::info_card(&mut c, t, f, top, &facts);
    // An app in the system's own partitions is not taken off from here: the
    // system would not start.
    let mut out = Vec::new();
    let bottom = if a.system {
        let row = Row::new(l.pick(["系统应用，不能这样卸载", "系統應用程式，不能這樣卸載", "A system app: not removed here"])).icon(SLATE, Icon::Shield);
        let listed = widgets::list(&mut c, t, f, y + t.m.card_gap, &[row]);
        widgets::footnote(&mut c, t, f, listed.bottom, l.pick(["这些装在系统分区里，删掉系统就起不来", "這些裝在系統分區裡，刪掉系統就起不來", "These are in the system's partitions; taking one away keeps it from starting"]))
    } else {
        let row = Row::new(l.pick(["卸载应用", "卸載應用程式", "Uninstall"])).icon(RED, Icon::Wipe).tint(t.pal.danger);
        let listed = widgets::list(&mut c, t, f, y + t.m.card_gap, &[row]);
        let (label, danger) = confirm_label(Confirm::DeleteApp, s);
        let track = widgets::confirm(&mut c, t, f, listed.bottom + t.m.dp(24.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::DeleteApp) });
        widgets::footnote(&mut c, t, f, track.y + track.h, l.pick(["卸载会删掉它的安装目录和 /data/data 里的数据", "卸載會刪掉它的安裝目錄和 /data/data 裡的資料", "Uninstalling removes its folder and what it kept in /data/data"]))
    };
    Frame::new(t, fit(c, t, bottom), out, l.pick(DATA_APPS), top)
}

/// Android Verified Boot's checks in the running slot: what they are, and
/// where the bootloader is unlocked, switches for them and the saved
/// top-level vbmeta image to back up or restore.
pub fn avb(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    // The page carries what turning a check off costs and how a phone that
    // no longer starts is brought back, which runs past three screens.
    let mut c = long_page_of(cx, 4);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let title = l.pick(AVB);
    let state = match &s.avb {
        None => {
            let y = widgets::list(&mut c, t, f, top, &[Row::new(l.pick(["正在读取 AVB 状态", "正在讀取 AVB 狀態", "Reading AVB state"])).trail(Trail::None)]).bottom;
            return Frame::new(t, fit(c, t, y), out, title, top);
        }
        Some(Err(why)) => {
            let y = widgets::list(&mut c, t, f, top, &[Row::new(l.pick(["无法读取 AVB 状态", "無法讀取 AVB 狀態", "Couldn't read AVB state"])).trail(Trail::None)]).bottom;
            let y = widgets::footnote(&mut c, t, f, y, why);
            let retry = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(["重试", "重試", "Try again"])).trail(Trail::None)]);
            out.extend(hits(&retry.rows, &[Action::ReadAvb]));
            return Frame::new(t, fit(c, t, retry.bottom), out, title, top);
        }
        Some(Ok(state)) => state,
    };
    let lock = match state.unlocked {
        Some(true) => l.pick(["已解锁", "已解鎖", "Unlocked"]),
        Some(false) => l.pick(["已锁定", "已鎖定", "Locked"]),
        None => l.pick(["未知", "未知", "Unknown"]),
    };
    let backup = l.pick(if state.backup { ["已有备份", "已有備份", "Backed up"] } else { ["尚未备份", "尚未備份", "Not backed up"] });
    let rollback = state.rollback_index.to_string();
    let y = widgets::info_card(
        &mut c,
        t,
        f,
        top,
        &[
            (l.pick(["Bootloader", "Bootloader", "Bootloader"]), lock.to_string()),
            (l.pick(["回滚索引", "回滾索引", "Rollback index"]), rollback),
            (l.pick(["vbmeta 备份", "vbmeta 備份", "vbmeta backup"]), backup.to_string()),
        ],
    );
    let y = widgets::section(&mut c, t, f, y, l.pick(["校验开关", "驗證開關", "Checks"]));
    let actions = widgets::list(
        &mut c,
        t,
        f,
        y,
        &[
            Row::new(l.pick(["dm-verity", "dm-verity", "dm-verity"])).trail(Trail::Switch(state.verity)),
            Row::new(l.pick(["启动验证", "啟動驗證", "Boot verification"])).trail(Trail::Switch(state.verification)),
            Row::new(l.pick(["备份 vbmeta", "備份 vbmeta", "Back up vbmeta"])).icon(GREEN, Icon::Backup).trail(Trail::Arrow),
            Row::new(l.pick(["恢复 vbmeta", "還原 vbmeta", "Restore vbmeta"])).icon(ORANGE, Icon::Restore).trail(Trail::Arrow),
        ],
    );
    if state.unlocked == Some(true) {
        out.extend(hits(&actions.rows[..2], &[Action::ToggleAvb(AvbCheck::Verity), Action::ToggleAvb(AvbCheck::Verification)]));
        out.extend(hits(&actions.rows[2..3], &[Action::BackupVbmeta]));
        if state.backup {
            out.extend(hits(&actions.rows[3..4], &[Action::RestoreVbmeta]));
        }
    }
    let mut y = actions.bottom;
    y = avb_warning(&mut c, cx, y);
    match state.unlocked {
        Some(true) => {}
        Some(false) => {
            y = widgets::footnote(
                &mut c,
                t,
                f,
                y,
                l.pick([
                    "Bootloader 已锁定，不能修改 AVB 或备份、恢复 vbmeta。",
                    "Bootloader 已鎖定，不能修改 AVB 或備份、還原 vbmeta。",
                    "The bootloader is locked, so AVB and vbmeta cannot be changed or backed up.",
                ]),
            );
        }
        None => {
            y = widgets::footnote(
                &mut c,
                t,
                f,
                y,
                l.pick(["无法确定 Bootloader 是否解锁，已禁用修改。", "無法確定 Bootloader 是否解鎖，已停用修改。", "The bootloader's lock state is unknown, so changes are disabled."]),
            );
        }
    }
    if let Some(notice) = &s.avb_notice {
        y = widgets::footnote(&mut c, t, f, y + t.m.card_gap, notice);
    }
    Frame::new(t, fit(c, t, y), out, title, top)
}

/// What turning a check off does to the system already installed, and how a
/// phone that no longer starts is brought back: the page that offers the
/// switch is the one place someone can read this before using it, and the
/// one place they can still reach once it has gone wrong.
fn avb_warning(c: &mut Canvas, cx: &Ctx, y: f32) -> f32 {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let title = l.pick([
        "关闭校验后官方系统将无法启动",
        "關閉驗證後官方系統將無法啟動",
        "Turning a check off stops the official system from starting",
    ]);
    let detail = l.pick([
        "这不是签名问题，解锁与否都一样：系统的 init 在 verity 关闭时不挂载分区，只能线刷恢复。",
        "這不是簽名問題，解鎖與否都一樣：系統的 init 在 verity 關閉時不掛載分割區，只能線刷還原。",
        "The signature is not the reason, and unlocking does not help: with verity off the system's init mounts no partition, and only a fastboot package brings it back.",
    ]);
    let y = widgets::list(c, t, f, y + t.m.card_gap, &[standing(title, RED, Icon::Systems, detail)]).bottom;
    let points = [
        l.pick([
            "改用第三方系统时关闭校验才有意义，官方系统请保持开启。",
            "改用第三方系統時關閉驗證才有意義，官方系統請保持開啟。",
            "Turning a check off only helps with a system other than the phone's own; leave it on for the official one.",
        ]),
        l.pick([
            "卡在 fastboot 时，先试 fastboot set_active 切到另一个槽位，再 fastboot continue；另一个槽位装过系统才有用。",
            "卡在 fastboot 時，先試 fastboot set_active 切換到另一個槽位，再 fastboot continue；另一個槽位裝過系統才有用。",
            "Stuck at fastboot: try fastboot set_active for the other slot, then fastboot continue. It helps only where that slot holds a system.",
        ]),
        l.pick([
            "仍进不去就刷完整的线刷包，它会一并刷回 vbmeta 并清掉 misc 里的启动指令，这样才恢复得了。",
            "仍進不去就刷完整的線刷包，它會一併刷回 vbmeta 並清掉 misc 裡的啟動指令，這樣才還原得了。",
            "If it still does not start, flash the full fastboot package: it writes vbmeta back and clears the start request in misc, which is what makes the phone start again.",
        ]),
        l.pick([
            "只重刷 vbmeta 往往不够，而且要先确认当前槽位：fastboot getvar current-slot，刷的必须是这个槽位的 vbmeta。",
            "只重刷 vbmeta 往往不夠，而且要先確認目前槽位：fastboot getvar current-slot，刷的必須是這個槽位的 vbmeta。",
            "Flashing vbmeta alone often is not enough, and the slot has to match: fastboot getvar current-slot, then flash that slot's vbmeta.",
        ]),
        l.pick([
            "改动前请先备份 vbmeta，备份读不回来时不要改动。",
            "改動前請先備份 vbmeta，備份讀不回來時不要改動。",
            "Back vbmeta up before changing it, and do not change it when the backup cannot be read back.",
        ]),
    ];
    let heading = l.pick(["关闭之前", "關閉之前", "Before turning it off"]);
    widgets::notes(c, t, f, y + t.m.card_gap, heading, &points)
}

/// Giving the data partition's files the SELinux labels the installed
/// system's rules give them: what it does, and a swipe to start it; then
/// how far it has come, a way to stop it, and how it ended.
pub fn relabel(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let title = l.pick(texts::page::RELABEL);
    let Some(job) = &s.relabel else {
        let mut y = widgets::paragraph(&mut c, t, f, top, l.pick(texts::relabel::WHAT));
        let note = match s.data {
            Data::Open | Data::Locked { .. } => None,
            Data::Erased => Some(texts::relabel::DATA_ERASED),
            Data::Closed | Data::Opening | Data::Failed(_) => Some(texts::relabel::OPEN_FIRST),
        };
        if let Some(note) = note {
            y = widgets::footnote(&mut c, t, f, y, l.pick(note));
        }
        let (label, danger) = confirm_label(Confirm::Relabel, cx.state);
        let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::Relabel) });
        return Frame::new(t, c, out, title, top);
    };
    let repair = Verb::of(&crate::app::PartKind::Repair);
    let heading = match &job.outcome {
        None => repair.going(l),
        Some(JobEnd::Done) => repair.complete(l),
        Some(JobEnd::Stopped) => l.pick(texts::common::STOPPED).into(),
        Some(JobEnd::Failed(_)) => repair.failed(l),
    };
    let detail = match (job.total, job.checked) {
        (0, _) if job.outcome.is_none() => l.pick(texts::installing::GETTING_READY).to_string(),
        (total, checked) => texts::relabel::checked(l, checked, total),
    };
    let y = widgets::progress_card(&mut c, t, f, top, &heading, &detail, Some(share(job.checked, job.total)));
    match &job.outcome {
        None => {
            let label = l.pick(if job.stopping { texts::common::STOPPING } else { texts::common::STOP });
            let stop = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopRelabel]));
            }
        }
        Some(end) => {
            let changed = job.changed;
            let note = match end {
                JobEnd::Done if changed == 0 => l
                    .pick(["全部文件的 SELinux 上下文都正确，无需修复。", "全部檔案的 SELinux 上下文都正確，無需修復。", "Every file's SELinux context is right, so nothing needed fixing."])
                    .to_string(),
                JobEnd::Done => {
                    tr!(l, "已按规则重新设置 {changed} 项的 SELinux 上下文。", "已按規則重新設定 {changed} 項的 SELinux 上下文。", "{changed} entries got the SELinux context the rules give them.")
                }
                JobEnd::Stopped => tr!(l, "已停止，已重新设置 {changed} 项。", "已停止，已重新設定 {changed} 項。", "Stopped, with {changed} entries set again."),
                JobEnd::Failed(why) => tr!(l, "修复失败：{why}", "修復失敗：{why}", "Repair failed: {why}"),
            };
            let y = widgets::footnote(&mut c, t, f, y, &note);
            let finish = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(texts::common::DONE)).trail(Trail::None)]);
            out.extend(hits(&finish.rows, &[Action::Back]));
        }
    }
    Frame { back: job.outcome.is_some(), ..Frame::new(t, c, out, title, top) }
}

/// The partitions whose filesystems can be repaired or made afresh as
/// another, each with what it holds and its size.
/// Every partition the phone's table lists, with how large it is and what it
/// holds: the table itself, and the way into what can be done with each.
///
/// The partitions are the ones the filesystem page reads, since they are read
/// from the same table; a partition that cannot be formatted still shows here
/// with what it holds, and opening it goes where the mount page goes.
pub fn partitions(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["分区表", "分割區表", "Partition table"]));
    let bottom = match &s.filesystems {
        None => widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["正在读取分区", "正在讀取分割區", "Reading the partitions"])).trail(Trail::None)]).bottom,
        Some(Err(why)) => {
            let y = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["无法读取分区", "無法讀取分割區", "Couldn't read the partitions"])).trail(Trail::None)]).bottom;
            widgets::footnote(&mut c, t, f, y, why)
        }
        Some(Ok(rows)) if rows.is_empty() => widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["本机没有分区", "本機沒有分割區", "This phone lists no partitions"])).trail(Trail::None)]).bottom,
        Some(Ok(rows)) => {
            // What each partition holds, and how large it is: the two things
            // the table says of every one of them.
            let subs: Vec<String> = rows.iter().map(|r| format!("{} · {}", fs_held(l, r, &s.data), fmt::bytes(r.size))).collect();
            let shown: Vec<Row> = rows.iter().zip(&subs).map(|(r, sub)| Row::new(&r.name).subtitle(sub)).collect();
            let list = widgets::list(&mut c, t, f, y, &shown);
            out.extend(hits(&list.rows, &(0..rows.len()).map(Action::OpenFilesystem).collect::<Vec<_>>()));
            let note = l.pick([
                "列出本机分区表中的全部分区，大小由分区表读得。",
                "列出本機分割區表中的全部分割區，大小由分割區表讀得。",
                "Every partition in this phone's table, each with the size the table gives it.",
            ]);
            widgets::footnote(&mut c, t, f, list.bottom, note)
        }
    };
    Frame::new(t, fit(c, t, bottom), out, l.pick(PARTITIONS), top)
}

pub fn filesystems(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(texts::mount::PARTITIONS));
    let bottom = match &s.filesystems {
        None => widgets::list(&mut c, t, f, y, &[Row::new(l.pick(texts::filesystems::READING)).trail(Trail::None)]).bottom,
        Some(Err(why)) => {
            let y = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(texts::mount::UNREADABLE)).trail(Trail::None)]).bottom;
            widgets::footnote(&mut c, t, f, y, why)
        }
        Some(Ok(rows)) if rows.is_empty() => widgets::list(&mut c, t, f, y, &[Row::new(l.pick(["暂无可修复的分区", "沒有可修復的分割區", "No partitions to repair"])).trail(Trail::None)]).bottom,
        Some(Ok(rows)) => {
            let subs: Vec<String> = rows.iter().map(|r| format!("{} · {}", fs_held(l, r, &s.data), fmt::bytes(r.size))).collect();
            let shown: Vec<Row> = rows.iter().zip(&subs).map(|(r, sub)| Row::new(&r.name).subtitle(sub)).collect();
            let list = widgets::list(&mut c, t, f, y, &shown);
            out.extend(hits(&list.rows, &(0..rows.len()).map(Action::OpenFilesystem).collect::<Vec<_>>()));
            widgets::footnote(&mut c, t, f, list.bottom, l.pick(texts::filesystems::NOTE))
        }
    };
    Frame::new(t, fit(c, t, bottom), out, l.pick(texts::page::FILESYSTEMS), top)
}

/// What a partition holds, as its row and page name it: the data
/// partition's is read only once it is open, and until then the row says
/// where it stands, as the home page does.
fn fs_held(l: Lang, row: &FsRow, data: &Data) -> String {
    match (&row.filesystem, row.name == DATA_PARTITION) {
        (Some(filesystem), _) => filesystem.clone(),
        (None, true) => fmt::data(l, data).into(),
        (None, false) => l.pick(texts::filesystems::UNRECOGNIZED).into(),
    }
}

/// Partitions written out whole as image files, images written back over the
/// partitions they came from, and the images kept.
///
/// Two lists share the page: the partitions an image can be taken from, and
/// the images already saved. A pill at the top says which is showing, and
/// work under way takes the page over, so nothing can be started while one is
/// still going.
pub fn image_backup(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    // The list of partitions is as long as the phone has partitions, which can
    // be more than three screens' worth: the sheet is made to fit them. A row
    // with a subtitle is taller than a bare one, so the height is measured
    // from a row of the list rather than assumed.
    let rows = match (&s.image_parts, s.image_showing_parts) {
        (Ok(parts), true) => parts.len(),
        _ => 0,
    };
    let row_h = match &s.image_parts {
        Ok(parts) if !parts.is_empty() => {
            let probe = Canvas::band(t.m.w, 1, 0..1, 0..t.m.w, t.pal.bg);
            let size = fmt::bytes(parts[0].size);
            widgets::one_height(&probe, t, f, &Row::new(&parts[0].name).subtitle(&size))
        }
        _ => t.m.row_h,
    };
    let screens = screens_for(t, t.m.content_top, row_h * 6.0, rows, row_h);
    let mut c = long_page_of(cx, screens);
    let mut out = Vec::new();
    let top = t.m.content_top;
    // Work under way, or how it last ended, takes the page over: a second
    // image started over a partition being written would spoil both.
    if let Some(job) = s.image_job.as_ref() {
        return image_job_page(cx, job);
    }
    // The two lists, as a pair of rows that say which is showing.
    let showing = |parts: bool| s.image_showing_parts == parts;
    let choose = widgets::list(
        &mut c,
        t,
        f,
        top,
        &[
            Row::new(l.pick(["分区", "分割區", "Partitions"])).subtitle(l.pick(["把分区写成镜像文件", "把分割區寫成映像檔", "Write a partition out as an image"])).trail(Trail::Check(showing(true))),
            Row::new(l.pick(["镜像", "映像檔", "Images"])).subtitle(l.pick(["写回分区，或删除", "寫回分割區，或刪除", "Write one back, or delete it"])).trail(Trail::Check(showing(false))),
        ],
    );
    out.extend(hits(&choose.rows, &[Action::ImageShows(true), Action::ImageShows(false)]));
    // A section label between the two rows that choose a list and the list
    // itself: without one the list's first row sits against the row above it
    // and reads as a third choice rather than the start of another list.
    let label = match s.image_showing_parts {
        true => l.pick(["分区", "分割區", "Partitions"]),
        false => l.pick(["镜像", "映像檔", "Images"]),
    };
    let mut y = widgets::section(&mut c, t, f, choose.bottom, label);
    y = if s.image_showing_parts {
        image_parts_list(cx, &mut c, &mut out, y)
    } else {
        image_files_list(cx, &mut c, &mut out, y)
    };
    Frame::new(t, fit(c, t, y), out, l.pick(IMAGE_BACKUP), top)
}

/// The partitions an image can be taken from, each with its size and a way to
/// write it out.
fn image_parts_list(cx: &Ctx, c: &mut Canvas, out: &mut Vec<Hit>, y: f32) -> f32 {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    match &s.image_parts {
        Err(why) => {
            let y = widgets::list(c, t, f, y, &[Row::new(l.pick(["无法读取分区", "無法讀取分割區", "Couldn't read the partitions"])).trail(Trail::None)]).bottom;
            widgets::footnote(c, t, f, y, why)
        }
        Ok(parts) if parts.is_empty() => widgets::list(c, t, f, y, &[Row::new(l.pick(["本机没有分区", "本機沒有分割區", "This phone lists no partitions"])).trail(Trail::None)]).bottom,
        Ok(parts) => {
            let sizes: Vec<String> = parts.iter().map(|p| fmt::bytes(p.size)).collect();
            let shown: Vec<Row> = parts.iter().zip(&sizes).map(|(p, size)| Row::new(&p.name).subtitle(size).trail(Trail::Arrow)).collect();
            let list = widgets::list(c, t, f, y, &shown);
            out.extend(hits(&list.rows, &(0..parts.len()).map(Action::ExportImage).collect::<Vec<_>>()));
            let note = l.pick([
                "选一个分区，把它的全部内容写成一个镜像文件。分区有多大，镜像就有多大。",
                "選一個分割區，把它的全部內容寫成一個映像檔。分割區有多大，映像檔就有多大。",
                "Pick a partition to write all of it out as one image file. The image is as large as the partition.",
            ]);
            widgets::footnote(c, t, f, list.bottom, note)
        }
    }
}

/// The images already saved, each with the partition it came from, how large
/// it is, and a way to write it back or delete it.
fn image_files_list(cx: &Ctx, c: &mut Canvas, out: &mut Vec<Hit>, y: f32) -> f32 {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    match &s.image_files {
        Err(why) => {
            let y = widgets::list(c, t, f, y, &[Row::new(l.pick(["无法读取镜像", "無法讀取映像檔", "Couldn't read the images"])).trail(Trail::None)]).bottom;
            widgets::footnote(c, t, f, y, why)
        }
        Ok(files) if files.is_empty() => widgets::list(c, t, f, y, &[Row::new(l.pick(["还没有保存的镜像", "還沒有儲存的映像檔", "No images saved yet"])).trail(Trail::None)]).bottom,
        Ok(files) => {
            let subs: Vec<String> = files
                .iter()
                .map(|file| format!("{} · {} · {}", file.partition, fmt::bytes(file.size), fmt::date(l, file.made, s.zone())))
                .collect();
            let shown: Vec<Row> = files.iter().zip(&subs).map(|(file, sub)| Row::new(&file.name).subtitle(sub).trail(Trail::Arrow)).collect();
            let list = widgets::list(c, t, f, y, &shown);
            out.extend(hits(&list.rows, &(0..files.len()).map(Action::ImportImage).collect::<Vec<_>>()));
            // Each image's own row opens it; the rows under the list are where
            // one is deleted, since deleting asks nothing of the partition.
            let mut below = list.bottom;
            let deletes_word = l.pick(["删除", "刪除", "Delete"]);
            let names: Vec<String> = files.iter().map(|file| format!("{deletes_word} {}", file.name)).collect();
            let actions: Vec<Row> = names.iter().map(|name| Row::new(name).tint(t.pal.danger)).collect();
            if !actions.is_empty() {
                let y = widgets::section(c, t, f, below, l.pick(ACTIONS));
                let deletes = widgets::list(c, t, f, y, &actions);
                out.extend(hits(&deletes.rows, &(0..files.len()).map(Action::DeleteImage).collect::<Vec<_>>()));
                below = deletes.bottom;
            }
            let note = l.pick([
                "写回前会先核对整份镜像，与保存时不一致就一个字节也不写。",
                "寫回前會先核對整份映像檔，與儲存時不一致就一個位元組也不寫。",
                "An image is checked whole before it is written back; if it does not match what was saved, not a byte is written.",
            ]);
            widgets::footnote(c, t, f, below, note)
        }
    }
}

/// Work on an image under way, or how it ended: the partition, how far it has
/// come, and a way to stop it while it runs.
fn image_job_page(cx: &Ctx, job: &ImageJob) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let exporting = job.kind == ImageFlow::Export;
    let title = match exporting {
        true => ["正在写入镜像", "正在寫入映像檔", "Writing the image"],
        false => ["正在写回镜像", "正在寫回映像檔", "Writing the image back"],
    };
    let done = job.done.min(job.of);
    // While it runs, the detail says what it is doing; once it has ended, how
    // much of the partition went through.
    let doing = match (exporting, job.outcome.is_some()) {
        (true, false) => l.pick(["正在读取分区", "正在讀取分割區", "Reading the partition"]),
        (false, false) => l.pick(["正在核对镜像", "正在核對映像檔", "Checking the image"]),
        _ => l.pick(["镜像", "映像檔", "Image"]),
    };
    let detail = format!("{doing} · {} · {} / {}", job.partition, fmt::bytes(done), fmt::bytes(job.of));
    let share = match job.of {
        0 => 0.0,
        of => done as f32 / of as f32,
    };
    let y = widgets::progress_card(&mut c, t, f, top, l.pick(title), &detail, Some(share));
    match &job.outcome {
        None => {
            let y = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
            let stop = if job.stopping { l.pick(["正在停止…", "正在停止…", "Stopping…"]) } else { l.pick(["停止", "停止", "Stop"]) };
            let list = widgets::list(&mut c, t, f, y, &[Row::new(stop).tint(t.pal.danger)]);
            if !job.stopping {
                out.extend(hits(&list.rows, &[Action::StopImage]));
            }
            let note = if exporting {
                l.pick(["写入完成前，请不要关机。", "寫入完成前，請不要關機。", "Don't power off until it's written."])
            } else {
                l.pick(["写回完成前，请不要关机，也不要断开数据线。", "寫回完成前，請不要關機，也不要拔除傳輸線。", "Don't power off or unplug the cable until it's written back."])
            };
            widgets::footnote(&mut c, t, f, list.bottom, note)
        }
        Some(end) => {
            let (said, ink, icon) = match end {
                ImageEnd::Exported(path) => (format!("{} {path}", l.pick(["镜像已保存到", "映像檔已儲存到", "Saved the image to"])), GREEN, Icon::Install),
                ImageEnd::Imported(part) => (format!("{} {part}", l.pick(["已将镜像写回", "已將映像檔寫回", "Wrote the image back over"])), GREEN, Icon::Install),
                ImageEnd::Deleted(path) => (format!("{} {path}", l.pick(["已删除", "已刪除", "Deleted"])), SLATE, Icon::Wipe),
                ImageEnd::Stopped => (l.pick(["已停止", "已停止", "Stopped"]).to_string(), SLATE, Icon::Wipe),
                ImageEnd::Mismatched { wanted, found } => (
                    format!(
                        "{} {wanted} {} {found}",
                        l.pick([
                            "镜像与保存时不一致，未写入任何内容。保存时为",
                            "映像檔與儲存時不一致，未寫入任何內容。儲存時為",
                            "The image does not match what was saved, so nothing was written. It was",
                        ]),
                        l.pick(["，现在读到", "，現在讀到", ", and reads"]),
                    ),
                    RED,
                    Icon::Shield,
                ),
                ImageEnd::Failed(why) => (format!("{} {why}", l.pick(["失败：", "失敗：", "Failed: "])), RED, Icon::Wipe),
            };
            let failed = matches!(end, ImageEnd::Mismatched { .. } | ImageEnd::Failed(_));
            let heading = if failed {
                l.pick(["未完成", "未完成", "Not done"])
            } else if matches!(end, ImageEnd::Stopped) {
                l.pick(["已停止", "已停止", "Stopped"])
            } else {
                l.pick(["完成", "完成", "Done"])
            };
            // How it went stands apart from the progress above it, and what to
            // do next is a row of its own: the card saying how it went is not
            // itself a button, and the page is left with `<`.
            let y = widgets::section(&mut c, t, f, y, l.pick(["结果", "結果", "Result"]));
            let said_row = widgets::list(&mut c, t, f, y, &[standing(heading, ink, icon, &said)]);
            let y = widgets::section(&mut c, t, f, said_row.bottom, l.pick(ACTIONS));
            let back = l.pick(["返回", "返回", "Back"]);
            let list = widgets::list(&mut c, t, f, y, &[Row::new(back)]);
            out.extend(hits(&list.rows, &[Action::Back]));
            list.bottom
        }
    };
    Frame::new(t, fit(c, t, y), out, l.pick(IMAGE_BACKUP), top)
}

/// One partition: what it holds, what can be done to it, and a swipe to do
/// what is chosen.
pub fn filesystem(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let Some(Ok(rows)) = &s.filesystems else { unreachable!("a partition's page opens from the list read") };
    let row = &rows[s.fs_chosen];
    let data = row.name == DATA_PARTITION;
    let facts = [
        (l.pick(texts::filesystems::MOUNT_POINT), row.mount_point.clone()),
        (l.pick(texts::filesystems::FILESYSTEM), fs_held(l, row, &s.data)),
        (l.pick(texts::common::SIZE), fmt::bytes(row.size)),
    ];
    let y = widgets::info_card(&mut c, t, f, top, &facts);
    let y = widgets::section(&mut c, t, f, y, l.pick(texts::common::ACTIONS));
    let chosen = |action: &FsAction| s.fs_action == *action;
    let remakes: Vec<String> = row.others.iter().map(|fs| texts::filesystems::change_to(l, fs)).collect();
    let made: Vec<String> = row.others.iter().map(|_| remade(row, s)).collect();
    let repair = Row::new(l.pick(texts::filesystems::REPAIR))
        .subtitle(l.pick(texts::filesystems::REPAIR_NOTE))
        .trail(Trail::Check(chosen(&FsAction::Repair)));
    let mut shown = vec![repair];
    let mut actions = vec![Action::ChooseFs(FsChoice::Repair)];
    for (k, fs) in row.others.iter().enumerate() {
        shown.push(Row::new(&remakes[k]).subtitle(&made[k]).trail(Trail::Check(chosen(&FsAction::Remake(fs.clone())))));
        actions.push(Action::ChooseFs(FsChoice::Remake(k)));
    }
    let list = widgets::list(&mut c, t, f, y, &shown);
    out.extend(hits(&list.rows, &actions));
    let mut y = list.bottom;
    if row.others.is_empty() {
        y = widgets::footnote(&mut c, t, f, y, l.pick(texts::filesystems::CANT_CHANGE));
    }
    if data {
        y = widgets::footnote(&mut c, t, f, y, l.pick(texts::filesystems::DATA_CLOSES));
    }
    if !row.others.is_empty() {
        y = widgets::footnote(&mut c, t, f, y, l.pick(texts::filesystems::CANT_UNDO));
    }
    let (label, danger) = confirm_label(Confirm::Filesystem, s);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Filesystem) });
    Frame::new(t, fit(c, t, track.y + track.h), out, partition_title(l, &row.name), top)
}

/// The title of the page of a partition the filesystem page lists.
fn partition_title(l: Lang, name: &str) -> &'static str {
    l.pick(match name {
        DATA_PARTITION => texts::wipe::part::DATA,
        "metadata" => texts::wipe::part::METADATA,
        "cache" => texts::wipe::part::CACHE,
        _ => texts::flash::PARTITION,
    })
}

/// What making `row` afresh as another filesystem erases: the data
/// partition goes as a factory reset goes, with /metadata and the cache
/// partition beside it.
fn remade(row: &FsRow, s: &State) -> String {
    match (row.name == DATA_PARTITION, s.cache_partition) {
        (true, true) => s.lang.pick(texts::filesystems::FORMATS_DATA_AND_CACHE).into(),
        (true, false) => s.lang.pick(texts::filesystems::FORMATS_DATA).into(),
        (false, _) => texts::filesystems::formats(s.lang, &row.name),
    }
}

/// Work on whole partitions: each partition and where it stands, what the
/// program said of the last, and once it has ended, how. What the system
/// asked for leaves on its own once it went through every partition, and
/// offers where to go otherwise; what the user chose goes back to its page.
pub fn part_work(cx: &Ctx) -> Frame {
    use crate::app::{ItemEnd, ItemState, PartKind};
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let work = s.part_work.as_ref().expect("the page of partition work shows work");
    let mut c = long_page(cx);
    let (mut out, mut rows) = (Vec::new(), None);
    let top = t.m.content_top;
    let verb = Verb::of(&work.kind);
    // Work on one partition fails; work through several may get partway.
    let (title, several) = match &work.kind {
        PartKind::Disable => (texts::parts::title::WIPE, true),
        PartKind::Unshare => (texts::parts::title::UNSHARE, true),
        PartKind::Repair => (texts::parts::title::REPAIR, false),
        PartKind::Remake(_) => (texts::parts::title::FORMAT, false),
    };
    let failed = |state: &ItemState| matches!(state, ItemState::Ended(ItemEnd::Failed(_)));
    let short = matches!(work.outcome, Some(Err(_))) || work.items.iter().any(|(_, state)| failed(state));
    let heading = match (&work.outcome, short, several) {
        (None, _, _) => verb.going(l),
        (Some(_), false, _) => verb.complete(l),
        (Some(_), true, true) => verb.incomplete(l),
        (Some(_), true, false) => verb.failed(l),
    };
    let mut y = widgets::section_first(&mut c, t, f, top, &heading);
    if !work.items.is_empty() {
        let states: Vec<String> = work
            .items
            .iter()
            .map(|(_, state)| match state {
                ItemState::Waiting if work.outcome.is_some() => verb.not_done(l),
                ItemState::Waiting => l.pick(texts::common::WAITING).into(),
                ItemState::Working => verb.going(l),
                ItemState::Ended(end) => fmt::item_end(l, end, verb),
            })
            .collect();
        let held: Vec<Option<&str>> = work
            .items
            .iter()
            .map(|(_, state)| match state {
                ItemState::Ended(ItemEnd::NotShared(fs)) if !fs.is_empty() => Some(fs.as_str()),
                _ => None,
            })
            .collect();
        let shown: Vec<Row> = work
            .items
            .iter()
            .zip(&states)
            .zip(&held)
            .map(|(((name, _), state), held)| {
                let row = Row::new(name).trail(Trail::Text(state));
                match held {
                    Some(fs) => row.subtitle(fs),
                    None => row,
                }
            })
            .collect();
        y = widgets::list(&mut c, t, f, y, &shown).bottom;
    }
    let leaving = match s.asked.shutdown_after {
        true => texts::parts::leaving::POWER_OFF,
        false => texts::parts::leaving::REBOOT,
    };
    let mut notes: Vec<String> = Vec::new();
    match &work.outcome {
        None if work.asked => notes.push(l.pick(leaving).into()),
        None => notes.push(verb.hold_on(l)),
        Some(Err(stop)) => {
            let (what, detail) = fmt::part_stop(l, stop, &cx.info.codename);
            notes.push(what);
            notes.extend(detail);
        }
        Some(Ok(())) => {}
    }
    for (name, state) in &work.items {
        if let ItemState::Ended(ItemEnd::Failed(Some(why))) = state {
            notes.push(texts::common::with_reason(l, verb.failed_on(l, name), why));
        }
    }
    for note in &notes {
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if let (true, Some(reason)) = (work.asked, &s.asked.reason) {
        y = widgets::info_card(&mut c, t, f, y + t.m.dp(12.0), &[(l.pick(texts::installing::REASON), reason.clone())]);
    }
    match (&work.outcome, work.asked) {
        (None, _) => {}
        (Some(_), true) if !short => {}
        // The system's work ran without the slot's partitions, which going
        // on in the recovery brings up first, as leaving fastboot does.
        (Some(_), true) => {
            let below = widgets::section(&mut c, t, f, y, l.pick(texts::common::ACTIONS));
            let rows: Vec<Row> = AFTERWARDS.iter().map(|&(title, _)| Row::new(l.pick(title)).trail(Trail::None)).collect();
            let actions = AFTERWARDS.map(|(_, action)| if action == Action::Dismiss { Action::EnterRecovery } else { action });
            let choices = widgets::list(&mut c, t, f, below, &rows);
            out.extend(hits(&choices.rows, &actions));
            y = choices.bottom;
        }
        (Some(_), false) => {
            let finish = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(texts::common::DONE)).trail(Trail::None)]);
            out.extend(hits(&finish.rows, &[Action::Back]));
            y = finish.bottom;
        }
    }
    if !work.output.is_empty() {
        let below = widgets::section(&mut c, t, f, y, l.pick(texts::parts::OUTPUT));
        rows = Some(Rows { edges: widgets::output_edges(&c, t, f, below, &work.output), action: None });
        y = below;
    }
    let back = work.outcome.is_some() && !work.asked;
    Frame { rows, back, ..Frame::new(t, fit(c, t, y), out, l.pick(title), top) }
}

/// The system on the phone now; the newest system the phone maker's
/// server knows, and downloading its full package to install it; and
/// where an update the system installed stands, merging it into the slot it
/// went to: how far a merge under way has come, and a swipe to merge one
/// that waits there for the system to start it.
pub fn update(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut out = Vec::new();
    if let Standing::Merging(done) = s.update {
        let share = done as f32;
        let detail = texts::common::share_done(l, percent(share));
        let y = widgets::progress_card(&mut c, t, f, top, l.pick(texts::snapshot::MERGING), &detail, Some(share));
        let y = widgets::footnote(&mut c, t, f, y, l.pick(texts::update::MERGING_NOTE));
        return Frame::new(t, fit(c, t, y), out, l.pick(texts::page::UPDATE), top);
    }
    let mut y = top;
    if let Some(v) = &i.os_version {
        y = widgets::section_first(&mut c, t, f, y, l.pick(texts::common::THIS_PHONE));
        y = widgets::info_card(&mut c, t, f, y, &[(l.pick(texts::update::CURRENT_VERSION), v.clone())]);
    }
    if s.update_server {
        let online_update = l.pick(texts::update::ONLINE);
        y = if y == top { widgets::section_first(&mut c, t, f, y, online_update) } else { widgets::section(&mut c, t, f, y, online_update) };
        y = online(&mut c, cx, y, &mut out);
    }
    let next = match &s.boot_slots {
        Some(Ok(slots)) => Some(slots.active),
        _ => None,
    };
    let (title, detail) = fmt::update(l, &s.update, next);
    let color = match s.update {
        Standing::None => SLATE,
        Standing::Merged => GREEN,
        Standing::Failed(_) | Standing::Unknown => RED,
        _ => ORANGE,
    };
    let mut row = Row::new(title).icon(color, Icon::Layers).trail(Trail::None);
    if let Some(detail) = &detail {
        row = row.subtitle(detail);
    }
    let merging = l.pick(texts::update::MERGING);
    y = if y == top { widgets::section_first(&mut c, t, f, y, merging) } else { widgets::section(&mut c, t, f, y, merging) };
    y = widgets::list(&mut c, t, f, y, &[row]).bottom;
    if s.update == Standing::Waiting {
        let (label, danger) = confirm_label(Confirm::Merge, cx.state);
        let track = widgets::confirm(&mut c, t, f, y + t.m.dp(28.0), label, danger, s.confirm);
        out.push(Hit { rect: track, action: Action::Confirm(Confirm::Merge) });
        y = widgets::footnote(&mut c, t, f, track.y + track.h, l.pick(texts::update::CANT_GO_BACK));
    }
    Frame::new(t, fit(c, t, y), out, l.pick(texts::page::UPDATE), top)
}

/// Asking the update server for the newest system, then its full package:
/// downloading it, how far that has come, and installing it once it is
/// whole; then what to note before installing it.
fn online(c: &mut Canvas, cx: &Ctx, y: f32, out: &mut Vec<Hit>) -> f32 {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let release = match &s.check {
        Check::Asking => {
            return widgets::list(c, t, f, y, &[Row::new(l.pick(CHECKING)).trail(Trail::None)]).bottom;
        }
        Check::Answered(Some(r)) => r,
        other => {
            let mut row = Row::new(l.pick(texts::update::CHECK));
            if !s.online() {
                row = row.subtitle(l.pick(texts::update::WLAN_FIRST));
            }
            let check = widgets::list(c, t, f, y, &[row]);
            out.extend(hits(&check.rows, &[Action::CheckUpdate]));
            return match other {
                Check::Failed(why) => widgets::footnote(c, t, f, check.bottom, &fmt::unanswered(l, why)),
                Check::Answered(None) => widgets::footnote(c, t, f, check.bottom, l.pick(texts::update::NONE_KNOWN)),
                _ => check.bottom,
            };
        }
    };
    let newer = i.os_version.as_deref() != Some(release.version.as_str());
    let title = if newer { texts::update::new_version_named(l, &release.version) } else { l.pick(texts::update::UP_TO_DATE).to_string() };
    let size = match release.length {
        Some(length) => fmt::file_size(length),
        None => fmt::package_size(&release.size),
    };
    let package = texts::update::full_package(l, size);
    let mut y = widgets::list(c, t, f, y, &[Row::new(&title).icon(GREEN, Icon::Layers).subtitle(&package).trail(Trail::None)]).bottom;
    match (&s.download, s.have) {
        (Some(job), _) if job.outcome.is_none() => {
            let (heading, detail, share) = match job.progress {
                None => (l.pick(texts::installing::GETTING_READY), String::new(), None),
                Some(p) => downloaded(l, p, job.rate),
            };
            y = widgets::progress_card(c, t, f, y + t.m.card_gap, heading, &detail, share);
            let label = l.pick(if job.stopping { texts::update::PAUSING } else { texts::update::PAUSE });
            let stop = widgets::list(c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopDownload]));
            }
            y = stop.bottom;
        }
        (_, Have::Whole(_)) => {
            let row = Row::new(l.pick(texts::page::INSTALL)).subtitle(l.pick(texts::update::DOWNLOADED_TO_STORAGE));
            let install = widgets::list(c, t, f, y + t.m.card_gap, &[row]);
            out.extend(hits(&install.rows, &[Action::InstallDownloaded]));
            y = install.bottom;
        }
        (job, have) => {
            match job.as_ref().and_then(|j| j.outcome.as_ref()) {
                Some(DownloadEnd::Failed(why)) => y = widgets::footnote(c, t, f, y, &fmt::download_failure(l, why)),
                Some(DownloadEnd::Stopped) => y = widgets::footnote(c, t, f, y, l.pick(texts::update::PAUSED)),
                _ => {}
            }
            let title = l.pick(if matches!(have, Have::Part(_)) { texts::update::RESUME } else { texts::update::DOWNLOAD });
            let had = match have {
                Have::Part(done) => Some(texts::update::downloaded(l, fmt::file_size(done))),
                _ => None,
            };
            let sub = if !s.data.storage_open() {
                Some(l.pick(texts::decrypt::NEEDED))
            } else if !s.online() {
                Some(l.pick(texts::update::WLAN_FIRST))
            } else {
                had.as_deref()
            };
            let row = Row::new(title);
            let download = widgets::list(c, t, f, y + t.m.card_gap, &[sub.map_or(row, |sub| row.subtitle(sub))]);
            out.extend(hits(&download.rows, &[Action::Download]));
            y = download.bottom;
        }
    }
    let saved = tr!(
        l,
        "完整更新包将保存到\"/sdcard/{DOWNLOADS}\"中，安装后不会自动删除。",
        "完整更新檔將儲存到「/sdcard/{DOWNLOADS}」中，安裝後不會自動刪除。",
        "The full OTA package is saved in “/sdcard/{DOWNLOADS}”, and stays there after it's installed."
    );
    let points = [
        l.pick([
            "安装系统更新不会删除用户数据，建议在安装前进行备份。",
            "安裝系統更新不會刪除使用者資料，建議在安裝前進行備份。",
            "Installing a system update deletes no user data. Backing up first is still wise.",
        ]),
        saved.as_str(),
        l.pick(["下载中断后可继续下载，已下载的部分会保留。", "下載中斷後可繼續下載，已下載的部分會保留。", "A download cut short can be resumed, keeping what was downloaded."]),
        l.pick([
            "更新后，系统会在后台进行一系列优化，期间可能出现发热、耗电增加等情况，1 ~ 3 天内将自行恢复。",
            "更新後，系統會在背景進行一系列最佳化，期間可能出現發熱、耗電增加等情況，1 ~ 3 天內將自行恢復。",
            "After the update the system optimizes itself in the background, and the phone may run warm or use more battery for 1 to 3 days.",
        ]),
    ];
    widgets::notes(c, t, f, y + t.m.card_gap, l.pick(NOTES), &points)
}

/// What a progress card says of a download, and the share of it done:
/// what an earlier download left being checked, or the rest coming at
/// `rate` bytes a second.
fn downloaded(l: Lang, progress: Progress, rate: u64) -> (&'static str, String, Option<f32>) {
    match progress {
        Progress::Checking { done, total } => (l.pick(CHECKING_PART), format!("{} / {}", fmt::file_size(done), fmt::file_size(total)), Some(share(done, total))),
        Progress::Downloading { done, total } => (l.pick(DOWNLOADING), format!("{} / {} · {}/s", fmt::file_size(done), fmt::file_size(total), fmt::file_size(rate)), Some(share(done, total))),
    }
}

/// Skyward's own update: the build running now; Skyward's newest release
/// on GitHub, and updating to it; then what to note before updating.
pub fn skyward_update(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut out = Vec::new();
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["本机", "本機", "This phone"]));
    let y = widgets::info_card(&mut c, t, f, y, &[(l.pick(["当前版本", "目前版本", "Current version"]), RUNNING.to_string())]);
    let y = widgets::section(&mut c, t, f, y, l.pick(["在线更新", "線上更新", "Online update"]));
    let y = skyward_online(&mut c, cx, y, &mut out);
    // What the release says it changed, cut to the few lines that fit here.
    if let Some(newest) = cx.state.skyward_newest()
        && !newest.notes.is_empty()
    {
        let points: Vec<&str> = newest.notes.iter().map(String::as_str).collect();
        let y2 = widgets::notes(&mut c, t, f, y + t.m.card_gap, l.pick(["本次更新", "本次更新", "What changed"]), &points);
        return Frame::new(t, fit(c, t, y2), out, l.pick(SKYWARD_UPDATE), top);
    }
    let points = [
        l.pick([
            "新版本从 GitHub 下载，校验 SHA-256 后写入 recovery 分区，不会改动系统和用户数据。",
            "新版本從 GitHub 下載，驗證 SHA-256 後寫入 recovery 分割區，不會改動系統和使用者資料。",
            "The new version is downloaded from GitHub and goes into the recovery partitions once its SHA-256 checks out. The system and user data are left as they are.",
        ]),
        l.pick([
            "新版本将写入下次启动所用槽位的 recovery 分区；另一个槽位装有 Skyward 时，也会一并更新。",
            "新版本將寫入下次啟動所用槽位的 recovery 分割區；另一個槽位裝有 Skyward 時，也會一併更新。",
            "The new version goes into the recovery partition of the slot that starts next, and into the other slot's where it holds Skyward.",
        ]),
        l.pick(["更新完成后，重启到恢复模式即可使用新版本。", "更新完成後，重新啟動到復原模式即可使用新版本。", "Once updated, reboot to recovery to use the new version."]),
    ];
    let y = widgets::notes(&mut c, t, f, y + t.m.card_gap, l.pick(NOTES), &points);
    Frame::new(t, fit(c, t, y), out, l.pick(SKYWARD_UPDATE), top)
}

/// Asking GitHub for Skyward's newest release, then updating to it: its
/// build for the phone downloaded and written into the recovery partitions,
/// how far that has come, and how it ended.
fn skyward_online(c: &mut Canvas, cx: &Ctx, y: f32, out: &mut Vec<Hit>) -> f32 {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let gap = t.m.card_gap;
    let newest = match &s.skyward_check {
        SkywardCheck::Asking => {
            return widgets::list(c, t, f, y, &[Row::new(l.pick(CHECKING)).trail(Trail::None)]).bottom;
        }
        SkywardCheck::Answered(n) => n,
        other => {
            let mut row = Row::new(l.pick(texts::update::CHECK));
            if !s.online() {
                row = row.subtitle(l.pick(texts::update::WLAN_FIRST));
            }
            let check = widgets::list(c, t, f, y, &[row]);
            out.extend(hits(&check.rows, &[Action::CheckSkyward]));
            return match other {
                SkywardCheck::Failed(why) => widgets::footnote(c, t, f, check.bottom, &fmt::unanswered(l, why)),
                _ => check.bottom,
            };
        }
    };
    let version = &newest.version;
    let (title, detail) = match (newest.newer, &newest.build) {
        (true, Some(build)) => {
            let size = fmt::file_size(build.size);
            (tr!(l, "新版本 {version}", "新版本 {version}", "New version {version}"), tr!(l, "安装包 {size}", "安裝包 {size}", "Package {size}"))
        }
        (true, None) => (tr!(l, "新版本 {version}", "新版本 {version}", "New version {version}"), l.pick(["没有本机的安装包", "沒有本機的安裝包", "No package for this phone"]).to_string()),
        (false, _) => (l.pick(["已是最新版本", "已是最新版本", "Up to date"]).to_string(), tr!(l, "最新发布的版本为 {version}", "最新發佈的版本為 {version}", "The newest release is {version}")),
    };
    let y = widgets::list(c, t, f, y, &[Row::new(&title).icon(GREEN, Icon::Install).subtitle(&detail).trail(Trail::None)]).bottom;
    let y = if newest.notes.is_empty() {
        y
    } else {
        let notes: Vec<&str> = newest.notes.iter().map(String::as_str).collect();
        widgets::notes(c, t, f, y + gap, l.pick(["更新内容", "更新內容", "What's new"]), &notes)
    };
    if !newest.newer || newest.build.is_none() {
        return y;
    }
    match &s.skyward_job {
        Some(job) if job.outcome.is_none() => {
            let (heading, detail, share) = match job.stage {
                None => (l.pick(texts::installing::GETTING_READY), String::new(), None),
                Some(SkywardStage::Download(p)) => downloaded(l, p, job.rate),
                Some(SkywardStage::Write { done, total }) => {
                    let part = share(done, total);
                    (l.pick(texts::skyward_update::WRITING), texts::common::share_done(l, percent(part)), Some(part))
                }
            };
            let y = widgets::progress_card(c, t, f, y + gap, heading, &detail, share);
            if matches!(job.stage, Some(SkywardStage::Write { .. })) {
                return widgets::footnote(c, t, f, y, l.pick(texts::skyward_update::KEEP_POWER));
            }
            let stop = widgets::list(c, t, f, y + gap, &[Row::new(l.pick(if job.stopping { texts::common::STOPPING } else { texts::common::STOP })).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopSkywardUpdate]));
            }
            stop.bottom
        }
        Some(SkywardJob { outcome: Some(SkywardEnd::Updated(slots)), .. }) => {
            let detail = match slots.as_slice() {
                [slot] => texts::skyward_update::in_one(l, version, slot.letter()),
                [a, b] => texts::skyward_update::in_two(l, version, a.letter(), b.letter()),
                other => unreachable!("a phone has two slots, and Skyward went into {other:?}"),
            };
            let y = widgets::list(c, t, f, y + gap, &[standing(l.pick(texts::skyward_update::SUCCEEDED), GREEN, Icon::Install, &detail)]).bottom;
            let restart = widgets::list(c, t, f, y + gap, &[Row::new(l.pick(texts::installed::REBOOT_TO_RECOVERY)).icon(ORANGE, Icon::Power)]);
            out.extend(hits(&restart.rows, &[Action::RebootTo(Target::Recovery)]));
            restart.bottom
        }
        job => {
            let mut y = y;
            match job.as_ref().and_then(|j| j.outcome.as_ref()) {
                Some(SkywardEnd::Stopped) => y = widgets::footnote(c, t, f, y, l.pick(texts::skyward_update::STOPPED)),
                Some(SkywardEnd::Download(why)) => y = widgets::footnote(c, t, f, y, &fmt::skyward_download_failure(l, why)),
                Some(SkywardEnd::NotWritten(why)) => {
                    y = widgets::footnote(c, t, f, y, l.pick(texts::skyward_update::NOT_WRITTEN));
                    y = widgets::footnote(c, t, f, y, why);
                }
                Some(SkywardEnd::WriteFailed(slots, why)) => {
                    let names = fmt::slots(l, slots);
                    let warning = match (l, slots.len()) {
                        (Lang::En, 1) => {
                            format!("{}'s recovery partition was only partly written, so rebooting to recovery may not start Skyward. Update again, and reboot once it's done.", capitalized(l, &names))
                        }
                        _ => tr!(
                            l,
                            "{names} 的 recovery 分区仅写入了部分内容，重启到恢复模式可能无法进入 Skyward。请重新更新，完成后再重启。",
                            "{names} 的 recovery 分割區僅寫入了部分內容，重新啟動到復原模式可能無法進入 Skyward。請重新更新，完成後再重新啟動。",
                            "The recovery partitions of {names} were only partly written, so rebooting to recovery may not start Skyward. Update again, and reboot once it's done."
                        ),
                    };
                    y = widgets::footnote(c, t, f, y, &warning);
                    y = widgets::footnote(c, t, f, y, why);
                }
                Some(SkywardEnd::Updated(_)) => {
                    unreachable!("an update that went in shows how it did")
                }
                None => {}
            }
            let mut row = Row::new(l.pick(texts::skyward_update::DOWNLOAD_AND_UPDATE));
            if !s.online() {
                row = row.subtitle(l.pick(texts::update::WLAN_FIRST));
            }
            let update = widgets::list(c, t, f, y + gap, &[row]);
            out.extend(hits(&update.rows, &[Action::UpdateSkyward]));
            update.bottom
        }
    }
}

/// The WLAN switch; then the network chosen and how joining it goes, and
/// the networks in range, strongest first.
pub fn wlan(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let w = s.wlan.as_ref().expect("the WLAN page shows on a phone whose WLAN is brought up");
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let on = !matches!(w.power, Power::Off | Power::Failed(_));
    // The WLAN's switch, and the two settings that go with it: opened at
    // start, and the password kept beside the network it was joined with.
    let switch = widgets::list(
        &mut c,
        t,
        f,
        top,
        &[Row::new(l.pick(WLAN)).trail(Trail::Switch(on)), Row::new(l.pick(WLAN_AUTO)).trail(Trail::Switch(s.wlan_auto)), Row::new(l.pick(WLAN_KEEP)).trail(Trail::Switch(s.wlan_keep))],
    );
    out.extend(hits(&switch.rows, &[Action::Toggle(Switch::Wlan), Action::Toggle(Switch::WlanAuto), Action::Toggle(Switch::WlanKeep)]));
    let mut y = switch.bottom;
    match &w.power {
        Power::Off => y = widgets::footnote(&mut c, t, f, y, l.pick(texts::wlan::TURN_ON_NOTE)),
        Power::Failed(why) => y = widgets::footnote(&mut c, t, f, y, &texts::wlan::cant_turn_on(l, why)),
        Power::Starting => y = widgets::footnote(&mut c, t, f, y, l.pick(texts::wlan::TURNING_ON_WLAN)),
        Power::On => {
            let locked = |s: Security| !matches!(s, Security::Open | Security::Owe);
            // The network chosen stands out in the accent, as the stock
            // page shows the one it is on.
            if let Some(j) = &w.joined {
                let found = w.networks.iter().find(|n| n.ssid == j.ssid);
                let (signal, bands) = match (&j.stage, found) {
                    (Joining::Connected(link), found) => (Some(link.signal), found.map_or(Bands::of(link.frequency), |n| n.bands)),
                    (_, Some(n)) => (Some(n.signal), n.bands),
                    (_, None) => (None, Bands::default()),
                };
                let tag = fmt::band_tag(bands);
                let mut row = Row::new(&j.name).fan(signal.map_or(0, fmt::signal_level)).subtitle(fmt::joining(l, &j.stage));
                row = row.trail(Trail::Network { locked: locked(j.security) });
                if let Some(tag) = &tag {
                    row = row.tag(tag);
                }
                let joined = widgets::list_toned(&mut c, t, f, y + t.m.card_gap, &[row], &Tone::lit(t));
                out.extend(hits(&joined.rows, &[Action::OpenJoined]));
                out.push(Hit { rect: widgets::round_button(t, joined.rows[0]), action: Action::OpenJoined });
                y = joined.bottom;
                if let Joining::Connected(link) = &j.stage
                    && let Some(note) = link.reach.as_ref().and_then(|r| fmt::reach(l, r))
                {
                    y = widgets::footnote(&mut c, t, f, y, &note);
                }
            }
            let others: Vec<(usize, &Network)> = w.networks.iter().enumerate().filter(|(_, n)| w.joined.as_ref().is_none_or(|j| j.ssid != n.ssid)).collect();
            let nearby = l.pick(["选取附近的 WLAN", "選擇附近的 Wi-Fi 網路", "Choose a nearby network"]);
            let (below, rescan) = widgets::section_action(&mut c, t, f, y, nearby, l.pick(["刷新", "重新整理", "Refresh"]));
            out.push(Hit { rect: rescan, action: Action::Rescan });
            y = below;
            if others.is_empty() {
                y = widgets::footnote(&mut c, t, f, y - t.m.dp(10.0), l.pick(texts::wlan::LOOKING)) + t.m.card_gap;
            }
            let tags: Vec<Option<String>> = others.iter().map(|(_, n)| fmt::band_tag(n.bands)).collect();
            let mut rows: Vec<Row> = others
                .iter()
                .zip(&tags)
                .map(|((_, n), tag)| {
                    let row = Row::new(&n.name).fan(fmt::signal_level(n.signal)).trail(Trail::Network { locked: locked(n.security) });
                    match tag {
                        Some(tag) => row.tag(tag),
                        None => row,
                    }
                })
                .collect();
            // The list ends with adding a network by hand, as the stock one
            // does.
            rows.push(Row::new(l.pick(texts::wlan::ADD_NETWORK)).tint(t.pal.accent).trail(Trail::None));
            let list = widgets::list(&mut c, t, f, y, &rows);
            let mut picks: Vec<Action> = others.iter().map(|(k, _)| Action::JoinNetwork(*k)).collect();
            picks.push(Action::AddNetwork);
            out.extend(hits(&list.rows, &picks));
            // The round buttons come after the rows they sit on, and a touch
            // finds the last control under it.
            for (rect, (k, _)) in list.rows.iter().zip(&others) {
                out.push(Hit { rect: widgets::round_button(t, *rect), action: Action::NetworkDetails(*k) });
            }
            y = list.bottom;
        }
    }
    Frame::new(t, fit(c, t, y), out, l.pick(texts::page::WLAN), top)
}

/// The password of the network chosen: its name, what is typed, a switch
/// to show it, and the keyboard.
pub fn wlan_password(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let network = s.joining.as_ref().expect("the password page is for a network");
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, &network.name);
    let shown = if s.show_password { s.password.clone() } else { "•".repeat(s.password.chars().count()) };
    let (y, caret) = widgets::text_field(&mut c, t, f, y, &shown);
    let show = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(texts::wlan::SHOW_PASSWORD)).trail(Trail::Switch(s.show_password))]);
    out.extend(hits(&show.rows, &[Action::Toggle(Switch::ShowPassword)]));
    keyboard_hits(&mut c, t, f, s, l.pick(texts::wlan::CONNECT), &mut out);
    Frame { caret: Some(caret), ..Frame::new(t, c, out, l.pick(texts::wlan::ENTER_PASSWORD), top) }
}

/// A network added by hand: its name, how it is protected, its password
/// where it takes one with the switch that shows it, and the keyboard,
/// which types into the field touched. The protections open over the page
/// from their row, which alone then answers a touch.
pub fn wlan_add(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let a = s.adding.as_ref().expect("the page adds a network");
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = m.content_top;
    let field = |y: f32| Rect::new(m.margin, y, m.w as f32 - m.margin * 2.0, m.dp(56.0));
    let y = widgets::section_first(&mut c, t, f, top, l.pick(texts::wlan::NETWORK_NAME));
    let (below, mut caret) = widgets::text_field(&mut c, t, f, y, &a.name);
    out.push(Hit { rect: field(y), action: Action::TypeInto(Field::Name) });
    // Held down while its list is open, as the stock row stays.
    let tone = if a.choosing { Tone::page(&t.pressed()) } else { Tone::page(t) };
    let protection = [Row::new(l.pick(texts::wlan::SECURITY_TITLE)).trail(Trail::Choice(fmt::security(l, a.security)))];
    let security = widgets::list_toned(&mut c, t, f, below + m.card_gap, &protection, &tone);
    out.extend(hits(&security.rows, &[Action::ChooseSecurity]));
    if a.security.password() {
        let y = widgets::section(&mut c, t, f, security.bottom, l.pick(texts::wlan::PASSWORD));
        let shown = if s.show_password { s.password.clone() } else { "•".repeat(s.password.chars().count()) };
        let (below, typed) = widgets::text_field(&mut c, t, f, y, &shown);
        out.push(Hit { rect: field(y), action: Action::TypeInto(Field::Password) });
        if a.field == Field::Password {
            caret = typed;
        }
        let show = widgets::list(&mut c, t, f, below + m.card_gap, &[Row::new(l.pick(texts::wlan::SHOW_PASSWORD)).trail(Trail::Switch(s.show_password))]);
        out.extend(hits(&show.rows, &[Action::Toggle(Switch::ShowPassword)]));
    }
    keyboard_hits(&mut c, t, f, s, l.pick(texts::wlan::CONNECT), &mut out);
    if !a.choosing {
        return Frame { caret: Some(caret), ..Frame::new(t, c, out, l.pick(texts::wlan::ADD_NETWORK), top) };
    }
    // A touch anywhere off the list closes it.
    out.clear();
    out.push(Hit { rect: Rect::new(0.0, 0.0, m.w as f32, m.h as f32), action: Action::CloseChoices });
    let names = ADDABLE.map(|p| fmt::security(l, p));
    let chosen = ADDABLE.iter().position(|&p| p == a.security);
    let held = ADDABLE.iter().position(|&p| s.held == Some(Action::PickSecurity(p)));
    let list = widgets::choices(&mut c, t, f, security.rows[0], &names, chosen, held);
    out.extend(hits(&list, &ADDABLE.map(Action::PickSecurity)));
    Frame::new(t, c, out, l.pick(texts::wlan::ADD_NETWORK), top)
}

/// A network's details, in the stock page's grid: for the one chosen, how
/// joining it goes and what it gave, and forgetting it; for any other, how
/// strong and how protected it is.
pub fn wlan_network(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let network = s.details.as_ref().expect("the details page is of a network");
    let joined = s.wlan.as_ref().and_then(|w| w.joined.as_ref()).filter(|j| j.ssid == network.ssid);
    let y = widgets::section_first(&mut c, t, f, top, &network.name);
    let detail = |icon, title, value: String| Detail { icon, title, value };
    let security = detail(Outline::Shield, l.pick(texts::wlan::SECURITY_TITLE), fmt::security(l, network.security).to_string());
    let signal = l.pick(texts::wlan::SIGNAL_STRENGTH);
    let title = l.pick(texts::wlan::DETAILS);
    let Some(j) = joined else {
        let items = [detail(Outline::Wifi, l.pick(["频段", "頻段", "Bands"]), fmt::bands(l, network.bands)), detail(Outline::Signal, signal, fmt::signal(l, network.signal).to_string()), security];
        let y = widgets::detail_grid(&mut c, t, f, y, &items);
        return Frame::new(t, fit(c, t, y), out, title, top);
    };
    let mut items = vec![detail(Outline::Info, l.pick(texts::wlan::STATUS), fmt::joining(l, &j.stage).to_string())];
    match &j.stage {
        Joining::Connected(link) => {
            if let Some(n) = link.generation {
                items.push(detail(Outline::Wifi, l.pick(texts::wlan::STANDARD), fmt::generation(l, n)));
            }
            if let Some(speed) = link.speed {
                items.push(detail(Outline::Speed, l.pick(texts::wlan::LINK_SPEED), format!("{speed} Mbps")));
            }
            items.push(detail(Outline::Signal, signal, fmt::signal(l, link.signal).to_string()));
            items.push(security);
            items.push(detail(Outline::Address, l.pick(texts::wlan::IP_ADDRESS), link.address.to_string()));
            items.push(detail(Outline::Mask, l.pick(texts::wlan::SUBNET_MASK), link.mask.to_string()));
            if let Some(router) = link.router {
                items.push(detail(Outline::Router, l.pick(texts::wlan::ROUTER), router.to_string()));
            }
        }
        _ => {
            items.push(detail(Outline::Signal, signal, fmt::signal(l, network.signal).to_string()));
            items.push(security);
        }
    }
    let y = widgets::detail_grid(&mut c, t, f, y, &items);
    let forget = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(texts::wlan::FORGET)).tint(t.pal.danger).trail(Trail::None)]);
    out.extend(hits(&forget.rows, &[Action::LeaveNetwork]));
    Frame::new(t, fit(c, t, forget.bottom), out, title, top)
}

/// The file manager, every entry of the folder shown, folders first.
/// The files on the OTG drive, all of them: folders and files alike.
///
/// An entry picked in the file manager is put down here from the bar held at
/// the bottom, so a file on the phone is copied onto the drive.
pub fn files_otg(cx: &Ctx) -> Frame {
    let l = cx.state.lang;
    let top = l.pick(["OTG", "OTG", "OTG"]);
    let dock = cx.state.clip.as_ref().map(|clip| {
        let go = l.pick(match clip.op {
            FileOp::Copy => texts::files::COPY_HERE,
            FileOp::Move => texts::files::MOVE_HERE,
        });
        let (canvas, rects) = widgets::dock_bar(cx.t, cx.f, files::page(cx.t), &[l.pick(CANCEL), go]);
        Dock { canvas, hits: hits(&rects, &[Action::Unpick, Action::Paste]) }
    });
    let frame = listing(
        cx,
        &cx.state.otg,
        top,
        None,
        l.pick(["U 盘为空", "USB 隨身碟是空的", "The drive is empty"]),
        Action::OpenOtgEntry,
        top,
        Some((l.pick(["内部存储", "內部儲存空間", "Internal storage"]), Action::Open(Screen::Files))),
        // The card is not a step below the drive: the drive's own files are
        // what this page is for, and the way back is the pill beside the
        // trail.
        None,
    );
    Frame { dock, ..frame }
}

pub fn files(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    // The sort panel is another screen of this page rather than a page of its
    // own: going back closes it instead of leaving the file manager.
    if s.sort_panel {
        return sort_panel(cx);
    }
    // An entry picked to copy or move goes to the folder shown, from the
    // bar held at the bottom.
    let picking = s.picking.then(|| {
        let use_here = l.pick(["用这个文件夹", "用這個資料夾", "Serve this folder"]);
        let (canvas, rects) = widgets::dock_bar(t, f, files::page(t), &[use_here]);
        Dock { canvas, hits: hits(&rects, &[Action::ServeUseHere]) }
    });
    let dock = picking.or_else(|| {
        s.clip.as_ref().map(|clip| {
            let go = l.pick(match clip.op {
                FileOp::Copy => ["复制到此处", "複製到這裡", "Copy here"],
                FileOp::Move => ["移动到此处", "移動到這裡", "Move here"],
            });
            let (canvas, rects) = widgets::dock_bar(t, f, files::page(t), &[l.pick(CANCEL), go]);
            Dock { canvas, hits: hits(&rects, &[Action::Unpick, Action::Paste]) }
        })
    });
    // Choosing several entries: the bar at the foot carries what can be done
    // with them, and the page's own button starts and stops the choosing.
    let choosing = s.choosing.then(|| {
        let all = l.pick(["全选", "全選", "All"]);
        let copy = l.pick(["复制", "複製", "Copy"]);
        let cut = l.pick(["移动", "移動", "Move"]);
        let delete = l.pick(["删除", "刪除", "Delete"]);
        let (canvas, rects) = widgets::dock_bar_danger(t, f, files::page(t), &[all, copy, cut, delete], Some(3));
        Dock { canvas, hits: hits(&rects, &[Action::ChooseAll, Action::PasteChosen(FileOp::Copy), Action::PasteChosen(FileOp::Move), Action::AskDeleteChosen]) }
    });
    let dock = choosing.or(dock);
    let button = Some((widgets::Glyph::Terminal, Action::TerminalHere));
    let root = l.pick(["内部存储", "內部儲存空間", "Internal storage"]);
    let other = l.pick(["U 盘", "USB 隨身碟", "USB drive"]);
    // The card is one more step below the folder shown, not another place
    // beside it: a phone's files are all under /sdcard, and this is the way
    // back up to the top of it from anywhere below. It is not drawn while
    // that is where the page already is, since it would lead nowhere.
    let on_card = s.files.path.trim_end_matches('/') == "/sdcard";
    let card_place: Option<(&str, Action)> = if on_card {
        None
    } else {
        Some((l.pick(["sdcard", "sdcard", "sdcard"]), Action::FilesSdCard))
    };
    // A name is looked for in the folder shown: what it holds is filtered as
    // the name is typed, so the page itself answers the search.
    let shown = filter_folder(s);
    let entry_action: fn(usize) -> Action = if s.choosing { Action::ChooseEntryAt } else { Action::OpenEntry };
    let frame = listing(
        cx,
        &shown,
        root,
        None,
        l.pick(["暂无文件", "沒有檔案", "No files"]),
        entry_action,
        l.pick(FILES),
        Some((other, Action::FilesOtg)),
        card_place,
    );
    // The controls sit beside the title as four round buttons, all of them
    // icons, rather than as a row of words over the list: the list is what the
    // page is for, and the words took a row of it.
    let chips: [(Icon, Action, bool); 4] = [
        (Icon::Sort, Action::SortPanel(true), false),
        (Icon::Choose, Action::ChooseMany(!s.choosing), s.choosing),
        (Icon::UsbDrive, Action::FilesOtg, false),
        (Icon::Search, Action::SearchFiles, false),
    ];
    let (d, gap) = (t.m.dp(40.0), t.m.dp(8.0));
    let mut art = Canvas::new((d * 4.0 + gap * 3.0).ceil() as u32, d.ceil() as u32);
    let mut bar_hits = Vec::new();
    for (k, (icon, action, on)) in chips.iter().enumerate() {
        let x = k as f32 * (d + gap);
        // The fill is the button's own colour, which shows as a circle on the
        // light theme too; the card's colour would be the page's.
        let (fill, ink) = if *on { (t.pal.button_press, t.pal.danger) } else { (t.pal.search, t.pal.text) };
        art.fill_circle(x + d / 2.0, d / 2.0, d / 2.0, fill);
        crate::icons::glyph(&mut art, *icon, x + d / 2.0, d / 2.0, d * 0.56, ink);
        bar_hits.push(Hit { rect: Rect::new(x, 0.0, d, d), action: *action });
    }
    // Placed to the right of the title's own words, on the title's line.
    let title = l.pick(FILES);
    let zero = Canvas::new(0, 0);
    let px = zero.fitting(&f.title, title, t.m.title_px, t.m.w as f32 - t.m.title_x - t.m.margin);
    let tw = zero.text_width(&f.title, title, px);
    let ax = t.m.title_x + tw + t.m.dp(16.0);
    let ay = t.m.title_top - t.m.dp(4.0);
    Frame { dock, button, bar_hits, title_art: Some((art, ax, ay)), ..frame }
}

/// The folder the file manager shows, with only the entries whose names hold
/// what was typed where the page was opened from there to look for one.
/// Everything is kept where nothing was typed: the page is the folder
/// otherwise.
fn filter_folder(s: &crate::app::State) -> crate::app::Folder {
    let mut folder = s.files.clone();
    if s.search_scope != crate::app::SearchScope::Files {
        return folder;
    }
    let query = s.search.trim().to_lowercase();
    if query.is_empty() {
        return folder;
    }
    if let Ok(entries) = &mut folder.entries {
        entries.retain(|e| e.name.to_lowercase().contains(&query));
    }
    folder
}

/// What the file manager's list is sorted by, and which way, chosen from one
/// page of ticks rather than cycled through a button.
fn sort_panel(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let keys: [(&str, SortKey); 4] = [
        (l.pick(["名称", "名稱", "Name"]), SortKey::Name),
        (l.pick(["大小", "大小", "Size"]), SortKey::Size),
        (l.pick(["修改时间", "修改時間", "Modified"]), SortKey::Modified),
        (l.pick(["类型", "類型", "Type"]), SortKey::Type),
    ];
    let sides: [(&str, bool); 2] = [
        (l.pick(["升序 ↑", "升序 ↑", "Up ↑"]), false),
        (l.pick(["降序 ↓", "降序 ↓", "Down ↓"]), true),
    ];
    let mut rows: Vec<Row> = Vec::new();
    for (label, key) in keys {
        rows.push(Row::new(label).trail(Trail::Check(s.sort == key)));
    }
    for (label, desc) in sides {
        rows.push(Row::new(label).trail(Trail::Check(s.sort_desc == desc)));
    }
    let done = l.pick(texts::common::DONE);
    let top = m.dp(56.0);
    let h = top + m.dp(56.0) * rows.len() as f32 + m.dp(96.0);
    let mut c = Canvas::new(m.w, h.ceil() as u32);
    c.clear(files::page(t));
    let mut out = Vec::new();
    let block = widgets::list(&mut c, t, f, top, &rows);
    for (k, rect) in block.rows.iter().enumerate() {
        let action = match keys.get(k) {
            Some((_, key)) => Action::SortTo(*key),
            None => Action::SortDesc(sides[k - keys.len()].1),
        };
        out.push(Hit { rect: *rect, action });
    }
    let y = block.bottom + m.dp(20.0);
    let px = m.dp(17.0);
    let h_row = m.dp(44.0);
    let w = c.text_width(&f.value, done, px) + m.dp(56.0);
    let x = (m.w as f32 - w) / 2.0;
    widgets::pill_sized(&mut c, f, x, y, h_row, px, m.dp(20.0), done, t.pal.search, t.pal.text);
    out.push(Hit { rect: Rect::new(x, y, w, h_row), action: Action::SortPanel(false) });
    let bottom = y + h_row;
    Frame::new(t, fit(c, t, bottom), out, l.pick(["排序", "排序", "Sort"]), top)
}

/// A place's folders and the files in them that can be installed, to
/// choose one. A USB drive's page waits for one to be plugged in, and the
/// drive's files show as soon as it is.
pub fn choose(cx: &Ctx, place: Place) -> Frame {
    let (s, l) = (cx.state, cx.state.lang);
    let unreadable;
    let absent = match (place, &s.usb_drive) {
        (Place::UsbDrive, None) => Some((l.pick(texts::choose::NO_USB_DRIVE), l.pick(texts::choose::CONNECT_USB_DRIVE))),
        (Place::UsbDrive, Some(UsbDrive::Unreadable(why))) => {
            unreadable = texts::choose::usb_drive_format(l, why);
            Some((l.pick(texts::usb_drive::UNREADABLE), unreadable.as_str()))
        }
        _ => None,
    };
    let empty = l.pick(["暂无可安装的文件", "沒有可安裝的檔案", "Nothing here to install"]);
    listing(cx, s.folder(Screen::Choose(place)), place_name(l, place), absent, empty, Action::ChooseEntry, l.pick(["选择文件", "選擇檔案", "Choose a file"]), None, None)
}

/// A folder laid out as the stock file manager lays one out: the path as
/// capsules from `top`, held under the bar once scrolled there, then the
/// entries in one list on the plain page, each row drawn as it scrolls into
/// view and touched for `action`. In place of the list, `absent` says why
/// nothing of the folder can be shown, and `empty` that it holds nothing.
// A folder shown in one place is described by where it is, what stands in
// for it when it cannot be read, and what its rows do: every part is used,
// and naming them in a struct would only move the list.
#[allow(clippy::too_many_arguments)]
fn listing(
    cx: &Ctx,
    folder: &Folder,
    top: &str,
    absent: Option<(&str, &str)>,
    empty: &str,
    action: fn(usize) -> Action,
    title: &'static str,
    // The other place the files can be shown from, beside the trail.
    places_other: Option<(&str, Action)>,
    // One more place to go, pinned to the end of the trail: the card, which
    // reads as one more step below the folder shown rather than as another
    // strip beside it.
    card_place: Option<(&str, Action)>,
) -> Frame {
    let (t, f) = (cx.t, cx.f);
    let bg = files::page(t);
    let mut c = sheet(cx, t.m.h, bg);
    let rows = match (absent, &folder.entries) {
        (Some((note, why)), _) => {
            files::note(&mut c, t, f, note, Some(why));
            None
        }
        (None, Err(why)) => {
            files::note(&mut c, t, f, cx.state.lang.pick(texts::files::UNREADABLE), Some(why));
            None
        }
        (None, Ok(entries)) if entries.is_empty() => {
            files::note(&mut c, t, f, empty, None);
            None
        }
        (None, Ok(entries)) => Some(files::rows(t, f, entries, action)),
    };
    let pinned = Some(files::strip_with(t, f, top, &folder.path, places_other, card_place));    Frame { bg, rows, pinned, ..Frame::new(t, c, Vec::new(), title, t.m.dp(files::LIST_TOP_DP)) }
}

/// Row `i` of the page's long list over the whole of `c`, sized for it,
/// drawn as the row comes into view.
pub fn row(screen: Screen, cx: &Ctx, i: usize, c: &mut Canvas) {
    match screen {
        Screen::Files | Screen::FilesOtg | Screen::Choose(_) => {
            let folder = cx.state.folder(screen);
            let entries = folder.entries.as_ref().expect("a folder listed draws its rows");
            let chosen = cx.state.choosing && cx.state.chosen.contains(&entries[i].name);
            files::row_marked(c, cx.t, cx.f, cx.state.lang, &entries[i], folder.listed, cx.state.zone(), chosen);
        }
        Screen::Installing => widgets::output_row(c, cx.t, cx.f, &cx.state.script_output, i),
        Screen::PartWork => {
            let work = cx.state.part_work.as_ref().expect("the page of partition work shows work");
            widgets::output_row(c, cx.t, cx.f, &work.output, i)
        }
        other => unreachable!("{other:?} draws no list row by row"),
    }
}

/// One entry of a folder, a file or a folder: what it is, and what can be
/// done with it. Deleting asks first, with a swipe.
pub fn file_info(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let e = s.file.as_ref().expect("the file page shows an entry");
    let folder = matches!(e.kind, FileKind::Folder { .. });
    let mut c = long_page(cx);
    let m = &t.m;
    let (w, h) = (m.w as f32, m.h as f32);
    let side = m.margin;
    let top = m.content_top;
    let (card, name_ink, note_ink, pill_bg, pill_ink) = if t.dark {
        (Rgba::rgb(28, 28, 30), Rgba::rgb(242, 242, 242), Rgba::rgb(128, 128, 128), Rgba::rgb(38, 38, 38), Rgba::rgb(214, 216, 222))
    } else {
        (Rgba::rgb(246, 247, 250), Rgba::rgb(0, 0, 0), Rgba::rgb(102, 102, 102), Rgba::rgb(238, 240, 245), Rgba::rgb(40, 44, 52))
    };

    // 这一个文件能做些什么。能装能改的排在最前面，删除永远在最后一格、红字。
    let mut pills: Vec<(String, Action, bool)> = Vec::new();
    if installs(e) {
        let image = Path::new(&e.name).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("img"));
        pills.push((l.pick(if image { ["刷入", "刷入", "Flash"] } else { ["安装", "安裝", "Install"] }).to_string(), Action::InstallEntry, false));
    }
    if e.kind == FileKind::File {
        pills.push((l.pick(["编辑", "編輯", "Edit"]).to_string(), Action::EditEntry, false));
    }
    // 四排：复制/移动、复制到 U 盘/移动到 U 盘、重命名/文件权限、删除独占一行
    //（2026-11-17）。移动就是剪切，同一边挑目录；到另一边是一步直达。
    pills.push((l.pick(["复制", "複製", "Copy"]).to_string(), Action::PickEntry(FileOp::Copy), false));
    pills.push((l.pick(["移动", "移動", "Move"]).to_string(), Action::PickEntry(FileOp::Move), false));
    match s.file_on_drive {
        true => {
            pills.push((l.pick(["复制到手机", "複製到手機", "Copy to phone"]).to_string(), Action::ToPhone(FileOp::Copy), false));
            pills.push((l.pick(["移动到手机", "移動到手機", "Move to phone"]).to_string(), Action::ToPhone(FileOp::Move), false));
        }
        false => {
            pills.push((l.pick(["复制到 U 盘", "複製到 USB", "Copy to drive"]).to_string(), Action::ToDrive(FileOp::Copy), false));
            pills.push((l.pick(["移动到 U 盘", "移動到 USB", "Move to drive"]).to_string(), Action::ToDrive(FileOp::Move), false));
        }
    }
    // 一个 zip 还能当模块装：KSU / Magisk 的模块就是这个形状。
    if e.kind == FileKind::File && e.name.to_lowercase().ends_with(".zip") {
        pills.push((l.pick(["作为模块安装", "當作模組安裝", "Install as module"]).to_string(), Action::InstallAsModule, false));
    }
    pills.push((l.pick(RENAME).to_string(), Action::RenameEntry, false));
    if !matches!(e.kind, FileKind::Link { .. }) {
        pills.push((l.pick(["文件权限", "檔案權限", "Permissions"]).to_string(), Action::OpenPermissions, false));
    }
    pills.push((l.pick(["删除", "刪除", "Delete"]).to_string(), Action::AskDelete, true));

    // 等宽网格：三列（选项多的）或两列，格子一样宽、左右都对齐。删除单独
    // 一行、占满整行——它和别的不是一类事，挤在角落里反而像没排完。
    let (danger, others) = match pills.split_last() {
        Some((last, rest)) if last.2 => (Some(last.clone()), rest.to_vec()),
        _ => (None, pills.clone()),
    };
    // 列数按总格数挑，保证最多两行——三行排下来像没排完（2026-11-17）。
    // 删除也算一格，所以是「总格数的一半向上取整」。
    // 第一排四个，剩下的排第二排并**把整排均分**：第二排三个就三个各占
    // 三分之一，不留空档（2026-11-17）。
    // 每排两个、均匀铺满；删除单独占最后一整行（2026-11-17）。
    let all: Vec<(String, Action, bool)> = others.clone();
    let mut rows: Vec<(usize, usize)> = all.len().checked_add(1).map(|_| (0..all.len()).step_by(2).map(|k| (k, (k + 2).min(all.len()))).collect::<Vec<_>>()).unwrap_or_default();
    if danger.is_some() {
        rows.push((all.len(), all.len()));
    }
    let row_count = rows.len();
    let gap = m.dp(widgets::PILL_GAP_DP);
    let ph = m.dp(widgets::PILL_DP);
    let room = w - side * 2.0;
    let px = m.dp(widgets::PILL_TEXT_DP);
    let grid_px = m.dp(13.0);
    let grid_pad = m.dp(7.0);
    let inner = m.dp(18.0);
    let card_h = inner * 2.0 + m.dp(42.0);
    let title_px = m.dp(13.0);
    let inner_h = card_h;
    let sheet_h = m.dp(30.0) + row_count as f32 * ph + (row_count.saturating_sub(1)) as f32 * gap + m.dp(18.0);
    // 整块（信息卡 + 操作格）在页面里居中：全压在底部会让上半屏空得像没加载完。
    let block_h = inner_h + m.dp(12.0) + sheet_h;
    let block_top = ((h - block_h) / 2.0).max(top);
    let sheet_top = block_top + inner_h + m.dp(12.0);
    let card_top = block_top;

    // 上面这张卡只说一件事：正在操作哪一个。名字、大小或项数、改于何时。
    // 卡片按内容高，不抻到底：空着一大片会把视线拽走。
    // 整块贴着底部：卡片紧挨在操作格上面，像从下面升起来的一段
    //（空着的那一截留在页面标题之下，不把视线拉成两段）。
    c.fill_round_rect(side, card_top, room, card_h, m.card_radius, card);
    let cx = side + inner;
    let avatar = m.dp(42.0);
    let ay = card_top + inner;
    match e.kind {
        FileKind::Folder { .. } => {
            icons::glyph(&mut c, Icon::Folder, cx + avatar / 2.0, ay + avatar / 2.0 - m.dp(4.0), avatar * 0.8, Rgba::rgb(246, 196, 68));
        }
        _ => {
            let half = avatar * 0.78;
            c.fill_round_rect(cx + (avatar - half) / 2.0, ay + (avatar - half) / 2.0, half, half, m.dp(6.0), Rgba::rgb(120, 168, 255));
            let ext = Path::new(&e.name).extension().map(|x| x.to_string_lossy().to_uppercase()).unwrap_or_default();
            let ext: String = ext.chars().take(4).collect();
            if !ext.is_empty() {
                let pxl = m.dp(10.0);
                let tw = c.text_width(&f.value, &ext, pxl);
                c.text(&f.value, &ext, cx + avatar / 2.0 - tw / 2.0, ay + avatar / 2.0 + pxl * 0.36, pxl, Rgba::rgb(255, 255, 255), None);
            }
        }
    }
    let text_x = cx + avatar + m.dp(16.0);
    let text_room = side + room - inner - text_x;
    let title = cut_to(&c, &f.value, &e.name, m.dp(17.0), text_room);
    c.text(&f.value, &title, text_x, ay + m.dp(18.0), m.dp(17.0), name_ink, None);
    let holds = match &e.kind {
        FileKind::Folder { items: Some(n) } => match (l, *n) {
            (Lang::En, 1) => "1 item".to_string(),
            (_, n) => tr!(l, "{n} 项", "{n} 項", "{n} items"),
        },
        FileKind::Folder { items: None } => l.pick(UNREADABLE).to_string(),
        FileKind::Link { .. } => l.pick(["链接", "連結", "A link"]).to_string(),
        _ => fmt::file_size(e.size),
    };
    let when = fmt::datetime(l, e.modified, s.zone());
    c.text(&f.value, &format!("{holds} · {when}"), text_x, ay + m.dp(38.0), m.dp(13.0), note_ink, None);

    // 下面：一句标题，再一层等宽的药丸。
    c.text(&f.value, l.pick(["文件的操作", "檔案的操作", "What can be done"]), side, sheet_top + m.dp(20.0), title_px, note_ink, None);
    let mut rects: Vec<Rect> = Vec::new();
    let mut acts: Vec<Action> = Vec::new();
    for (r, (from, to)) in rows.iter().enumerate() {
        let n = to - from;
        let y = sheet_top + m.dp(30.0) + r as f32 * (ph + gap);
        // 删除那一排：一格占满整行。
        if n == 0 {
            if let Some((label, action, _)) = &danger {
                c.fill_round_rect(side, y, room, ph, ph / 2.0, pill_bg);
                c.text_center(&f.value, label, side + room / 2.0, y + ph / 2.0, px, t.pal.danger);
                rects.push(Rect::new(side, y, room, ph));
                acts.push(*action);
            }
            continue;
        }
        let cell = (room - gap * (n as f32 - 1.0)) / n as f32;
        for (i, k) in (*from..*to).enumerate() {
            let (label, action, bad) = &all[k];
            let x = side + i as f32 * (cell + gap);
            c.fill_round_rect(x, y, cell, ph, ph / 2.0, pill_bg);
            let shown = cut_to(&c, &f.value, label, grid_px, cell - grid_pad * 2.0);
            c.text_center(&f.value, &shown, x + cell / 2.0, y + ph / 2.0, grid_px, if *bad { t.pal.danger } else { pill_ink });
            rects.push(Rect::new(x, y, cell, ph));
            acts.push(*action);
        }
    }
    let page_title = l.pick(if folder { ["文件夹信息", "資料夾資訊", "Folder info"] } else { ["文件信息", "檔案資訊", "File info"] });
    let bottom = sheet_top + sheet_h + m.dp(8.0);
    Frame::new(t, fit(c, t, bottom), hits(&rects, &acts), page_title, top)
}

/// `text` cut to `w` pixels, its middle taken out.
fn cut_to(c: &Canvas, face: &crate::font::Face, text: &str, px: f32, w: f32) -> String {
    let width = |t: &str| c.text_width(face, t, px);
    if width(text) <= w {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let room = w - width("…");
    let mut right = chars.len();
    while right > 0 && width(&chars[right - 1..].iter().collect::<String>()) <= room / 2.0 {
        right -= 1;
    }
    let left_room = room - width(&chars[right..].iter().collect::<String>());
    let mut left = 0;
    while left < right && width(&chars[..=left].iter().collect::<String>()) <= left_room {
        left += 1;
    }
    format!("{}…{}", chars[..left].iter().collect::<String>(), chars[right..].iter().collect::<String>())
}

/// The permission bits of the entry the file page shows, a switch each that
/// takes effect at once: reading, writing and running for its owner, its
/// group and everyone else, then the set-ID and sticky bits.
pub fn permissions(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let e = s.file.as_ref().expect("the permissions page shows an entry");
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mode = e.mode & 0o7777;
    let bits = fmt::mode(e.mode);
    use texts::files::mode as said;
    let facts = [(l.pick(texts::files::NAME), e.name.clone()), (l.pick(texts::files::PERMISSIONS), said::bits(l, bits, format!("{mode:04o}")))];
    let mut y = widgets::info_card(&mut c, t, f, top, &facts);
    let mut out = Vec::new();
    let classes = [(said::OWNER, 6), (said::GROUP, 3), (said::OTHERS, 0)];
    let (read, write, run) = (l.pick(said::READ), l.pick(said::WRITE), l.pick(said::EXECUTE));
    let groups = classes.map(|(title, shift)| (l.pick(title), [(read, 4 << shift), (write, 2 << shift), (run, 1 << shift)]));
    let special = (l.pick(said::SPECIAL), [("SUID", 0o4000), ("SGID", 0o2000), (l.pick(said::STICKY), 0o1000)]);
    for (title, bits) in groups.into_iter().chain([special]) {
        y = widgets::section(&mut c, t, f, y, title);
        let rows = bits.map(|(name, bit)| Row::new(name).trail(Trail::Switch(mode & bit != 0)));
        let list = widgets::list(&mut c, t, f, y, &rows);
        out.extend(hits(&list.rows, &bits.map(|(_, bit)| Action::ModeBit(bit))));
        y = list.bottom;
    }
    Frame::new(t, fit(c, t, y), out, l.pick(texts::files::CHANGE_PERMISSIONS), top)
}

/// A new name for the entry the file page shows, typed on the keyboard.
pub fn rename(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let (y, caret) = widgets::text_field(&mut c, t, f, top, &s.new_name);
    widgets::footnote(&mut c, t, f, y, l.pick(texts::files::RENAME_NOTE));
    keyboard_hits(&mut c, t, f, s, l.pick(texts::common::DONE), &mut out);
    Frame { caret: Some(caret), ..Frame::new(t, c, out, l.pick(texts::files::RENAME), top) }
}

/// The folder an entry goes to already holds its name: what to do.
pub fn clash(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let (clip, into, _) = s.clash.as_ref().expect("the clash page shows a clash");
    let name = Path::new(&clip.path).file_name().map_or_else(|| clip.path.clone(), |n| n.to_string_lossy().into_owned());
    let verb = l.pick(match clip.op {
        FileOp::Copy => ["复制", "複製", "copy"],
        FileOp::Move => ["移动", "移動", "move"],
    });
    let mut c = page(cx);
    let top = t.m.content_top;
    let there = tr!(l, "\"{into}\"中已有同名的\"{name}\"。", "「{into}」中已有同名的「{name}」。", "“{into}” already holds an entry named “{name}”.");
    let y = widgets::paragraph(&mut c, t, f, top, &there);
    let skip = tr!(l, "保留已有的同名项，不{verb}该项", "保留已有的同名項目，不{verb}這個項目", "Keeps the entry there, and doesn't {verb} this one");
    let choices = widgets::list(
        &mut c,
        t,
        f,
        y + t.m.dp(12.0),
        &[
            Row::new(l.pick(["替换", "取代", "Replace"])).subtitle(l.pick(["覆盖已有的同名项", "覆寫已有的同名項目", "Writes over the entry there"])),
            Row::new(l.pick(["跳过", "略過", "Skip"])).subtitle(&skip),
            Row::new(l.pick(["保留两者", "保留兩者", "Keep both"])).subtitle(l.pick(["自动为该项重新命名", "自動為這個項目重新命名", "Gives this one a new name"])),
        ],
    );
    let out = hits(&choices.rows, &[Action::Resolve(ClashChoice::Replace), Action::Resolve(ClashChoice::Skip), Action::Resolve(ClashChoice::KeepBoth)]);
    Frame::new(t, c, out, l.pick(["名称已存在", "名稱已存在", "Name taken"]), top)
}

/// Copying, moving or deleting: how far it has come and a way to stop it,
/// then, if it did not finish, why.
pub fn file_job(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let job = s.job.as_ref().expect("the job page shows work on files");
    let path = match &job.task {
        FileTask::Copy { path, .. } | FileTask::Move { path, .. } | FileTask::Delete { path } => path,
    };
    use texts::files::job as said;
    let name = Path::new(path).file_name().map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());
    let share = |(done, total): (u64, u64)| {
        if total == 0 { 0.0 } else { done as f32 / total as f32 }
    };
    // The bar stays while the work measures, so that the button under it
    // does not move. Deleting, and copying entries that hold no bytes, is
    // counted in entries.
    let (detail, done) = match (job.progress, &job.task) {
        (None, _) => (l.pick(texts::installing::GETTING_READY).to_string(), 0.0),
        (Some((bytes, _)), FileTask::Copy { .. }) if bytes.1 > 0 => (said::copied_of(l, fmt::file_size(bytes.0), fmt::file_size(bytes.1)), share(bytes)),
        (Some((bytes, _)), FileTask::Move { .. }) if bytes.1 > 0 => (said::moved_of(l, fmt::file_size(bytes.0), fmt::file_size(bytes.1)), share(bytes)),
        (Some((_, entries)), FileTask::Copy { .. }) => (said::copied_items(l, entries.0, entries.1), share(entries)),
        (Some((_, entries)), FileTask::Move { .. }) => (said::moved_items(l, entries.0, entries.1), share(entries)),
        (Some((_, entries)), FileTask::Delete { .. }) => (said::deleted_items(l, entries.0, entries.1), share(entries)),
    };
    let mut c = page(cx);
    let top = t.m.content_top;
    let y = widgets::progress_card(&mut c, t, f, top, &name, &detail, Some(done));
    let mut out = Vec::new();
    match &job.outcome {
        None => {
            let label = l.pick(if job.stopping { texts::common::STOPPING } else { texts::common::STOP });
            let stop = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(label).trail(Trail::None)]);
            if !job.stopping {
                out.extend(hits(&stop.rows, &[Action::StopJob]));
            }
        }
        Some(end) => {
            let why = match (end, &job.task) {
                (JobEnd::Stopped, FileTask::Copy { .. }) => l.pick(said::COPY_STOPPED).to_string(),
                (JobEnd::Stopped, FileTask::Move { .. }) => l.pick(said::MOVE_STOPPED).to_string(),
                (JobEnd::Stopped, FileTask::Delete { .. }) => l.pick(said::DELETE_STOPPED).to_string(),
                (JobEnd::Failed(why), FileTask::Copy { .. }) => said::copy_failed(l, why),
                (JobEnd::Failed(why), FileTask::Move { .. }) => said::move_failed(l, why),
                (JobEnd::Failed(why), FileTask::Delete { .. }) => said::delete_failed(l, why),
                (JobEnd::Done, _) => unreachable!("work done leaves its page"),
            };
            let y = widgets::footnote(&mut c, t, f, y, &why);
            let finish = widgets::list(&mut c, t, f, y + t.m.card_gap, &[Row::new(l.pick(texts::common::DONE)).trail(Trail::None)]);
            out.extend(hits(&finish.rows, &[Action::Back]));
        }
    }
    let title = l.pick(match (&job.task, job.outcome.is_none()) {
        (FileTask::Copy { .. }, true) => said::COPYING,
        (FileTask::Copy { .. }, false) => said::COPY_INCOMPLETE,
        (FileTask::Move { .. }, true) => said::MOVING,
        (FileTask::Move { .. }, false) => said::MOVE_INCOMPLETE,
        (FileTask::Delete { .. }, true) => said::DELETING,
        (FileTask::Delete { .. }, false) => said::DELETE_INCOMPLETE,
    });
    Frame { back: job.outcome.is_some(), ..Frame::new(t, c, out, title, top) }
}

/// A shell on a terminal: the screen it writes, the keys a phone keyboard
/// lacks, and the keyboard, every key of which goes to the shell.
pub fn terminal(cx: &Ctx) -> Frame {
    let (t, f, s) = (cx.t, cx.f, cx.state);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    keyboard_hits(&mut c, t, f, s, s.lang.pick(texts::keyboard::ENTER), &mut out);
    let l = term::layout(t, f, top);
    term::draw(&mut c, t, f, &l, cx.term);
    let held = cx.term.is_some_and(|term| term.ctrl);
    let keys = term::keys(&mut c, t, f, &l, &term::TERM_KEYS, held);
    out.extend(keys.into_iter().map(|(rect, key)| Hit { rect, action: Action::TermKey(key) }));
    Frame { scrollback: Some((l.card, l.pitch)), ..Frame::new(t, c, out, s.lang.pick(texts::page::TERMINAL), top) }
}

/// A text file being edited: where it is, its lines wrapped on a card, the
/// keys a phone keyboard lacks for moving the cursor, and the keyboard.
/// Once it is changed, the round button saves it.
pub fn editor(cx: &Ctx) -> Frame {
    let (t, f, s) = (cx.t, cx.f, cx.state);
    let ed = s.editor.as_ref().expect("the editor page shows a file");
    let mut c = page(cx);
    let mut out = Vec::new();
    keyboard_hits(&mut c, t, f, s, s.lang.pick(texts::keyboard::NEW_LINE), &mut out);
    editor::caption(&mut c, t, f, &ed.path);
    let l = editor::layout(t, f);
    let caret = editor::draw(&mut c, t, f, &l, ed);
    let keys = term::keys(&mut c, t, f, &l, &editor::EDITOR_KEYS, false);
    out.extend(keys.into_iter().map(|(rect, key)| Hit { rect, action: Action::TermKey(key) }));
    let button = ed.changed.then_some((widgets::Glyph::Done, Action::SaveText));
    Frame { scrollback: Some((l.card, l.pitch)), caret, button, ..Frame::new(t, c, out, s.lang.pick(texts::files::EDIT), t.m.content_top) }
}

/// The languages the pages speak, each named in itself, to speak another.
/// The recording: whether a take is running, how the next one is recorded,
/// and the way to start or stop one.
///
/// The page is a little longer than a screen: the note at its foot used to be
/// cut in half by the screen's edge, so it is measured by what it holds and
/// scrolls to the end.
pub fn record(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["这一段", "這一段", "This take"]));
    let state = match (s.record.on, s.record.paused) {
        (true, true) => l.pick(["已暂停", "已暫停", "Paused"]),
        (true, false) => l.pick(["正在录制", "正在錄影", "Recording"]),
        _ => l.pick(["没有在录", "沒有在錄", "Not recording"]),
    };
    let length = if s.record.on { s.record.elapsed.clone() } else { "—".to_string() };
    let file = if s.record.file.is_empty() {
        l.pick(["还没有录过", "還沒有錄過", "None yet"]).to_string()
    } else {
        s.record.file.clone()
    };
    let spec = if s.record.spec.is_empty() { "—".to_string() } else { s.record.spec.clone() };
    let rows = [
        (l.pick(["状态", "狀態", "State"]), state.to_string()),
        (l.pick(["已录", "已錄", "Length"]), length),
        (l.pick(["文件", "檔案", "File"]), file),
        (l.pick(["规格", "規格", "Format"]), spec),
    ];
    let y = widgets::info_card(&mut c, t, f, y, &rows);

    // How it records: each of the four changes a notch on a tap. Nothing
    // changes while a take runs -- a different size would begin the encoders
    // and the file's header again.
    let y = widgets::section(&mut c, t, f, y, l.pick(["录成什么样", "錄成什麼樣", "How it records"]));
    let panel = (t.m.w, t.m.h);
    let scale_names = [
        l.pick(["原画", "原畫", "Full"]),
        l.pick(["高清", "高清", "Sharp"]),
        l.pick(["标准", "標準", "Std"]),
        l.pick(["流畅", "流暢", "Smooth"]),
    ];
    let quality_names = [l.pick(["高", "高", "High"]), l.pick(["中", "中", "Mid"]), l.pick(["低", "低", "Low"])];
    let sizes: Vec<String> = (0..crate::record::SCALES.len())
        .map(|k| {
            let z = crate::record::Settings::of(panel, k, s.record.quality, s.record.fps);
            format!("{}×{}", z.width, z.height)
        })
        .collect();
    let chosen = crate::record::Settings::of(panel, s.record.scale, s.record.quality, s.record.fps);
    let per_minute = chosen.per_frame() as f64 * chosen.fps as f64 * 60.0 / 1e6;
    let values = [
        format!("{} {}", scale_names[s.record.scale.min(3)], sizes[s.record.scale.min(3)]),
        quality_names[s.record.quality.min(2)].to_string(),
        format!("{:.0} fps", crate::record::RATES[s.record.fps.min(3)]),
    ];
    let sized = format!("{}×{} · 每分钟约 {per_minute:.0} MB", chosen.width, chosen.height);
    let locked = l.pick(["正在录制，停下这一段才能改", "正在錄影，停下這一段才能改", "Recording: stop this take to change it"]);
    let rows = [
        Row::new(l.pick(["分辨率", "解析度", "Size"]))
            .icon(BLUE, Icon::Record)
            .subtitle(if s.record.on { locked } else { sized.as_str() })
            .trail(Trail::Value(values[0].as_str())),
        Row::new(l.pick(["画质", "畫質", "Quality"]))
            .icon(ORANGE, Icon::Record)
            .subtitle(if s.record.on {
                locked
            } else {
                l.pick(["画质越高越清楚，文件也越大", "畫質越高越清楚，檔案也越大", "Higher quality is sharper and bigger"])
            })
            .trail(Trail::Value(values[1].as_str())),
        Row::new(l.pick(["帧率", "幀率", "Frame rate"]))
            .icon(GREEN, Icon::Record)
            .subtitle(if s.record.on {
                locked
            } else {
                l.pick([
                    "60 帧最顺，15 帧最省；太高会补帧",
                    "60 幀最順，15 幀最省；太高會補幀",
                    "60 is smoothest, 15 the lightest; too high and frames repeat",
                ])
            })
            .trail(Trail::Value(values[2].as_str())),
        Row::new(l.pick(["容器", "容器", "Container"]))
            .icon(SLATE, Icon::Record)
            .subtitle(if s.record.on {
                locked
            } else {
                l.pick([
                    "AVI 到处都能放；MP4 相册更认，但断电那份打不开",
                    "AVI 到處都能放；MP4 相簿更認，但斷電那份打不開",
                    "AVI plays anywhere; MP4 suits galleries, but a cut file has no index",
                ])
            })
            .trail(Trail::Value(if s.record.format == 0 { "AVI" } else { "MP4" })),
    ];
    let list = widgets::list(&mut c, t, f, y, &rows);
    let mut out = hits(
        &list.rows,
        &[
            Action::RecordSetting(RecordSetting::Scale),
            Action::RecordSetting(RecordSetting::Quality),
            Action::RecordSetting(RecordSetting::Fps),
            Action::RecordSetting(RecordSetting::Format),
        ],
    );

    let doing = if s.record.on {
        l.pick(["停止录制", "停止錄影", "Stop recording"])
    } else {
        l.pick(["开始录制", "開始錄影", "Start recording"])
    };
    let rows = [Row::new(doing).icon(RED, Icon::Record).trail(Trail::None)];
    let list = widgets::list(&mut c, t, f, list.bottom + t.m.dp(10.0), &rows);
    out.extend(hits(&list.rows, &[Action::Record]));
    let bottom = widgets::footnote(
        &mut c,
        t,
        f,
        list.bottom,
        l.pick([
            "录好的文件在内部存储的 DCIM/Screenrecords，边录边写：断电最多丢最后一秒。录的时候那枚胶囊可以拖到任意位置：点左边的红点是暂停或继续，点右边的方块是结束这一段。",
            "錄好的檔案在內部儲存的 DCIM/Screenrecords，邊錄邊寫：斷電最多丟最後一秒。錄的時候那枚膠囊可以拖到任意位置：點左邊的紅點是暫停或繼續，點右邊的方塊是結束這一段。",
            "Takes go to DCIM/Screenrecords on internal storage and are written as they are recorded: a power cut loses the last second at most. Drag the pill anywhere while it runs; tap its dot to pause or carry on, tap its square to end the take.",
        ]),
    );
    Frame::new(t, fit(c, t, bottom), out, l.pick(["录屏", "錄影", "Screen recording"]), top)
}

pub fn language(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let rows: Vec<Row> = Lang::ALL.iter().map(|&k| Row::new(k.name()).trail(Trail::Check(k == l))).collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let out = hits(&list.rows, &Lang::ALL.map(Action::Language));
    Frame::new(t, fit(c, t, list.bottom), out, l.pick(texts::page::LANGUAGE), top)
}

/// The time zone the status bar's clock and the times on the pages keep.
/// On a phone whose installed system's zone is known, a switch follows it
/// and tells which it is; a phone with no system, or one that has set no
/// zone, shows none. Under it a zone is chosen from a part of the world,
/// whose row names the zone kept while that is one of its own.
pub fn time_zone(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = match &s.system_zone {
        Some(zone) => {
            let name = listed(&zone.name).map_or(zone.name.as_str(), |at| l.pick(TIME_ZONES[at].name));
            let system = format!("{} · {name}", fmt::utc_offset(s.now, &zone.rules));
            let follow = Row::new(l.pick(texts::time_zone::FOLLOW_SYSTEM)).subtitle(&system).trail(Trail::Switch(s.follow_system_zone));
            let follow = widgets::list(&mut c, t, f, top, &[follow]);
            out.extend(hits(&follow.rows, &[Action::Toggle(Switch::SystemTimeZone)]));
            widgets::section(&mut c, t, f, follow.bottom, l.pick(texts::time_zone::CHOOSE))
        }
        None => top,
    };
    let kept = s.zone_listed();
    let rows = Region::ALL.map(|region| {
        let row = Row::new(l.pick(region.name()));
        match kept.filter(|&at| TIME_ZONES[at].region == Some(region)) {
            Some(at) => row.trail(Trail::Value(l.pick(TIME_ZONES[at].name))),
            None => row,
        }
    });
    let regions = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&regions.rows, &Region::ALL.map(|region| Action::Open(Screen::TimeZones(region)))));
    let utc = listed(UTC).expect("UTC is a zone listed");
    let offset = fmt::utc_offset(s.now, fmt::rules(utc));
    let universal = Row::new(l.pick(TIME_ZONES[utc].name)).subtitle(&offset).trail(Trail::Check(kept == Some(utc)));
    let universal = widgets::list(&mut c, t, f, regions.bottom + t.m.card_gap, &[universal]);
    out.extend(hits(&universal.rows, &[Action::TimeZone(utc)]));
    Frame::new(t, fit(c, t, universal.bottom), out, l.pick(TIME_ZONE), top)
}

/// The zones of one part of the world, as its row on the time zone page opens
/// them.
pub fn time_zones(cx: &Ctx, region: Region) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let mut zones: Vec<usize> = (0..TIME_ZONES.len()).filter(|&at| TIME_ZONES[at].region == Some(region)).collect();
    zones.sort_by_key(|&at| fmt::utc_offset_seconds(s.now, fmt::rules(at)));
    let lines: Vec<String> = zones.iter().map(|&at| format!("{} · {}", fmt::utc_offset(s.now, fmt::rules(at)), l.pick(TIME_ZONES[at].cities))).collect();
    let kept = s.zone_listed();
    let rows: Vec<Row> = zones.iter().zip(&lines).map(|(&at, line)| Row::new(l.pick(TIME_ZONES[at].name)).subtitle(line).trail(Trail::Check(kept == Some(at)))).collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = zones.iter().map(|&at| Action::TimeZone(at)).collect();
    Frame::new(t, fit(c, t, list.bottom), hits(&list.rows, &picks), l.pick(region.name()), top)
}

/// A phrase that opens a line on its own: in English with its first letter
/// a capital.
fn capitalized(l: Lang, phrase: &str) -> String {
    let mut chars = phrase.chars();
    match (l, chars.next()) {
        (Lang::En, Some(first)) => first.to_uppercase().chain(chars).collect(),
        _ => phrase.to_string(),
    }
}

/// Saving the logs, as a bug report asks for them.
pub fn logs(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["日志内容", "記錄內容", "Logs"]));
    let parts = widgets::list(
        &mut c,
        t,
        f,
        y,
        &[
            Row::new(l.pick(["Recovery 日志", "Recovery 記錄", "Recovery log"]))
                .subtitle(l.pick(["界面、安装和挂载的记录", "介面、安裝和掛載的記錄", "What the pages, installs and mounts did"]))
                .trail(Trail::Check(s.logs[0])),
            Row::new(l.pick(["内核日志", "核心記錄", "Kernel log"]))
                .subtitle(l.pick(["本次开机以来的内核消息", "本次開機以來的核心訊息", "The kernel's messages since this start"]))
                .trail(Trail::Check(s.logs[1])),
        ],
    );
    out.extend(hits(&parts.rows, &[Action::LogPart(0), Action::LogPart(1)]));
    let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(texts::logs::SAVE_TO));
    let storage = l.pick(match cx.state.data.storage_open() {
        true => texts::logs::FOLDER,
        false => texts::decrypt::NEEDED,
    });
    let places = widgets::list(
        &mut c,
        t,
        f,
        y,
        &[
            Row::new(l.pick(["临时文件夹", "暫存資料夾", "Temporary folder"]))
                .subtitle(l.pick(["/skyward/run/logs，可通过 adb pull 导出到电脑", "/skyward/run/logs，可透過 adb pull 匯出到電腦", "/skyward/run/logs, which adb pull copies to a computer"]))
                .trail(Trail::Check(s.logs_to == 0)),
            Row::new(place_name(l, Place::Storage)).subtitle(storage).trail(Trail::Check(s.logs_to == 1)),
        ],
    );
    out.extend(hits(&places.rows, &[Action::LogsTo(0), Action::LogsTo(1)]));
    let y = widgets::footnote(&mut c, t, f, places.bottom, l.pick(texts::logs::NOTE));
    let export = widgets::button(&mut c, t, f, y + t.m.dp(24.0), l.pick(texts::logs::EXPORT));
    out.push(Hit { rect: export, action: Action::ExportLogs });
    Frame::new(t, c, out, l.pick(texts::page::LOGS), top)
}

/// Reporting a bug: what is written about it, what goes with it, and where
/// sending it stands.
pub fn feedback(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let m = &t.m;
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = m.content_top;
    let fb = &s.feedback;
    let title = l.pick(["问题反馈", "問題回報", "Report a bug"]);
    // A report was taken: the number it was given is the whole of what the
    // page has to say, since nothing more may be sent this start.
    if let Report::Sent { id } = &fb.report {
        let y = widgets::section_first(&mut c, t, f, top, l.pick(["本次已反馈", "本次已回報", "Sent this start"]));
        let done = widgets::info_card(&mut c, t, f, y, &[(l.pick(["问题 ID", "問題 ID", "Report ID"]), id.clone())]);
        let y = widgets::footnote(
            &mut c,
            t,
            f,
            done + m.dp(8.0),
            l.pick([
                "把上面这个 ID 写在作者的评论区，就能对应到这次反馈的日志。一次开机只能反馈一次，重启后可以再反馈新的问题。",
                "把上面這個 ID 寫在作者的留言區，就能對應到這次回報的記錄。一次開機只能回報一次，重啟後可以再回報新的問題。",
                "Mention this ID where the author takes comments, and it leads to the logs of this report. One report goes per start; start again to report another bug.",
            ]),
        );
        let c = fit(c, t, y);
        return Frame::new(t, c, out, title, top);
    }
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["问题描述", "問題描述", "What went wrong"]));
    let (field, caret) = widgets::words_field(&mut c, t, f, y, &fb.words, WORDS_LINES);
    let mut y = field + m.dp(10.0);
    if fb.words.trim().is_empty() {
        y = widgets::footnote(
            &mut c,
            t,
            f,
            y,
            l.pick([
                "中英文都可以写。中文用上面那一条拼音候选打出来，意思到了就行。",
                "中英文都可以寫。中文用上面那一條拼音候選打出來，意思到了就行。",
                "Write in Chinese or English: Chinese comes from the spelling bar above the keys, and what it means is what matters.",
            ]),
        );
    }
    // What is sent is close under the field: the list is what the page is
    // read for, and the gap above it only put it further away.
    y = widgets::section(&mut c, t, f, y + m.dp(2.0), l.pick(["会上传什么", "會上傳什麼", "What is sent"]));
    let sent = widgets::list(
        &mut c,
        t,
        f,
        y,
        &[
            Row::new("kernel.log").subtitle(l.pick(["内核本次开机的记录", "核心本次開機的記錄", "The kernel's messages since this start"])),
            Row::new("recovery.log").subtitle(l.pick(["界面与操作本次开机的记录", "介面與操作本次開機的記錄", "What the pages and actions did this start"])),
        ],
    );
    y = widgets::section(&mut c, t, f, sent.bottom, l.pick(["隐私", "隱私", "Privacy"]));
    y = widgets::footnote(
        &mut c,
        t,
        f,
        y,
        l.pick([
            "上传的只有上面两个日志和这里写的内容。日志用于定位这个 bug，不会公开。",
            "上傳的只有上面兩個記錄和這裡寫的內容。記錄用於定位這個 bug，不會公開。",
            "What goes is the two logs above and what you write here. They are read to place this bug, and are not made public.",
        ]),
    );
    // The report is written after the bug, and the button says so where it is
    // touched: a report from before it leaves nothing in the logs to find.
    y = widgets::footnote(
        &mut c,
        t,
        f,
        y + m.dp(8.0),
        l.pick([
            "请在 bug 出现之后再反馈：没触发 bug 就反馈，日志里看不到问题，也就无从查起。",
            "請在 bug 出現之後再回報：沒觸發 bug 就回報，記錄裡看不到問題，也就無從查起。",
            "Send this after the bug has happened: a report from before it leaves nothing in the logs to look for.",
        ]),
    );
    // The button sits above the note rather than under it: a report is sent
    // without reading to the end, and the note is read while waiting.
    y += m.dp(20.0);
    let end = match &fb.report {
        Report::Sending { done, total } => {
            let detail = tr!(l, "已上传 {done} / {total} 字节", "已上傳 {done} / {total} 位元組", "{done} of {total} bytes sent");
            widgets::progress_card(&mut c, t, f, y, l.pick(["正在上传", "正在上傳", "Sending"]), &detail, fb.report.share())
        }
        Report::Refused { why, again } => {
            let y = widgets::section(&mut c, t, f, y, l.pick(["上传失败", "上傳失敗", "Not sent"]));
            let y = widgets::footnote(&mut c, t, f, y, why);
            if *again {
                let retry = widgets::button(&mut c, t, f, y + m.dp(16.0), l.pick(["重试", "重試", "Try again"]));
                out.push(Hit { rect: retry, action: Action::SendReportAgain });
            }
            let edit = widgets::button(&mut c, t, f, y + m.dp(16.0), l.pick(["重新填写", "重新填寫", "Write again"]));
            let bottom = edit.y + edit.h;
            out.push(Hit { rect: edit, action: Action::WriteReportAgain });
            bottom
        }
        _ => {
            let send = widgets::button(&mut c, t, f, y, l.pick(["提交", "提交", "Send"]));
            let bottom = send.y + send.h;
            out.push(Hit { rect: send, action: Action::SendReport });
            bottom
        }
    };
    // The keyboard is held at the foot of the screen rather than drawn on the
    // page: a page that scrolls would take the keys with it, and the report is
    // written over more lines than a screen holds. A report on its way, or one
    // turned down, is read rather than written: the keys would cover it.
    let dock = if matches!(fb.report, Report::Writing) {
        Some(keys_dock(t, f, s, l.pick(["换行", "換行", "Return"])))
    } else {
        None
    };
    // What the page scrolls to is where its content stops: `fit` adds the
    // margin under it, and the keys at the foot are counted in the page's
    // height rather than taken from it, so the last of it stops above them.
    let c = fit(c, t, end);
    Frame { caret: Some(caret), dock, content_end: Some(end), ..Frame::new(t, c, out, title, top) }
}

const LICENSE_ORIGINAL: &str = "Skyward\nCopyright (C) 2026 Kirk Lin\n\n\
This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.\n\n\
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.\n\n\
You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.";

/// Skyward's own licence.
pub fn license(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let y = widgets::info_card(&mut c, t, f, top, &[(l.pick(texts::license::TERMS_TITLE), l.pick(texts::license::TERMS).to_string())]);
    let bottom = match l.pick(texts::license::ORIGINAL_ALONE) {
        true => widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), LICENSE_ORIGINAL),
        false => {
            let y = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), l.pick(texts::license::NOTICE));
            let y = widgets::section(&mut c, t, f, y, l.pick(texts::license::ORIGINAL));
            widgets::paragraph(&mut c, t, f, y - t.m.dp(16.0), LICENSE_ORIGINAL)
        }
    };
    Frame::new(t, fit(c, t, bottom), Vec::new(), l.pick(texts::page::LICENSE), top)
}

/// The monospaced typeface's licence, which names the copyright holders on
/// its first lines; the image carries the same file beside the font.
const MONO_LICENSE: &str = include_str!("../../../resources/fonts/KirkMono-OFL.txt");

/// The emoji typefaces' licence, which names the copyright holder on its
/// first lines; the image carries the same file beside the fonts.
const EMOJI_LICENSE: &str = include_str!("../../../resources/fonts/NotoColorEmoji-OFL.txt");

/// The typefaces, as they name themselves, and the terms each is used under.
pub fn font_license(cx: &Ctx) -> Frame {
    let (t, f, l) = (cx.t, cx.f, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let (typeface, version, copyright) = (l.pick(["字体", "字型", "Typeface"]), l.pick(["版本", "版本", "Version"]), l.pick(["版权", "版權", "Copyright"]));
    let y = widgets::section_first(&mut c, t, f, top, l.pick(["界面文字", "介面文字", "Interface text"]));
    let y =
        widgets::info_card(&mut c, t, f, y, &[(typeface, f.info.family.clone()), (version, f.info.version.clone()), (copyright, f.info.copyright.clone().expect("MiSans names its copyright holder"))]);
    let misans = l.pick([
        "界面文字使用小米 MiSans 字体，依据《MiSans 字体知识产权许可协议》免费使用。字体文件未经修改，仅随 Skyward 提供，不单独分发。\n\n协议全文可在 MiSans 官方网站查看。",
        "介面文字使用小米 MiSans 字型，依據《MiSans 字體知識產權許可協議》免費使用。字型檔案未經修改，僅隨 Skyward 提供，不單獨散布。\n\n協議全文可在 MiSans 官方網站查看。",
        "The interface text is set in Xiaomi's MiSans, used free of charge under the MiSans Font Intellectual Property License Agreement. The font files are unchanged, and come with Skyward alone, never on their own.\n\nThe agreement in full is on the MiSans website.",
    ]);
    let y = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), misans);
    let y = widgets::section(&mut c, t, f, y, l.pick(["终端文字", "終端機文字", "Terminal text"]));
    let holders: Vec<&str> = MONO_LICENSE.lines().take_while(|l| l.starts_with("Copyright")).collect();
    let y = widgets::info_card(
        &mut c,
        t,
        f,
        y,
        &[
            (typeface, f.mono_info.family.clone()),
            (version, f.mono_info.version.clone()),
            (copyright, holders.join("\n")),
            (l.pick(LICENSE_TERMS), l.pick(["SIL 开源字体许可证 1.1 版（OFL-1.1）", "SIL 開放字型授權條款 1.1 版（OFL-1.1）", "SIL Open Font License, version 1.1 (OFL-1.1)"]).to_string()),
        ],
    );
    let mono = l.pick([
        "终端文字使用 Kirk Mono 等宽字体。该许可证允许自由使用、修改和再发布字体，但不允许单独出售字体。\n\n许可证全文位于 /skyward/KirkMono-OFL.txt，也可在 openfontlicense.org 查看。",
        "終端機文字使用 Kirk Mono 等寬字型。這份授權條款允許自由使用、修改和再散布字型，但不允許單獨出售字型。\n\n授權條款全文位於 /skyward/KirkMono-OFL.txt，也可在 openfontlicense.org 查看。",
        "The terminal text is set in the monospaced Kirk Mono. Its license lets the font be used, changed and passed on freely, but not sold on its own.\n\nThe license in full is at /skyward/KirkMono-OFL.txt, and on openfontlicense.org.",
    ]);
    let y = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), mono);
    // The emoji are drawn from a typeface of their own, which is under the
    // same licence as the terminal's but names a different holder.
    let y = widgets::section(&mut c, t, f, y, l.pick(["表情符号", "表情符號", "Emoji"]));
    let holders: Vec<&str> = EMOJI_LICENSE.lines().take_while(|l| l.starts_with("Copyright")).collect();
    let y = widgets::info_card(
        &mut c,
        t,
        f,
        y,
        &[
            (typeface, "Noto Color Emoji".to_string()),
            (copyright, holders.join("\n")),
            (l.pick(LICENSE_TERMS), l.pick(["SIL 开源字体许可证 1.1 版（OFL-1.1）", "SIL 開放字型授權條款 1.1 版（OFL-1.1）", "SIL Open Font License, version 1.1 (OFL-1.1)"]).to_string()),
        ],
    );
    let emoji = l.pick([
        "名字里的表情符号使用 Google 的 Noto Color Emoji 绘制。该许可证允许自由使用、修改和再发布字体，但不允许单独出售字体。\n\n许可证全文位于 /skyward/NotoColorEmoji-OFL.txt，也可在 openfontlicense.org 查看。",
        "名字裡的表情符號使用 Google 的 Noto Color Emoji 繪製。這份授權條款允許自由使用、修改和再散布字型，但不允許單獨出售字型。\n\n授權條款全文位於 /skyward/NotoColorEmoji-OFL.txt，也可在 openfontlicense.org 查看。",
        "An emoji in a name is drawn from Google's Noto Color Emoji. Its license lets the font be used, changed and passed on freely, but not sold on its own.\n\nThe license in full is at /skyward/NotoColorEmoji-OFL.txt, and on openfontlicense.org.",
    ]);
    let bottom = widgets::paragraph(&mut c, t, f, y + t.m.dp(4.0), emoji);
    Frame::new(t, fit(c, t, bottom), Vec::new(), l.pick(FONTS), top)
}

/// Which slot the phone starts from next, as the boot control holds it,
/// and what it holds of each; picking the other and swiping switches.
/// While an update is in flight the slot stays as the update set it.
pub fn slot(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let title = l.pick(texts::slot::BOOT_SLOT);
    let Some(current) = i.slot else {
        widgets::list(&mut c, t, f, top, &[Row::new(l.pick(texts::slot::NO_SLOTS)).trail(Trail::None)]);
        return Frame::new(t, c, out, title, top);
    };
    let slots = match &s.boot_slots {
        Some(Ok(slots)) => *slots,
        Some(Err(why)) => {
            let list = widgets::list(&mut c, t, f, top, &[Row::new(l.pick(texts::slot::UNREADABLE)).trail(Trail::None)]);
            widgets::footnote(&mut c, t, f, list.bottom, why);
            return Frame::new(t, c, out, title, top);
        }
        None => return Frame::new(t, c, out, title, top),
    };
    let chosen = s.next_slot.unwrap_or(slots.active);
    let y = widgets::section_first(&mut c, t, f, top, l.pick(texts::reboot::NEXT_BOOT));
    let titles = [Slot::A, Slot::B].map(|slot| texts::slot::named(l, slot.letter()));
    let subs = [Slot::A, Slot::B].map(|slot| {
        let k = slot.number() as usize;
        l.pick(match (slot == current, slots.bootable[k], slots.successful[k]) {
            (true, false, _) => texts::slot::RUNNING_UNBOOTABLE,
            (true, true, false) => texts::slot::RUNNING_UNSTARTED,
            (true, true, true) => texts::slot::RUNNING,
            (false, false, _) => texts::slot::OTHER_UNBOOTABLE,
            (false, true, false) => texts::slot::OTHER_UNSTARTED,
            (false, true, true) => texts::slot::OTHER,
        })
    });
    let rows: Vec<Row> = [Slot::A, Slot::B].iter().enumerate().map(|(k, &slot)| Row::new(titles[k].as_str()).subtitle(subs[k]).trail(Trail::Check(slot == chosen))).collect();
    let list = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&list.rows, &[Slot::A, Slot::B].map(Action::PickSlot)));
    if let Some(note) = fmt::snapshot_slot_note(l, &i.snapshot) {
        widgets::footnote(&mut c, t, f, list.bottom, note);
        return Frame::new(t, c, out, title, top);
    }
    let y = widgets::footnote(&mut c, t, f, list.bottom, l.pick(texts::slot::SWITCH_NOTE));
    let (label, danger) = confirm_label(Confirm::Slot, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::Slot) });
    Frame::new(t, c, out, title, top)
}

/// The two systems as big cards one over the other, laid out as the boot
/// slot page lays out its slots: the main system, in the phone's own
/// partitions, and the second, whose files lie in the data partition, or
/// the place to install one, the one that starts next checked. Picking the
/// other and swiping writes its boot images into the slot that starts
/// next, and restarts into it. Under them, the second system, and removing
/// it.
pub fn systems(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let title = l.pick(texts::page::DUAL_BOOT);
    // While the boot partitions are written, or the second system removed,
    // the page only says so: of a switch the system asked for, perhaps
    // before the systems are read.
    if let Some(work) = s.systems_work {
        let (heading, detail) = match work {
            SystemsWork::Switching(to) => {
                let name = match (&s.systems, to) {
                    (Some(Ok(systems)), Who::Installed) => Some(systems.installed.name.as_str()),
                    (Some(Ok(systems)), Who::Other) => systems.other.as_ref().map(|o| o.name.as_str()),
                    _ => None,
                };
                let heading = match name {
                    Some(name) => texts::systems::switching_to(l, name),
                    None => l.pick(texts::systems::SWITCHING).to_string(),
                };
                (heading, l.pick(texts::systems::WRITING_BOOT_IMAGES))
            }
            SystemsWork::Removing => (l.pick(texts::systems::REMOVING).to_string(), l.pick(texts::systems::GIVING_ROOM_BACK)),
        };
        let y = widgets::progress_card(&mut c, t, f, top, &heading, detail, None);
        let y = widgets::footnote(&mut c, t, f, y, l.pick(texts::systems::KEEP_POWER));
        return Frame::new(t, fit(c, t, y), out, title, top);
    }
    let systems = match &s.systems {
        Some(Ok(systems)) => systems,
        Some(Err(why)) => {
            let list = widgets::list(&mut c, t, f, top, &[Row::new(l.pick(texts::systems::UNREADABLE)).trail(Trail::None)]);
            let y = widgets::footnote(&mut c, t, f, list.bottom, why);
            return Frame::new(t, fit(c, t, y), out, title, top);
        }
        None => return Frame::new(t, fit(c, t, top), out, title, top),
    };
    let starts = starting(systems);
    let chosen = s.system_pick.unwrap_or(starts);
    let second_role = l.pick(texts::systems::SECOND);
    // Boot partitions that hold a second system's image nothing is kept of
    // start neither system the page knows.
    let main = SystemCard {
        role: l.pick(texts::systems::MAIN),
        name: &systems.installed.name,
        detail: systems.installed.version.as_deref().unwrap_or_default(),
        mark: SystemMark::Check(chosen == Who::Installed && systems.held != Held::Unrecorded),
    };
    let second = match &systems.other {
        Some(other) => SystemCard { role: second_role, name: &other.name, detail: &other.version, mark: SystemMark::Check(chosen == Who::Other) },
        None => SystemCard {
            role: second_role,
            name: l.pick(texts::systems::INSTALL),
            detail: l.pick(texts::systems::CHOOSE_PACKAGE),
            mark: SystemMark::Opens,
        },
    };
    let y = widgets::section_first(&mut c, t, f, top, l.pick(texts::reboot::NEXT_BOOT));
    let shown = widgets::system_cards(&mut c, t, f, y, &[main, second]);
    let second_opens = match systems.other {
        Some(_) => Action::PickSystem(Who::Other),
        None => Action::Open(Screen::Install),
    };
    out.extend(hits(&shown.rows, &[Action::PickSystem(Who::Installed), second_opens]));
    let mut y = shown.bottom + t.m.dp(12.0);
    if let Some(why) = &s.system_notice {
        y = widgets::info_card(&mut c, t, f, y, &[(l.pick(texts::systems::SECOND_FAILED), why.clone())]);
    }
    if chosen != starts {
        match (chosen, fmt::snapshot_system_note(l, &i.snapshot)) {
            (Who::Other, Some(note)) => y = widgets::footnote(&mut c, t, f, y, note),
            _ => {
                let (label, danger) = confirm_label(Confirm::SwitchSystem, s);
                let track = widgets::confirm(&mut c, t, f, y + t.m.dp(12.0), label, danger, s.confirm);
                out.push(Hit { rect: track, action: Action::Confirm(Confirm::SwitchSystem) });
                y = track.y + track.h;
            }
        }
    }
    y = widgets::footnote(&mut c, t, f, y, l.pick(texts::systems::WHAT));
    match systems.held {
        Held::Recorded => {}
        Held::Rewritten => y = widgets::footnote(&mut c, t, f, y, l.pick(texts::systems::REWRITTEN)),
        Held::Unrecorded => y = widgets::footnote(&mut c, t, f, y, l.pick(texts::systems::UNRECORDED)),
    }
    if systems.other.is_some() && systems.changed {
        y = widgets::footnote(&mut c, t, f, y, l.pick(texts::systems::CHANGED));
    }
    if let Some(other) = &systems.other {
        y = widgets::section(&mut c, t, f, y, second_role);
        let size = fmt::bytes(other.size);
        let facts = widgets::list(&mut c, t, f, y, &[Row::new(l.pick(texts::systems::SPACE_USED)).trail(Trail::Text(&size))]);
        let mounted = matches!(s.data, Data::Open | Data::Locked { .. });
        let remove = Row::new(l.pick(texts::systems::REMOVE)).tint(t.pal.danger);
        let (remove, act) = match mounted {
            true => (remove.trail(Trail::None), Action::AskDelete),
            false => (remove.subtitle(l.pick(texts::decrypt::NEEDED)), Action::Open(Screen::Decrypt)),
        };
        let removing = widgets::list(&mut c, t, f, facts.bottom + t.m.card_gap, &[remove]);
        out.extend(hits(&removing.rows, &[act]));
        y = removing.bottom;
        if s.asking_delete && mounted {
            let ask = match starts {
                Who::Other => l.pick([
                    "将先切换回主系统，再删除第二系统及其全部数据，删除后无法恢复。",
                    "將先切換回主系統，再刪除第二系統及其全部資料，刪除後無法復原。",
                    "The phone goes back to the main system first, then the second system and all its data are removed for good.",
                ]),
                Who::Installed => {
                    l.pick(["删除第二系统及其全部数据？删除后无法恢复。", "刪除第二系統及其全部資料？刪除後無法復原。", "Remove the second system and all its data? It can't be brought back."])
                }
            };
            let y2 = widgets::footnote(&mut c, t, f, y, ask);
            let (label, danger) = confirm_label(Confirm::RemoveSystem, s);
            let track = widgets::confirm(&mut c, t, f, y2 + t.m.dp(24.0), label, danger, s.confirm);
            out.push(Hit { rect: track, action: Action::Confirm(Confirm::RemoveSystem) });
            y = track.y + track.h;
        }
    }
    Frame::new(t, fit(c, t, y), out, title, top)
}

/// The phone's own fastbootd serving the computer: whether it runs, how far
/// the computer has the port, and where to go from here, chosen with the
/// keys and lit where they are.
pub fn fastboot(cx: &Ctx) -> Frame {
    let (t, f, s, i, l) = (cx.t, cx.f, cx.state, cx.info, cx.state.lang);
    let mut c = page(cx);
    let top = t.m.content_top;
    let running = l.pick(if s.fastboot.running { texts::fastboot::RUNNING } else { texts::fastboot::NOT_RUNNING });
    let link = l.pick(match s.fastboot.link {
        UsbLink::Connected => texts::fastboot::CONNECTED,
        UsbLink::Connecting => texts::fastboot::CONNECTING,
        UsbLink::Unplugged => texts::fastboot::NO_COMPUTER,
    });
    let slot = i.slot.map(|s| s.letter());
    let mut facts =
        vec![Row::new("fastbootd").trail(Trail::Text(running)), Row::new("USB").trail(Trail::Text(link)), Row::new(l.pick(["设备代号", "裝置代號", "Codename"])).trail(Trail::Text(&i.codename))];
    if let Some(letter) = slot {
        facts.push(Row::new(l.pick(texts::slot::CURRENT)).trail(Trail::Text(letter)));
    }
    let y = widgets::section_first(&mut c, t, f, top, l.pick(texts::fastboot::STATUS));
    let state = widgets::list(&mut c, t, f, y, &facts);
    let y = widgets::footnote(&mut c, t, f, state.bottom, l.pick(texts::fastboot::NOTE));
    let y = widgets::section(&mut c, t, f, y, l.pick(texts::fastboot::LEAVE));
    let rows: Vec<Row> = FASTBOOT_CHOICES.iter().map(|&(title, _)| Row::new(l.pick(title)).trail(Trail::None)).collect();
    let choices = widgets::list(&mut c, t, f, y, &rows);
    let actions: Vec<Action> = FASTBOOT_CHOICES.iter().map(|&(_, action)| action).collect();
    let out = hits(&choices.rows, &actions);
    widgets::footnote(&mut c, t, f, choices.bottom, l.pick(texts::fastboot::KEYS));
    Frame { back: false, ..Frame::new(t, c, out, "Fastboot", top) }
}

/// Where to go once what the system asked for has stopped or cannot be
/// done, in the order the pages list them.
const AFTERWARDS: [(Text, Action); 3] = [
    (texts::installed::REBOOT_TO_SYSTEM, Action::RebootTo(Target::System)),
    (texts::afterwards::ENTER_RECOVERY, Action::Dismiss),
    (texts::reboot::target::POWER_OFF, Action::RebootTo(Target::PowerOff)),
];

/// The list of [`AFTERWARDS`] at `y`, and where it ends. Work the user
/// chose in the recovery goes back to its home page, rather than into it.
fn afterwards(c: &mut Canvas, t: &Theme, f: &Fonts, l: Lang, y: f32, asked: bool, out: &mut Vec<Hit>) -> f32 {
    let rows: Vec<Row> =
        AFTERWARDS.iter().map(|&(title, action)| Row::new(l.pick(if action == Action::Dismiss && !asked { &["返回主页", "返回主頁", "Back to home"][..] } else { &title })).trail(Trail::None)).collect();
    let choices = widgets::list(c, t, f, y, &rows);
    out.extend(hits(&choices.rows, &AFTERWARDS.map(|(_, action)| action)));
    choices.bottom
}

/// Erasing: each part and how far it has come, and once erasing has
/// stopped, why. What the system asked for leaves on its own once it is
/// done, and offers where to go only if it stops; what the user chose
/// offers where to go once it ends either way.
pub fn erase(cx: &Ctx) -> Frame {
    use std::cmp::Ordering;
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let e = &s.erase;
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let failed = matches!(e.outcome, Some(Err(_)));
    // Only a part that went wrong was under way; whatever else stopped
    // erasing stopped it before any part began.
    let begun = !failed || matches!(e.outcome, Some(Err(EraseStop::Failed(..))));
    let heading = l.pick(match &e.outcome {
        None => texts::wipe::WIPING,
        Some(Ok(())) => texts::wipe::WIPED,
        Some(Err(_)) => texts::wipe::INCOMPLETE,
    });
    let mut y = widgets::section_first(&mut c, t, f, top, heading);
    if !e.parts.is_empty() {
        let states: Vec<String> = (0..e.parts.len())
            .map(|k| match (k.cmp(&e.done), failed) {
                (Ordering::Less, _) => l.pick(texts::wipe::WIPED).into(),
                (Ordering::Equal, false) => match e.progress {
                    Some((done, total)) if total > 0 => {
                        format!("{} {}%", l.pick(WIPING), percent(share(done, total)))
                    }
                    _ => l.pick(WIPING).into(),
                },
                (Ordering::Equal, true) if begun => l.pick(texts::common::UNFINISHED).into(),
                (Ordering::Greater, false) => l.pick(texts::common::WAITING).into(),
                (Ordering::Equal | Ordering::Greater, true) => l.pick(texts::wipe::NOT_WIPED).into(),
            })
            .collect();
        let rows: Vec<Row> = e.parts.iter().zip(&states).map(|(&part, state)| Row::new(fmt::part(l, part)).trail(Trail::Text(state))).collect();
        y = widgets::list(&mut c, t, f, y, &rows).bottom;
    }
    let notes: Vec<String> = match (&e.outcome, e.asked, e.reset) {
        (Some(Err(stop)), _, _) => {
            let (what, detail) = fmt::erase_stop(l, stop);
            std::iter::once(what).chain(detail).collect()
        }
        (_, true, true) => {
            let leaving = match s.asked.shutdown_after {
                true => {
                    ["本机的全部数据将被清除，完成后将自动关机。", "本機的全部資料將被清除，完成後將自動關機。", "All of this phone's data is being wiped, and it powers off on its own once done."]
                }
                false => {
                    ["本机的全部数据将被清除，完成后将自动重启。", "本機的全部資料將被清除，完成後將自動重新啟動。", "All of this phone's data is being wiped, and it reboots on its own once done."]
                }
            };
            vec![l.pick(leaving).into()]
        }
        (_, true, false) => vec![l.pick(if s.asked.shutdown_after { POWERS_OFF_WHEN_DONE } else { REBOOTS_WHEN_DONE }).into()],
        (None, false, _) => vec![l.pick(KEEP_POWER).into()],
        (Some(Ok(())), false, true) => {
            vec![
                l.pick([
                    "下次开机时将重新创建 data 分区，启动时间较长。",
                    "下次開機時將重新建立 data 分割區，啟動時間較長。",
                    "The data partition is made anew at the next start, which then takes longer.",
                ])
                .into(),
            ]
        }
        (Some(Ok(())), false, false) => Vec::new(),
    };
    for note in &notes {
        y = widgets::footnote(&mut c, t, f, y, note);
    }
    if let (true, Some(reason)) = (e.asked, &s.asked.reason) {
        y = widgets::info_card(&mut c, t, f, y + t.m.dp(12.0), &[(l.pick(texts::installing::REASON), reason.clone())]);
    }
    if failed || (!e.asked && e.outcome.is_some()) {
        let below = widgets::section(&mut c, t, f, y, l.pick(texts::common::ACTIONS));
        y = afterwards(&mut c, t, f, l, below, e.asked, &mut out);
    }
    let title = l.pick(match (e.reset, e.asked) {
        (true, _) => texts::wipe::RESET,
        (false, true) => texts::erase::WIPE_CACHE,
        (false, false) => texts::page::WIPE,
    });
    Frame { back: false, ..Frame::new(t, fit(c, t, y), out, title, top) }
}

/// The system could not start Android and asks whether to erase all user
/// data: start it again, look around the recovery first, or erase.
pub fn cant_start(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let mut y = widgets::paragraph(&mut c, t, f, top, l.pick(texts::cant_start::WHAT));
    if let Some(reason) = &s.asked.reason {
        y = widgets::info_card(&mut c, t, f, y + t.m.dp(12.0), &[(l.pick(texts::installing::REASON), reason.clone())]);
    }
    y = widgets::section(&mut c, t, f, y, l.pick(texts::common::ACTIONS));
    let rows = [
        Row::new(l.pick(texts::common::TRY_AGAIN)).subtitle(l.pick(texts::cant_start::STARTS_AGAIN)).trail(Trail::None),
        Row::new(l.pick(texts::afterwards::ENTER_RECOVERY)).subtitle(l.pick(texts::cant_start::WIPES_NOTHING)).trail(Trail::None),
    ];
    let choices = widgets::list(&mut c, t, f, y, &rows);
    out.extend(hits(&choices.rows, &[Action::RebootTo(Target::System), Action::Dismiss]));
    let y = widgets::footnote(&mut c, t, f, choices.bottom, l.pick(texts::cant_start::CANT_UNDO));
    let (label, danger) = confirm_label(Confirm::FactoryReset, cx.state);
    let track = widgets::confirm(&mut c, t, f, y + t.m.dp(24.0), label, danger, s.confirm);
    out.push(Hit { rect: track, action: Action::Confirm(Confirm::FactoryReset) });
    Frame { back: false, ..Frame::new(t, c, out, l.pick(texts::cant_start::TITLE), top) }
}

/// Something the system asked of the recovery that it does not do: what
/// was asked, and where to go instead.
pub fn unhandled(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let mut items = vec![(l.pick(texts::unhandled::REQUEST), s.asked.unhandled.clone().unwrap_or_default())];
    if let Some(reason) = &s.asked.reason {
        items.push((l.pick(texts::installing::REASON), reason.clone()));
    }
    let y = widgets::info_card(&mut c, t, f, top, &items);
    let y = widgets::footnote(
        &mut c,
        t,
        f,
        y,
        l.pick(["Skyward 暂不支持该请求，本机未做任何改动。", "Skyward 尚不支援這項要求，本機未做任何變更。", "Skyward doesn't support this request, and changed nothing on this phone."]),
    );
    let y = widgets::section(&mut c, t, f, y, l.pick(ACTIONS));
    afterwards(&mut c, t, f, l, y, true, &mut out);
    Frame { back: false, ..Frame::new(t, c, out, l.pick(texts::unhandled::TITLE), top) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::BackupRefusal;

    fn job(restore: bool, checks: bool) -> BackupJob {
        let plan = vec![("boot".into(), 200), ("system".into(), 800)];
        BackupJob { restore, place: Place::Storage, checks, plan, at: None, outcome: None, stopping: false }
    }

    fn at(item: usize, done: u64, of: u64, checking: bool) -> BackupAt {
        BackupAt { item, done, of, checking }
    }

    #[test]
    fn a_script_shows_its_last_line_until_skyward_goes_back() {
        let said = ["- Flashing boot".to_string(), "  ".to_string()];
        let l = Lang::Hans;
        assert_eq!(script_stage(l, None, &[]), ("正在运行安装脚本", None));
        assert_eq!(script_stage(Lang::En, None, &[]), ("Running the install script", None));
        assert_eq!(script_stage(l, Some(&Stage::Script { thousandths: 250 }), &said), ("- Flashing boot", Some(0.25)), "past a blank line");
        assert_eq!(script_stage(l, Some(&Stage::Reflashing { done: 1, total: 4 }), &said), ("正在重新刷入 Skyward", Some(0.25)));
        assert_eq!(update_stage(Lang::En, Some(&Stage::Reflashing { done: 3, total: 4 })), ("Flashing Skyward again".to_string(), Some(0.75)));
        assert_eq!(update_stage(Lang::Hant, Some(&Stage::Writing { done: 1, total: 4 })).0, "第 1 步，共 3 步：寫入新系統 25%");
    }

    #[test]
    fn a_backup_checks_its_parts_last_and_a_restore_first() {
        // Halfway through writing system, 600 of the 1000 bytes: the first
        // of two passes, then the second.
        assert_eq!(job_progress(&job(false, true), &at(1, 400, 800, false)), (0.3, 600));
        assert_eq!(job_progress(&job(false, true), &at(1, 400, 800, true)), (0.8, 600));
        assert_eq!(job_progress(&job(false, false), &at(1, 400, 800, false)), (0.6, 600), "without a check, writing is all");
        assert_eq!(job_progress(&job(true, true), &at(0, 100, 200, true)), (0.05, 100));
        assert_eq!(job_progress(&job(true, true), &at(0, 100, 200, false)), (0.55, 100));
    }

    #[test]
    fn each_part_says_where_it_stands() {
        let states = |j: &BackupJob| (0..2).map(|k| part_state(Lang::Hans, j, k)).collect::<Vec<_>>();
        let mut b = job(false, true);
        assert_eq!(states(&b), ["等待中", "等待中"]);
        b.at = Some(at(1, 400, 800, false));
        assert_eq!(states(&b), ["已备份", "正在备份 50%"]);
        b.at = Some(at(0, 50, 200, true));
        assert_eq!(states(&b), ["正在校验 25%", "已备份"]);
        b.outcome = Some(BackupEnd::Failed("the backup does not hold what was stored".into()));
        assert_eq!(states(&b), ["未完成", "未校验"]);
        b.outcome = Some(BackupEnd::Saved("20261021-141639".into()));
        assert_eq!(states(&b), ["已校验", "已校验"]);
        let mut r = job(true, true);
        r.at = Some(at(1, 0, 800, true));
        assert_eq!(states(&r), ["已校验", "正在校验 0%"]);
        r.at = Some(at(0, 200, 400, false));
        assert_eq!(states(&r), ["正在恢复 50%", "已校验"]);
        r.outcome = Some(BackupEnd::Failed("write /dev/block/by-name/boot_b".into()));
        assert_eq!(states(&r), ["未完成", "未恢复"]);
        r.at = None;
        r.outcome = Some(BackupEnd::Refused(BackupRefusal::OlderAndroid { was: "17".into(), now: "16".into() }));
        assert_eq!(states(&r), ["未开始", "未开始"]);
    }

    #[test]
    fn each_part_says_where_it_stands_in_the_language() {
        let mut b = job(false, true);
        b.at = Some(at(1, 400, 800, false));
        assert_eq!((0..2).map(|k| part_state(Lang::En, &b, k)).collect::<Vec<_>>(), ["Backed up", "Backing up 50%"]);
        let mut r = job(true, true);
        r.at = Some(at(0, 200, 400, false));
        assert_eq!((0..2).map(|k| part_state(Lang::Hant, &r, k)).collect::<Vec<_>>(), ["正在還原 50%", "已驗證"]);
    }
}
