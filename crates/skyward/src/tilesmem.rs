//! Which tiles the control centre lays out, and in what order, kept for the
//! starts that follow.
//!
//! It is kept under `/metadata/skyward/`, as the file manager's order is: that
//! is the one place that survives a start without touching the owner's own
//! data.

use anyhow::{Context, Result};
use skyward_ui::app::{Tile, from_ids, to_text};
use std::path::Path;

const FILE: &str = "/metadata/skyward/tiles";

/// The order kept, if one was; `None` where none was ever kept, which leaves
/// the sheet laying out its default tiles.
pub fn read() -> Option<Vec<Tile>> {
    with_metadata(read_unmounted)
}

/// `/metadata` is mounted for the reading: the pages are built before recovery
/// mounts it, and reading a file there before that finds nothing at all -- the
/// same way the file manager's order and the WLAN kept were lost every start.
fn with_metadata<T>(work: impl FnOnce() -> T) -> T {
    let at = Path::new("/metadata");
    let mounted = skyward_block::mount::is_mount_point(at).unwrap_or(false);
    if !mounted && skyward_block::mount::mount_ro(&Path::new(skyward_block::BY_NAME).join("metadata"), at).is_err() {
        return work();
    }
    let done = work();
    if !mounted {
        let _ = skyward_block::mount::umount(at);
    }
    done
}

fn read_unmounted() -> Option<Vec<Tile>> {
    let text = std::fs::read_to_string(FILE).ok()?;
    let tiles = from_ids(text.lines());
    // An empty file counts as none kept: it is more likely a write cut off by
    // the power than an owner who wants no tiles at all.
    (!tiles.is_empty()).then_some(tiles)
}

/// Keep the order for the next start.
pub fn write(tiles: &[Tile]) -> Result<()> {
    let body = to_text(tiles);
    let at = Path::new(FILE);
    // `/metadata` comes up read-only: writing needs it made writable for the
    // moment it takes, as the kept WLAN's own file does.
    let _ = skyward_block::mount::remount_writable(Path::new("/metadata"), true);
    let written = std::fs::write(at, body).with_context(|| format!("write {}", at.display()));
    let _ = skyward_block::mount::remount_writable(Path::new("/metadata"), false);
    written
}
