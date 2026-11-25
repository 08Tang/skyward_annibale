//! What the pages show and the user changes, all in one place.

use super::*;

/// The folder served to the network the phone is on: not served, served and
/// where, or why it could not be served.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Serving {
    #[default]
    Off,
    Running {
        /// The address a browser on the same network opens.
        at: String,
        /// How many requests it has answered, so the page shows it is alive.
        served: u64,
    },
    Failed(String),
}

/// How a download asked for on the download page stands.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Fetch {
    /// Nothing asked for yet.
    #[default]
    Idle,
    /// How many bytes have come, and how many the server named where it did.
    Running {
        done: u64,
        total: Option<u64>,
    },
    Done {
        path: String,
        bytes: u64,
    },
    Failed(String),
    Stopped,
}

/// What the search field is looking through: the whole of what the phone
/// offers, or the entries of one page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchScope {
    Features,
    Apps,
    Modules,
    /// The folder the file manager shows: a name is looked for among what it
    /// holds, and the entry found is the one opened.
    Files,
}

/// The warmest and coolest the charge may be let to warm the battery to, in
/// whole degrees Celsius, and where the slider starts.
pub const THERMAL_LIMIT_LEAST: u8 = 40;
pub const THERMAL_LIMIT_MOST: u8 = 55;
pub const THERMAL_LIMIT_DEFAULT: u8 = 45;

/// How fast the phone is let to charge, as the charge page offers it: three
/// steps, each the programs the recovery starts for it, since it is the
/// phone's own programs that tell the kernel what a charge may draw and
/// whether the charger may be verified.
///
/// A step reaches the phone at the next start of the recovery, not at once:
/// the programs are started once, as the recovery comes up, and what the
/// secret keeper tells the kernel of a charge cannot be taken back while the
/// cable is in — so ending it later leaves the charge as fast as it would
/// have been, and starting it later would have nothing to say until the phone
/// was started again anyway. The page says so, since a step that looked as if
/// it had taken at once would have the page saying one thing and the phone
/// doing another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ChargeMode {
    /// Neither program: the kernel holds the charge at the ceiling it sets
    /// for a battery it has not been told passed its check.
    #[default]
    Plain,
    /// Xiaomi's charging service, which passes on the battery's check: the
    /// ceiling lifts, and the charge draws what a plain power delivery
    /// connection gives.
    Fast,
    /// That service and Xiaomi's secret keeper, which has the charger
    /// verified: the protocol goes on to the adjustable kind, and the charge
    /// draws what the charger can give.
    Fastest,
}

impl ChargeMode {
    /// The steps, in the order the page offers them.
    pub const ALL: [ChargeMode; 3] = [ChargeMode::Plain, ChargeMode::Fast, ChargeMode::Fastest];

    /// The name it is kept by in the settings file, so a later version that
    /// lists the steps otherwise reads it the same.
    pub fn name(self) -> &'static str {
        match self {
            ChargeMode::Plain => "plain",
            ChargeMode::Fast => "fast",
            ChargeMode::Fastest => "fastest",
        }
    }

    /// The step a settings file names, where it names one this version has.
    pub fn named(name: &str) -> Option<ChargeMode> {
        ChargeMode::ALL.into_iter().find(|mode| mode.name() == name)
    }

    /// Whether Xiaomi's charging service is started for it.
    pub fn charging_service(self) -> bool {
        matches!(self, ChargeMode::Fast | ChargeMode::Fastest)
    }

    /// Whether Xiaomi's secret keeper is started for it.
    pub fn secret_keeper(self) -> bool {
        matches!(self, ChargeMode::Fastest)
    }

    /// The step below this one, where there is one: what the thermal guard
    /// chooses when the battery gets too warm.
    pub fn stepped_down(self) -> Option<ChargeMode> {
        match self {
            ChargeMode::Plain => None,
            ChargeMode::Fast => Some(ChargeMode::Plain),
            ChargeMode::Fastest => Some(ChargeMode::Fast),
        }
    }

    /// Whether a charge at this step is fast enough for its warmth to be
    /// worth watching: the plain step starts neither of Xiaomi's programs,
    /// and what it draws cannot warm the battery enough to matter, so the
    /// thermal guard is neither offered nor obeyed there.
    pub fn guarded(self) -> bool {
        !matches!(self, ChargeMode::Plain)
    }
}

/// One module the phone's own system would load: KernelSU's and Magisk's keep
/// them the same way, a folder each with a `module.prop` beside it, and a
/// `disable` file inside a folder that is switched off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleRow {
    /// The folder's name, which is how the module is known.
    pub id: String,
    pub name: String,
    /// What module.prop says of it: its version and who made it.
    pub note: String,
    /// Its own words about what it does.
    pub about: String,
    pub enabled: bool,
    /// Marked to be removed the next time the phone starts.
    pub removing: bool,
}

/// Choices the pages show and the user changes, and where installing
/// stands.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// The language the pages speak.
    pub lang: Lang,
    /// The language was chosen on the language page, now or at an earlier
    /// start, where otherwise it is the one the system names.
    pub language_chosen: bool,
    pub dark: bool,
    pub brightness: f32,
    /// How long the screen stays on with nothing touched, by its place in
    /// [`SCREEN_OFF`].
    pub screen_off: usize,
    /// The clock and the times on the pages keep the time zone the
    /// installed system is set to, where that is known.
    pub follow_system_zone: bool,
    /// The time zone chosen on its pages, by its place in [`TIME_ZONES`],
    /// which the clock keeps while the system's is not followed or not
    /// known.
    pub time_zone: usize,
    /// The time zone the installed system is set to, as far as it is
    /// known: read from the data partition once it opens, and until then
    /// the one read at an earlier start, or the one the system is built to
    /// start in.
    pub system_zone: Option<SystemZone>,
    /// The time, in Unix seconds and to the minute, at which the time zone
    /// pages tell how far each zone is from UTC.
    pub now: i64,
    pub vibrate: bool,
    /// How strongly the motor taps, from 0 at the slider's left end to 1
    /// at its right.
    pub vibration: f32,
    /// A computer connected over USB is offered internal storage.
    pub file_transfer: bool,
    pub reboot: usize,
    /// What the wipe page has chosen, by place in [`WIPES`].
    pub wipe: [bool; WIPES.len()],
    /// The phone has a cache partition, which the wipe page then offers.
    pub cache_partition: bool,
    /// How many folders of its own the phone keeps logs in on the data
    /// partition, which erasing the logs takes in.
    pub phone_logs: usize,
    pub pattern: Vec<usize>,
    /// How far the swipe-to-confirm knob is dragged, 0 to 1.
    pub confirm: f32,
    pub install: Install,
    /// Where the package being installed comes from.
    pub source: Source,
    /// The install page was opened from the second system's place in Dual
    /// boot, so only a package that installs a system beside the installed
    /// one is taken there; any other is refused before it is written.
    pub second_system_asked: bool,
    /// The name of the file being installed, when it is one on the phone.
    pub package_name: Option<String>,
    /// What the install script last run showed, its latest lines.
    pub script_output: Vec<String>,
    /// A zip's install script runs only if the zip was signed with one of
    /// the phone's update keys.
    pub check_signature: bool,
    /// The phone keeps its recovery in a partition of each slot, which the
    /// install page then offers to keep Skyward in.
    pub recovery_partitions: bool,
    /// Skyward goes back into the recovery partitions an install writes
    /// over.
    pub keep_recovery: bool,
    /// How Skyward went back into the recovery partitions the install
    /// wrote over, where it did.
    pub reflash: Option<Reflash>,
    /// The mount page's partitions, as the runtime last read them, or why
    /// it could not.
    pub mounts: Result<Vec<MountRow>, String>,
    pub mount_failure: Option<MountFailure>,
    /// Which of [`BACKUP_PARTS`] a backup takes.
    pub backup: [bool; 5],
    /// Where a backup goes.
    pub backup_to: Place,
    pub compress: bool,
    pub verify: bool,
    /// The backups found, newest first, those that cannot be read last.
    pub backups: Vec<BackupEntry>,
    /// The backup the restore page shows, as it was found, and which of its
    /// parts to restore.
    pub restore: Option<(BackupEntry, [bool; 5])>,
    /// Backing up or restoring under way, or how it last ended.
    pub backup_job: Option<BackupJob>,
    /// Which logs to export: the recovery's, the kernel's.
    pub logs: [bool; 2],
    /// Where the logs go: 0 for the temporary folder, 1 for internal
    /// storage.
    pub logs_to: usize,
    /// What the phone's boot control holds of the slots, as the runtime
    /// last read it for the slot or reboot page, or why it could not.
    pub boot_slots: Option<Result<BootSlots, String>>,
    /// The slot picked on the slot page to start next; with none picked,
    /// the page shows the one the boot control starts next.
    pub next_slot: Option<Slot>,
    /// The phone has what a second system needs: boot and init_boot
    /// partitions in both slots, which a switch writes, and recovery
    /// partitions apart from them, which keep Skyward through it.
    pub dual_boot: bool,
    /// Whether this build offers to update itself, as its device says.
    pub updates: bool,
    /// The modules installed for the phone's own system, read when the page
    /// opens: the folder each sits in, what its module.prop says of it, and
    /// whether it is switched on.
    pub modules: Option<Result<Vec<ModuleRow>, String>>,
    /// The module the detail page shows, by its place in the list.
    pub module_at: Option<usize>,
    /// The apps installed in the data partition, newest first, read when the
    /// page opens; none until they are, and a reason where they could not be.
    pub apps: Option<Result<Vec<crate::app::dataapp::DataApp>, String>>,
    /// The app the detail page shows, by its place in the list.
    pub app_at: Option<usize>,
    /// The apps the system's own partitions hold, for the page that only
    /// looks at them; none until that page is opened.
    pub sys_apps: Option<Vec<crate::app::dataapp::DataApp>>,
    /// Which system apps the system itself has disabled, by package: what the
    /// script wrote back after the last start. None until it has been read.
    pub app_disabled: Option<std::collections::BTreeMap<String, bool>>,
    /// The changes asked for and not carried out yet, by package: the page
    /// shows one as waiting rather than as done, since only the system itself
    /// can change it, at the next start.
    pub app_state_queued: std::collections::BTreeMap<String, bool>,
    /// How many screens of apps the page has been turned by: a screen holds
    /// what fits, and the list is read a screen at a time.
    pub app_page: usize,
    /// The screen of system apps the page shows.
    pub sys_page: usize,
    /// What the search field is looking through, which the page it was opened
    /// from decides.
    pub search_scope: SearchScope,
    /// A copy or a move waiting to be told where it goes: the phone or the
    /// drive, asked before the folder it lands in is walked to.
    pub place_ask: Option<crate::app::FileOp>,
    /// The systems, as the runtime last read them, or why it could not;
    /// none until it has.
    pub systems: Option<Result<Overview, String>>,
    /// The system picked on the systems page to start next; with none
    /// picked, the page shows the one that starts.
    pub system_pick: Option<Who>,
    /// Work on the systems under way.
    pub systems_work: Option<SystemsWork>,
    /// Why the second system's start stopped, as its first program told
    /// it, which the systems page shows.
    pub system_notice: Option<String>,
    /// Where the data partition stands, and what is typed at the screen
    /// lock its users' own files wait on.
    pub data: Data,
    /// How far opening the data partition has come, 0 to 1, which the bar on
    /// the decrypt page runs toward, and how far the ring about its sign has
    /// turned.
    pub data_phase: f32,
    /// Whether the phone's own parts -- its partitions, its drivers and its
    /// services -- are up. The decrypt page says it waits for them until they
    /// are, rather than that it is at the partition.
    pub prepared: bool,
    pub code: String,
    /// What is typed in the search field.
    pub search: String,
    /// The on-screen keyboard's layer.
    pub layer: Layer,
    /// The key of the on-screen keyboard, or the choice of a list open over
    /// the page, a finger holds down.
    pub held: Option<Action>,
    /// The folder the file manager shows.
    pub files: Folder,
    /// The folder the OTG page shows.
    pub otg: Folder,
    /// The screen lock is about to be kept, and the risk of keeping it is
    /// shown before it is.
    pub asking_remember: bool,
    /// The entry the file page shows was opened on the OTG page, so its path
    /// is measured from the top of the drive rather than the owner's storage.
    pub file_on_drive: bool,
    /// The date the lock screen shows under the clock, as the phone's own
    /// lock screen shows it.
    pub date: String,
    /// The order the file manager shows entries in, kept between starts.
    pub sort: SortKey,
    pub sort_desc: bool,
    pub folders_first: bool,
    /// Whether the panel that chooses the order is showing over the file
    /// manager: it is another screen of that page rather than a page of its
    /// own, so going back closes it rather than leaving the file manager.
    pub sort_panel: bool,
    /// The tiles the control centre lays out, in the order they stand in,
    /// kept between starts.
    pub tiles: Vec<Tile>,
    /// The folder of internal storage a file to install is chosen in, its
    /// path from the top of the storage.
    pub storage: Folder,
    /// The USB drive plugged in, as the runtime last found it, and the
    /// folder of it a file to install is chosen in, its path from the top
    /// of the drive.
    pub usb_drive: Option<UsbDrive>,
    pub drive: Folder,
    /// The entry the file page describes.
    pub file: Option<FileRow>,
    pub fastboot: Fastboot,
    pub erase: Erase,
    pub asked: Asked,
    /// The entry picked to copy or move, while a folder for it is looked
    /// for.
    pub clip: Option<Clip>,
    /// What is typed as an entry's new name.
    pub new_name: String,
    /// The text file the editor page shows, as typed so far.
    pub editor: Option<Editor>,
    /// Deleting the entry the file page shows, or the backup the restore
    /// page shows, was asked for; a swipe confirms it.
    pub asking_delete: bool,
    /// The picture the phone shows for its owner, and the name it gives
    /// itself: both are on the encrypted data partition, so they come once it
    /// is open, and the home page shows a row of them once they do.
    pub avatar: Option<crate::avatar::Avatar>,
    pub device_name: Option<String>,
    /// The file manager is choosing several entries: the names it has chosen,
    /// in the folder it shows.
    pub choosing: bool,
    pub chosen: std::collections::BTreeSet<String>,
    /// Deleting the entries chosen was asked for: their full paths, waiting on
    /// the swipe that confirms it.
    pub asking_many: Option<Vec<String>>,
    /// A copy or move waiting on what to do with the names taken where it
    /// goes, and those names.
    pub clash: Option<(Clip, String, Vec<String>)>,
    /// Work on files under way, or how it last ended.
    pub job: Option<FileJob>,
    /// Giving the data partition's files their SELinux labels under way, or
    /// how it last ended.
    pub relabel: Option<RelabelJob>,
    /// The filesystem page's partitions, as the runtime last read them, or
    /// why it could not; none until it has.
    pub filesystems: Option<Result<Vec<FsRow>, String>>,
    /// Which of them the page for one shows, and what it is to do to it:
    /// a repair, which leaves what it holds as it is, until another is
    /// chosen.
    pub fs_chosen: usize,
    pub fs_action: FsAction,
    /// Work on whole partitions under way, or how it last ended.
    pub part_work: Option<PartWork>,
    /// The partitions an image can be taken from, and the images already
    /// saved, as the runtime last read them, or why it could not; none until
    /// it has.
    pub image_parts: Result<Vec<ImagePart>, String>,
    pub image_files: Result<Vec<ImageFile>, String>,
    /// Whether the image page lists the partitions to take an image from, or
    /// the images already saved.
    pub image_showing_parts: bool,
    /// Whether an image being taken goes to a USB drive rather than internal
    /// storage.
    pub image_to_drive: bool,
    /// Work on an image under way, or how it last ended.
    pub image_job: Option<ImageJob>,
    /// The row of the image page whose swipe is showing: a partition to write
    /// out, or an image to write back or delete, chosen by tapping its row.
    pub image_asking: Option<Confirm>,
    /// What the runtime last read from the charger: the power it delivers,
    /// the protocol it negotiated, and the most it has given this charge.
    pub charge: ChargeNow,
    /// The battery's temperature in tenths of a degree, as the charge page
    /// last read it, which the battery card shows.
    pub battery_temp: Option<i32>,
    /// Whether the kernel's own node says the battery passed its check, which
    /// is what decides the current the charge may draw.
    pub fast_charge_kernel: bool,
    /// How fast the phone is let to charge, as the page offers it. The
    /// programs that settle it are started as the recovery comes up, so the
    /// step reaches the phone at the next start.
    pub charge_mode: ChargeMode,
    /// Whether the charge is stepped down when the battery gets too warm, and
    /// how warm is too warm in whole degrees Celsius.
    ///
    /// A step only counts above the plain one: the plain step starts neither
    /// of Xiaomi's programs, and what a charge draws there cannot warm the
    /// battery enough to be worth watching, so the guard is neither offered
    /// nor obeyed there.
    pub thermal_guard: bool,
    pub thermal_limit: u8,
    /// The bug report being written, and where sending it stands.
    pub feedback: Feedback,
    /// Where an update the system installed stands, as the runtime last
    /// read it.
    pub update: Standing,
    /// The WLAN, as the runtime last told it; none on a phone whose WLAN the
    /// recovery does not bring up.
    pub wlan: Option<Wlan>,
    /// Whether the screen lock that opened the user's files is kept for the
    /// starts that follow, so the decrypt page needs nothing typed.
    pub remember: bool,
    /// The folder served to the network, where it is served and how many
    /// requests it has answered; and whether it is being chosen, the file
    /// manager shown for it.
    pub serving: Serving,
    pub picking: bool,
    /// The address typed on the download page, how its download stands, and
    /// where the page another device types an address for it is.
    pub fetch_url: String,
    pub fetch: Fetch,
    pub fetch_portal: Option<String>,
    /// Whether the WLAN opens as the recovery starts, and whether the
    /// password of the network joined last is kept with the network itself.
    pub wlan_auto: bool,
    pub wlan_keep: bool,
    /// The network the password page is for, and what is typed there.
    pub joining: Option<Network>,
    /// The network the details page is of, as it was found.
    pub details: Option<Network>,
    /// The network the page that adds one is adding.
    pub adding: Option<Adding>,
    /// The password typed on the password page or the page that adds a
    /// network, and whether it shows.
    pub password: String,
    pub show_password: bool,
    /// The phone maker's update server is known, and the update page asks
    /// it.
    pub update_server: bool,
    pub check: Check,
    pub have: Have,
    /// Downloading the newest system's full package, or how that last
    /// ended.
    pub download: Option<DownloadJob>,
    pub skyward_check: SkywardCheck,
    /// Updating Skyward, or how that last ended.
    pub skyward_job: Option<SkywardJob>,
    /// The running slot's AVB state, as the runtime last read it, or why
    /// it could not.
    pub avb: Option<Result<AvbInfo, String>>,
    /// How an AVB change or backup ended.
    pub avb_notice: Option<String>,
    /// How fast data went over the network joined in the last second.
    pub traffic: Traffic,
}

impl Default for State {
    fn default() -> Self {
        State {
            lang: Lang::Hans,
            language_chosen: false,
            dark: false,
            brightness: SLIDER_START,
            // Two minutes.
            screen_off: 3,
            follow_system_zone: true,
            time_zone: listed(FIRST_ZONE).expect("the first zone is one listed"),
            system_zone: None,
            now: 0,
            vibrate: true,
            vibration: SLIDER_START,
            file_transfer: true,
            reboot: 0,
            wipe: [false; WIPES.len()],
            cache_partition: false,
            phone_logs: 0,
            pattern: Vec::new(),
            confirm: 0.0,
            install: Install::Idle,
            source: Source::Computer,
            second_system_asked: false,
            package_name: None,
            script_output: Vec::new(),
            check_signature: false,
            recovery_partitions: false,
            keep_recovery: true,
            reflash: None,
            mounts: Ok(Vec::new()),
            mount_failure: None,
            backup: [true, true, false, false, true],
            backup_to: Place::Storage,
            compress: false,
            verify: true,
            backups: Vec::new(),
            restore: None,
            backup_job: None,
            logs: [true, true],
            logs_to: 0,
            boot_slots: None,
            next_slot: None,
            dual_boot: false,
            updates: true,
            modules: None,
            module_at: None,
            apps: None,
            app_at: None,
            sys_apps: None,
            app_disabled: None,
            app_state_queued: std::collections::BTreeMap::new(),
            app_page: 0,
            sys_page: 0,
            search_scope: SearchScope::Features,
            place_ask: None,
            systems: None,
            system_pick: None,
            systems_work: None,
            system_notice: None,
            data: Data::Closed,
            data_phase: 0.0,
            prepared: false,
            code: String::new(),
            search: String::new(),
            layer: Layer::Letters,
            held: None,
            files: Folder::new("/"),
            otg: Folder::new(""),
            asking_remember: false,
            file_on_drive: false,
            date: String::new(),
            sort: SortKey::Name,
            sort_desc: false,
            folders_first: true,
            sort_panel: false,
            tiles: DEFAULT.to_vec(),
            storage: Folder::new(""),
            usb_drive: None,
            drive: Folder::new(""),
            file: None,
            fastboot: Fastboot { running: false, link: UsbLink::Unplugged },
            erase: Erase::default(),
            asked: Asked::default(),
            clip: None,
            new_name: String::new(),
            editor: None,
            asking_delete: false,
            avatar: None,
            device_name: None,
            choosing: false,
            chosen: std::collections::BTreeSet::new(),
            asking_many: None,
            clash: None,
            job: None,
            relabel: None,
            filesystems: None,
            fs_chosen: 0,
            fs_action: FsAction::Repair,
            part_work: None,
            image_parts: Err("the partitions have not been read yet".into()),
            image_files: Err("the images have not been read yet".into()),
            image_showing_parts: true,
            image_to_drive: false,
            image_job: None,
            image_asking: None,
            charge: ChargeNow::default(),
            battery_temp: None,
            fast_charge_kernel: false,
            charge_mode: ChargeMode::default(),
        thermal_guard: true,
        thermal_limit: THERMAL_LIMIT_DEFAULT,
        feedback: Feedback::default(),
            update: Standing::None,
            wlan: None,
            remember: false,
            serving: Serving::Off,
            picking: false,
            fetch_url: String::new(),
            fetch: Fetch::Idle,
            fetch_portal: None,
            wlan_auto: false,
            wlan_keep: false,
            joining: None,
            details: None,
            adding: None,
            password: String::new(),
            show_password: false,
            update_server: false,
            check: Check::Idle,
            have: Have::Nothing,
            download: None,
            skyward_check: SkywardCheck::Idle,
            skyward_job: None,
            avb: None,
            avb_notice: None,
            traffic: Traffic::default(),
        }
    }
}

impl State {
    /// The key of the on-screen keyboard a finger holds down.
    pub fn key_held(&self) -> Option<Cap> {
        match self.held {
            Some(Action::Type(cap)) => Some(cap),
            _ => None,
        }
    }

    /// The folder the list on page `screen` shows: the file manager's, or
    /// a place's to choose a file to install in.
    pub fn folder(&self, screen: Screen) -> &Folder {
        match screen {
            Screen::Files => &self.files,
            Screen::FilesOtg => &self.otg,
            Screen::Choose(Place::Storage) => &self.storage,
            Screen::Choose(Place::UsbDrive) => &self.drive,
            other => unreachable!("{other:?} lists no folder"),
        }
    }

    pub(super) fn folder_mut(&mut self, screen: Screen) -> &mut Folder {
        match screen {
            Screen::Files => &mut self.files,
            Screen::FilesOtg => &mut self.otg,
            Screen::Choose(Place::Storage) => &mut self.storage,
            Screen::Choose(Place::UsbDrive) => &mut self.drive,
            other => unreachable!("{other:?} lists no folder"),
        }
    }

    /// Whether the files of `place` can be read: internal storage once it
    /// is open, a USB drive once the runtime shows it.
    /// The folder the page shows, as the phone knows it: the OTG page keeps
    /// the folder it shows from the top of the drive, and the file manager the
    /// whole of the path.
    pub fn folder_here(&self, screen: Screen) -> String {
        match screen {
            Screen::FilesOtg => {
                let at = self.otg.path.trim_matches('/');
                match at.is_empty() {
                    true => super::USB_DRIVE.to_string(),
                    false => format!("{}/{}", super::USB_DRIVE, at),
                }
            }
            // A page that lists no folder has nothing to paste into.
            other => match other {
                Screen::Files => self.files.path.clone(),
                Screen::Choose(Place::Storage) => self.storage.path.clone(),
                Screen::Choose(Place::UsbDrive) => self.drive.path.clone(),
                _ => String::new(),
            },
        }
    }

    pub fn place_open(&self, place: Place) -> bool {
        match place {
            Place::Storage => self.data.storage_open(),
            Place::UsbDrive => self.usb_drive == Some(UsbDrive::Shown),
        }
    }

    /// Whether a network is joined and has given an address.
    pub fn online(&self) -> bool {
        matches!(&self.wlan, Some(Wlan { joined: Some(j), .. }) if matches!(j.stage, Joining::Connected(_)))
    }

    /// The newest system the update server told of.
    pub fn newest(&self) -> Option<&Release> {
        match &self.check {
            Check::Answered(Some(r)) => Some(r),
            _ => None,
        }
    }

    /// Whether the full package is being downloaded.
    pub fn downloading(&self) -> bool {
        self.download.as_ref().is_some_and(|d| d.outcome.is_none())
    }

    /// Skyward's newest release, as GitHub told of it.
    pub fn skyward_newest(&self) -> Option<&Newest> {
        match &self.skyward_check {
            SkywardCheck::Answered(n) => Some(n),
            _ => None,
        }
    }

    /// Whether Skyward is being updated.
    pub fn updating_skyward(&self) -> bool {
        self.skyward_job.as_ref().is_some_and(|j| j.outcome.is_none())
    }

    /// Whether the phone has the page `screen`: the WLAN page where the
    /// recovery brings the phone's WLAN up, and updating Skyward where the
    /// phone also keeps its recovery in a partition of each slot.
    /// The modules read for the page, or why they could not be.
    pub fn set_modules(&mut self, rows: Result<Vec<ModuleRow>, String>) {
        self.modules = Some(rows);
    }

    /// The apps read for the page, or why they could not be.
    pub fn set_apps(&mut self, rows: Result<Vec<crate::app::dataapp::DataApp>, String>) {
        self.apps = Some(rows);
    }

    pub fn offers(&self, screen: Screen) -> bool {
        match screen {
            Screen::Wlan => self.wlan.is_some(),
            Screen::SkywardUpdate => self.updates && self.wlan.is_some() && self.recovery_partitions,
            Screen::Systems => self.dual_boot,
            // A report goes to an address, so the page is offered where the
            // phone can reach one.
            Screen::Feedback => self.wlan.is_some(),
            _ => true,
        }
    }
}
