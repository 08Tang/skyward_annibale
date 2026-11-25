//! The tiles the control centre lays out: which kinds there are, what each
//! one is called, and the order they stand in.
//!
//! The order is kept in `/metadata/skyward/tiles`, one tile's name to a line;
//! a phone that has never kept one lays out [`DEFAULT`]. The names are for the
//! file to read, so changing one would lose the order it stands for.

use super::*;

/// One kind of tile the control centre can lay out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tile {
    // The first twelve are the ones the sheet always had, and the order a
    // phone that has kept none lays out: an owner coming up from an older
    // build sees the panel they knew, and the twelve added later wait under
    // the editor.
    Wlan,
    Vibrate,
    FileTransfer,
    Usb,
    Files,
    Modules,
    Partitions,
    Terminal,
    Screenshot,
    /// A take of the screen, which the capsule shows while it runs.
    Record,
    Reboot,
    Settings,
    // Added once the pages that were only reachable from their own lists
    // could be opened from the panel too.
    Backup,
    Restore,
    Mount,
    Filesystems,
    Logs,
    Advanced,
    DataApps,
    Systems,
    ImageBackup,
    PowerOff,
    Fetch,
    ScreenOff,
}

/// What a phone that has kept no order lays out: the twelve the sheet always
/// had, four to a row and three rows.
pub const DEFAULT: [Tile; 12] = [
    Tile::Wlan,
    Tile::Vibrate,
    Tile::FileTransfer,
    Tile::Usb,
    Tile::Files,
    Tile::Modules,
    Tile::Partitions,
    Tile::Terminal,
    Tile::Record,
    Tile::Screenshot,
    Tile::Reboot,
    Tile::Settings,
];

/// Every tile, in the order a phone that has kept none shows them: the first
/// twelve are [`DEFAULT`], the rest come after.
pub const TILES: [Tile; 24] = [
    Tile::Wlan,
    Tile::Vibrate,
    Tile::FileTransfer,
    Tile::Usb,
    Tile::Files,
    Tile::Modules,
    Tile::Partitions,
    Tile::Terminal,
    Tile::Record,
    Tile::Screenshot,
    Tile::Reboot,
    Tile::Settings,
    Tile::Backup,
    Tile::Restore,
    Tile::Mount,
    Tile::Filesystems,
    Tile::Logs,
    Tile::Advanced,
    Tile::DataApps,
    Tile::Systems,
    Tile::ImageBackup,
    Tile::PowerOff,
    Tile::Fetch,
    Tile::ScreenOff,
];

impl Tile {
    /// The name the order is kept under: enough to tell one from another.
    pub fn id(self) -> &'static str {
        match self {
            Tile::Wlan => "wlan",
            Tile::Vibrate => "vibrate",
            Tile::FileTransfer => "mtp",
            Tile::Usb => "usb",
            Tile::Files => "files",
            Tile::Modules => "modules",
            Tile::Partitions => "partitions",
            Tile::Terminal => "terminal",
            Tile::Screenshot => "screenshot",
            Tile::Record => "record",
            Tile::Reboot => "reboot",
            Tile::Settings => "settings",
            Tile::Backup => "backup",
            Tile::Restore => "restore",
            Tile::Mount => "mount",
            Tile::Filesystems => "filesystems",
            Tile::Logs => "logs",
            Tile::Advanced => "advanced",
            Tile::DataApps => "dataapps",
            Tile::Systems => "systems",
            Tile::ImageBackup => "imagebackup",
            Tile::PowerOff => "poweroff",
            Tile::Fetch => "fetch",
            Tile::ScreenOff => "screenoff",
        }
    }

    /// Which tile a name in the file stands for; `None` where it names none.
    pub fn from_id(id: &str) -> Option<Tile> {
        TILES.into_iter().find(|t| t.id() == id.trim())
    }

    /// The word under the tile. The WLAN's own state is drawn apart.
    pub fn label(self, l: Lang) -> &'static str {
        match self {
            Tile::Wlan => "WLAN",
            Tile::Vibrate => l.pick(["震动", "震動", "Haptics"]),
            Tile::FileTransfer => "MTP",
            Tile::Usb => l.pick(["U 盘", "USB 隨身碟", "USB drive"]),
            Tile::Files => l.pick(["文件管理", "檔案管理", "Files"]),
            Tile::Modules => l.pick(["模块管理", "模組管理", "Modules"]),
            Tile::Partitions => l.pick(["分区管理", "分割區管理", "Partitions"]),
            Tile::Terminal => l.pick(["终端", "終端機", "Terminal"]),
            Tile::Screenshot => l.pick(["截图", "截圖", "Screenshot"]),
            Tile::Record => l.pick(["录屏", "錄影", "Record"]),
            Tile::Reboot => l.pick(["重启", "重新啟動", "Reboot"]),
            Tile::Settings => l.pick(["设置", "設定", "Settings"]),
            Tile::Backup => l.pick(["备份", "備份", "Backup"]),
            Tile::Restore => l.pick(["恢复", "還原", "Restore"]),
            Tile::Mount => l.pick(["挂载", "掛載", "Mount"]),
            Tile::Filesystems => l.pick(["文件系统", "檔案系統", "Filesystems"]),
            Tile::Logs => l.pick(["日志", "日誌", "Logs"]),
            Tile::Advanced => l.pick(["高级工具", "進階工具", "Advanced"]),
            Tile::DataApps => l.pick(["应用管理", "應用程式", "Apps"]),
            Tile::Systems => l.pick(["第二系统", "第二系統", "Dual boot"]),
            Tile::ImageBackup => l.pick(["镜像备份", "映像備份", "Images"]),
            Tile::PowerOff => l.pick(["关机", "關機", "Power off"]),
            Tile::Fetch => l.pick(["下载", "下載", "Download"]),
            Tile::ScreenOff => l.pick(["息屏时间", "螢幕逾時", "Screen timeout"]),
        }
    }

    /// Where it stands in the default order: the editor sorts the tiles left
    /// over by this, so one taken off and put back lands where it was.
    pub fn rank(self) -> usize {
        TILES.iter().position(|&t| t == self).unwrap_or(TILES.len())
    }

    /// The tile's own icon, the one the sheet draws in it.
    pub fn icon(self) -> crate::icons::Icon {
        use crate::icons::Icon;
        match self {
            Tile::Wlan => Icon::Wlan,
            Tile::Vibrate => Icon::Phone,
            Tile::FileTransfer => Icon::Usb,
            Tile::Usb => Icon::UsbDrive,
            Tile::Files => Icon::Folder,
            Tile::Modules => Icon::Layers,
            Tile::Partitions => Icon::Mount,
            Tile::Terminal => Icon::Terminal,
            Tile::Screenshot => Icon::Camera,
            Tile::Record => Icon::Record,
            Tile::Reboot => Icon::Restore,
            Tile::Settings => Icon::Gear,
            Tile::Backup => Icon::Backup,
            Tile::Restore => Icon::Restore,
            Tile::Mount => Icon::Mount,
            Tile::Filesystems => Icon::Layers,
            Tile::Logs => Icon::Log,
            Tile::Advanced => Icon::More,
            Tile::DataApps => Icon::Layers,
            Tile::Systems => Icon::Systems,
            Tile::ImageBackup => Icon::Backup,
            Tile::PowerOff => Icon::Power,
            Tile::Fetch => Icon::Install,
            Tile::ScreenOff => Icon::Clock,
        }
    }

    /// The page this tile opens when touched; the three that switch something
    /// on and off, and the screenshot and the recording, have their own ways
    /// and answer `None`.
    pub fn screen(self) -> Option<Screen> {
        match self {
            Tile::Wlan | Tile::Vibrate | Tile::FileTransfer | Tile::Screenshot | Tile::Record => None,
            Tile::Usb => Some(Screen::FilesOtg),
            Tile::Files => Some(Screen::Files),
            Tile::Modules => Some(Screen::Modules),
            Tile::Partitions => Some(Screen::Partitions),
            Tile::Terminal => Some(Screen::Terminal),
            Tile::Reboot => Some(Screen::Reboot),
            Tile::Settings => Some(Screen::Settings),
            Tile::Backup => Some(Screen::Backup),
            Tile::Restore => Some(Screen::Restore),
            Tile::Mount => Some(Screen::Mount),
            Tile::Filesystems => Some(Screen::Filesystems),
            Tile::Logs => Some(Screen::Logs),
            Tile::Advanced => Some(Screen::Advanced),
            Tile::DataApps => Some(Screen::DataApps),
            Tile::Systems => Some(Screen::Systems),
            Tile::ImageBackup => Some(Screen::ImageBackup),
            Tile::PowerOff => Some(Screen::PowerOff),
            Tile::Fetch => Some(Screen::Fetch),
            Tile::ScreenOff => Some(Screen::ScreenOff),
        }
    }

    /// Whether the tile is on, and so wears the accent colour. The three that
    /// switch something say so; the rest open a page and are drawn as off.
    pub fn lit(self, s: &State) -> bool {
        match self {
            Tile::Wlan => s.wlan.as_ref().is_some_and(|w| w.power == Power::On),
            Tile::Vibrate => s.vibrate,
            Tile::FileTransfer => s.file_transfer,
            Tile::Record => s.record.on,
            _ => false,
        }
    }
}

/// The names read out of the file as a row of tiles: those that name nothing
/// are dropped, and of two that name the same only the first stands.
pub fn from_ids<'a>(ids: impl IntoIterator<Item = &'a str>) -> Vec<Tile> {
    let mut out: Vec<Tile> = Vec::new();
    for id in ids {
        if let Some(tile) = Tile::from_id(id)
            && !out.contains(&tile)
        {
            out.push(tile);
        }
    }
    out
}

/// A row of tiles as the file keeps it: one name to a line.
pub fn to_text(tiles: &[Tile]) -> String {
    let mut out = String::new();
    for tile in tiles {
        out.push_str(tile.id());
        out.push('\n');
    }
    out
}

impl State {
    /// The tiles not laid out yet, in the default order.
    pub fn tiles_left(&self) -> Vec<Tile> {
        let mut left: Vec<Tile> = TILES.into_iter().filter(|t| !self.tiles.contains(t)).collect();
        left.sort_by_key(|t| t.rank());
        left
    }

    /// Take `tile` off the panel; a tile that was not laid out is left alone.
    pub fn tile_remove(&mut self, tile: Tile) -> bool {
        match self.tiles.iter().position(|&t| t == tile) {
            Some(at) => {
                self.tiles.remove(at);
                true
            }
            None => false,
        }
    }

    /// Lay `tile` out in the last place; one already laid out stays where it
    /// is.
    pub fn tile_add(&mut self, tile: Tile) -> bool {
        if self.tiles.contains(&tile) {
            return false;
        }
        self.tiles.push(tile);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::testing::*;

    #[test]
    fn a_tile_keeps_its_name_through_the_file() {
        for tile in TILES {
            assert_eq!(Tile::from_id(tile.id()), Some(tile), "{tile:?} reads back");
        }
        assert_eq!(Tile::from_id("no such tile"), None);
        assert_eq!(Tile::from_id("  wlan \n"), Some(Tile::Wlan), "the space around it is no part of the name");
    }

    #[test]
    fn a_file_read_back_drops_what_is_not_a_tile_and_keeps_one_of_each() {
        let read = from_ids(["wlan", "nope", "files", "wlan", "", "screenshot"]);
        assert_eq!(read, vec![Tile::Wlan, Tile::Files, Tile::Screenshot]);
        assert_eq!(from_ids(std::iter::empty()), Vec::new());
    }

    #[test]
    fn the_text_goes_round_the_file() {
        let tiles = vec![Tile::Screenshot, Tile::Wlan, Tile::Settings];
        let text = to_text(&tiles);
        assert_eq!(text, "screenshot\nwlan\nsettings\n");
        assert_eq!(from_ids(text.lines()), tiles);
    }

    #[test]
    fn a_tile_taken_off_goes_back_to_its_own_place_among_those_left() {
        let mut s = State::default();
        assert!(s.tile_remove(Tile::Files));
        assert!(!s.tiles.contains(&Tile::Files));
        assert!(!s.tile_remove(Tile::Files), "it was not laid out");
        let left = s.tiles_left();
        assert!(left.contains(&Tile::Files), "one taken off is one the editor offers");
        assert!(s.tile_add(Tile::Files));
        assert_eq!(s.tiles.last(), Some(&Tile::Files), "one put back lands in the last place");
        assert!(!s.tile_add(Tile::Files), "one already laid out is not laid out twice");
    }

    #[test]
    fn the_first_eleven_stand_by_default_and_the_rest_wait_below() {
        let s = State::default();
        assert_eq!(s.tiles, DEFAULT.to_vec(), "the panel starts as it always was");
        let left = s.tiles_left();
        assert_eq!(left.len(), TILES.len() - DEFAULT.len(), "the rest wait in the editor");
        assert_eq!(left.first(), Some(&Tile::Backup), "the editor offers them in the default order");
        assert_eq!(left.last(), Some(&Tile::ScreenOff));
    }

    /// Every tile that opens a page opens its own: the ones added later had
    /// nothing but their own page to reach.
    #[test]
    fn every_tile_that_opens_a_page_opens_its_own() {
        let (f, i) = (fonts(), info());
        for tile in TILES {
            let Some(screen) = tile.screen() else { continue };
            let mut app = App::new(Screen::Home, W, H, PPD);
            app.state.tiles = vec![tile];
            app.shade = 1.0;
            app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
            let (x, y) = app
                .shade_hits
                .iter()
                .find(|(_, h)| *h == ShadeHit::Tile(tile))
                .expect("the tile answers a touch")
                .0
                .centre();
            app.touch(Touch::Down(x, y), &f, 1000);
            app.touch(Touch::Up, &f, 1000);
            assert_eq!(app.screen(), screen, "{tile:?} opens {screen:?}");
        }
    }

    /// The pencil at the top right opens the editor; a tile no longer needs to
    /// be held down for it.
    #[test]
    fn the_pencil_at_the_top_right_opens_the_editor() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Screenshot, Tile::Files, Tile::Settings];
        app.shade = 1.0;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        assert!(!app.shade_editing, "the sheet comes down as it always was");
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TilesEdit).expect("the pencil answers a touch").0.centre();
        let fx = {
            app.touch(Touch::Down(x, y), &f, 1000);
            app.touch(Touch::Up, &f, 1000)
        };
        assert!(app.shade_editing, "one touch opens it");
        assert_eq!(fx, vec![Effect::Haptic], "and the motor says so");
        assert!(app.shade_drag.is_none(), "nothing is picked up by a touch");
    }

    /// A tile touched is a tile acted on, however long the finger stays: the
    /// editor is opened by its pencil alone.
    #[test]
    fn holding_a_tile_does_not_open_the_editor() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Screenshot, Tile::Files, Tile::Settings];
        app.shade = 1.0;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::Tile(Tile::Screenshot)).expect("the first tile answers a touch").0.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        app.tick(1000 + 3000);
        assert!(!app.shade_editing, "holding it does not open the editor");
        assert!(app.shade_drag.is_none(), "and picks nothing up");
    }

    /// The recording tile has no page to open, so a touch on it must start a
    /// take itself: a tile that answered nothing at all would be the whole of
    /// the recording's control and do nothing.
    #[test]
    fn the_recording_tile_starts_a_take() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Record];
        // A take is written to internal storage, so it is only started once
        // that is open; the page says why when it is not.
        app.state.data = Data::Open;
        app.shade = 1.0;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::Tile(Tile::Record)).expect("the tile answers a touch").0.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        let fx = app.touch(Touch::Up, &f, 1000);
        assert!(
            fx.iter().any(|e| matches!(e, Effect::RecordStart)),
            "the tile asks for a take: {fx:?}"
        );
    }

    /// In the editor a tile held down and dragged lands where the finger lets
    /// it go.
    #[test]
    fn a_tile_dragged_in_the_editor_lands_where_the_finger_lets_it_go() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Screenshot, Tile::Files, Tile::Settings, Tile::Wlan];
        app.shade = 1.0;
        app.shade_editing = true;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::Tile(Tile::Screenshot)).expect("the tile answers a touch").0.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        assert_eq!(app.shade_drag.map(|d| d.tile), Some(Tile::Screenshot), "the editor picks it up");
        let grid = TileGrid::of(&app.theme(), app.shade_top());
        let (tx, ty) = grid.centre(2);
        app.touch(Touch::Move(tx, ty), &f, 1100);
        let fx = app.touch(Touch::Up, &f, 1200);
        assert_eq!(app.state.tiles, vec![Tile::Files, Tile::Settings, Tile::Screenshot, Tile::Wlan], "it lands in the third place");
        assert!(app.shade_drag.is_none(), "the finger let it go");
        assert!(fx.iter().any(|e| matches!(e, Effect::KeepTiles(t) if t == &app.state.tiles)), "and it is kept: {fx:?}");
    }

    /// In the editor the small cross on a tile takes it off, and a tile offered
    /// below puts it back.
    #[test]
    fn the_editor_takes_tiles_off_and_puts_them_back_by_tapping() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Screenshot, Tile::Files];
        app.shade = 1.0;
        app.shade_editing = true;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TileRemove(Tile::Files)).expect("the cross answers a touch").0.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        app.touch(Touch::Up, &f, 1000);
        assert_eq!(app.state.tiles, vec![Tile::Screenshot], "the cross takes it off");
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TileAdd(Tile::Terminal)).expect("the tile offered answers a touch").0.centre();
        app.touch(Touch::Down(x, y), &f, 1100);
        app.touch(Touch::Up, &f, 1100);
        assert_eq!(app.state.tiles, vec![Tile::Screenshot, Tile::Terminal], "and the one offered is laid out");
    }

    /// The tiles added later are offered too, and one touch lays one out.
    #[test]
    fn the_new_tiles_stand_in_the_candidate_area_too() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = DEFAULT.to_vec();
        app.shade = 1.0;
        app.shade_editing = true;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        for tile in [Tile::Backup, Tile::PowerOff, Tile::ScreenOff] {
            let found = app.shade_hits.iter().any(|(_, h)| *h == ShadeHit::TileAdd(tile));
            assert!(found, "{tile:?} is offered");
        }
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TileAdd(Tile::Backup)).expect("it answers a touch").0.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        app.touch(Touch::Up, &f, 1000);
        assert_eq!(app.state.tiles.last(), Some(&Tile::Backup), "one touch lays it out in the last place");
    }

    /// Done closes the editor, and a tile touched while editing runs nothing:
    /// the editor is not the place to open a page from.
    #[test]
    fn done_ends_the_editing_and_a_tap_does_not_fire_a_tile_while_editing() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Files, Tile::Settings];
        app.shade = 1.0;
        app.shade_editing = true;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::Tile(Tile::Files)).expect("the tile answers a touch").0.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        let fx = app.touch(Touch::Up, &f, 1000);
        assert_eq!(app.screen(), Screen::Home, "a tile touched while editing opens nothing: {fx:?}");
        assert!(app.shade > 0.0, "and the sheet stays");
        assert!(app.shade_editing, "still editing");
        let (x, y) = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TilesDone).expect("done answers a touch").0.centre();
        app.touch(Touch::Down(x, y), &f, 1100);
        app.touch(Touch::Up, &f, 1100);
        assert!(!app.shade_editing, "done closes it");
    }

    /// Putting the sheet away ends the editing too: it comes down as it always
    /// was next time.
    #[test]
    fn closing_the_sheet_ends_the_editing() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.shade = 1.0;
        app.shade_editing = true;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        app.touch(Touch::Down(700.0, H as f32 - 4.0), &f, 1000);
        assert_eq!(app.shade, 0.0, "a touch past the sheet puts it away");
        assert!(!app.shade_editing, "and the editing ends with it");
    }

    /// The sheet says nothing about editing while it is not.
    #[test]
    fn the_sheet_says_nothing_about_editing_while_it_is_not() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.shade = 1.0;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        let editing_hits: Vec<&ShadeHit> = app
            .shade_hits
            .iter()
            .map(|(_, h)| h)
            .filter(|h| matches!(h, ShadeHit::TilesDone | ShadeHit::TileAdd(_) | ShadeHit::TileRemove(_)))
            .collect();
        assert!(editing_hits.is_empty(), "nothing of the editor shows: {editing_hits:?}");
    }

    /// Done's middle stands over the last column's tiles, not over the middle
    /// of its cell: a cell is wider than the disc and the disc is centred in
    /// it, so the cell's middle sits to the right.
    #[test]
    fn the_done_circle_stands_over_the_last_column_of_tiles() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.state.tiles = vec![Tile::Wlan, Tile::Vibrate, Tile::Files, Tile::Settings];
        app.shade = 1.0;
        app.shade_editing = true;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        let grid = TileGrid::of(&app.theme(), app.shade_top());
        let (last_cx, _) = grid.centre(3);
        let done = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TilesDone).expect("done answers a touch").0;
        let (done_cx, _) = done.centre();
        assert!((done_cx - last_cx).abs() < 0.5, "done stands at {done_cx}, the last column at {last_cx}");
    }

    /// Done is a disc, not a pill with a word in it.
    #[test]
    fn the_done_circle_stays_small_and_round() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.shade = 1.0;
        app.shade_editing = true;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        let done = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TilesDone).expect("done answers a touch").0;
        let m = app.theme().m;
        // The place that answers reaches 6 dp past the disc on each side.
        let drawn = done.w - m.dp(12.0);
        assert!((drawn - m.dp(34.0)).abs() < 0.5, "the disc drawn is {drawn}, and 34 dp is what it should be");
        assert!((done.w - done.h).abs() < 0.5, "the place that answers is square too, since it stands for a disc");
    }

    /// Done's middle is on the clock's own line: the two read as one row.
    #[test]
    fn the_done_circle_sits_on_the_clock_s_line() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        app.shade = 1.0;
        app.shade_editing = true;
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        let done = app.shade_hits.iter().find(|(_, h)| *h == ShadeHit::TilesDone).expect("done answers a touch").0;
        let (_, done_cy) = done.centre();
        let m = app.theme().m;
        let clock_px = m.dp(44.0);
        let clock_y = app.shade_top() + m.dp(88.0);
        let (tt, tb) = f.heading.ink(clock_px);
        let clock_cy = clock_y + (tt + tb) / 2.0;
        assert!((done_cy - clock_cy).abs() < 1.0, "done stands at {done_cy}, the clock's ink at {clock_cy}");
    }

    /// A sheet four tiles tall is one row, and closes up under them: the height
    /// comes from the count rather than from a fixed number.
    #[test]
    fn the_sheet_is_as_tall_as_the_tiles_it_lays_out() {
        let t = Theme::light(W, H, PPD);
        let one_row = shade_height(&t, 4, false);
        let three_rows = shade_height(&t, 12, false);
        assert!(three_rows > one_row, "three rows take more room than one");
        assert_eq!(shade_height(&t, 1, false), one_row, "a single tile still takes one row");
        assert!(shade_height(&t, 4, true) > one_row, "the editor needs the room the tiles offered take");
    }
}
