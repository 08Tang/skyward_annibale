//! Drawing: the page rendered a band at a time, and what the panel shows
//! composed from it, with the bar, the titles and what slides or fades
//! over the page.

use super::*;
use crate::tr;
use crate::icons::{self, Icon};

impl App {
    /// What the status bar shows, its clock reading `clock`: the battery,
    /// the network joined, the data partition's lock, the slot the system
    /// runs from, and the battery's temperature.
    pub fn bar<'a>(&self, clock: &'a str, info: &DeviceInfo) -> Bar<'a> {
        let battery = info.battery.as_ref();
        Bar {
            clock,
            battery: battery.map(|b| (b.level, b.charging().then_some(if b.quick { Bolt::Quick } else { Bolt::Charging }))),
            wlan: self.wlan_mark(),
            data: match self.state.data {
                Data::Erased => None,
                Data::Open => Some(true),
                Data::Closed | Data::Opening | Data::Failed(_) | Data::Locked { .. } => Some(false),
            },
            slot: info.slot.map(Slot::letter),
            temp: battery.and_then(|b| b.temp),
            watts: battery.and_then(|b| b.watts()),
        }
    }

    /// Render the current page if what it shows has changed: the rows of
    /// it on screen, the rest as they come into view; or, where only some
    /// rows changed, those rows.
    pub fn frame(&mut self, f: &Fonts, info: &DeviceInfo) -> &Frame {
        self.render(f, info, self.w)
    }

    /// [`App::frame`], drawing of a page rendered anew only its columns
    /// from the left edge to `wide`.
    fn render(&mut self, f: &Fonts, info: &DeviceInfo, wide: u32) -> &Frame {
        if self.frame.is_some() && !self.stale.is_empty() {
            let t = self.theme();
            let screen = self.screen();
            for rows in std::mem::take(&mut self.stale) {
                let band = Band { rows, cols: (0, self.cols) };
                let cx = Ctx { t: &t, f, info, state: &self.state, term: self.term.as_ref(), band: Some(band) };
                let fresh = pages::render(screen, &cx);
                let Some(mut frame) = self.frame.take() else { break };
                // A page drawn again in part is drawn over the frame there
                // already is, so it has to be as tall as that frame. A page
                // whose cards are as tall as what they have to say is not: a
                // charger card gains or loses a row as the charger comes up or
                // goes, and the charge figures move while it charges. Rather
                // than refuse to draw, the frame is let go and the page drawn
                // whole, which is what a change of height asks for anyway.
                if frame.canvas.h != fresh.canvas.h {
                    self.frame = None;
                    break;
                }
                frame.canvas.replace(&fresh.canvas);
                self.frame = Some(Frame { canvas: frame.canvas, ..fresh });
            }
        }
        if self.frame.is_none() {
            let t = self.theme();
            let screen = self.screen();
            let first = self.top().1.max(0.0) as u32;
            let band = Band { rows: (first, first + self.h), cols: (0, wide.min(self.w)) };
            let cx = Ctx { t: &t, f, info, state: &self.state, term: self.term.as_ref(), band: Some(band) };
            let mut frame = pages::render(screen, &cx);
            // A page drawn in part across holds its rows whole, the columns
            // of them drawn on as they are needed.
            self.cols = frame.canvas.cols().end;
            if self.cols < self.w {
                let mut whole = Canvas::unfilled(self.w, frame.canvas.h, frame.canvas.held());
                whole.replace(&frame.canvas);
                frame.canvas = whole;
            }
            self.height = frame.height(&t);
            self.frame = Some(frame);
            self.clamp_scroll();
            if std::mem::take(&mut self.snap) {
                let (title, bar) = self.bar_wanted().expect("rendered above");
                self.title.jump(title);
                self.bar.jump(bar);
            }
        } else {
            // The page drawn again, whole or in part, may be a different
            // height than it was: a page whose rows depend on what is typed,
            // as the search page's do, is as tall as what it found. Its
            // scroll is worked out from the height it has now.
            let t = self.theme();
            let height = self.frame.as_ref().expect("checked above").height(&t);
            if height != self.height {
                self.height = height;
                self.clamp_scroll();
            }
        }
        self.frame.as_ref().expect("rendered above")
    }

    /// The current page rendered, all of it drawn.
    pub fn whole_frame(&mut self, f: &Fonts, info: &DeviceInfo) -> &Frame {
        let h = self.frame(f, info).canvas.h;
        self.draw_cols(f, info, self.w);
        self.draw_rows(f, info, 0.0, h as f32);
        self.frame.as_ref().expect("rendered above")
    }

    /// Draw a part of the current page not drawn yet: the rest of the
    /// columns of its rows drawn, then a band of rows below those or else
    /// above them, for a page at rest to have all of it drawn by the time
    /// it scrolls into view; that done, make the About page field's dither,
    /// for that page to open without making it. Whether any is left to do.
    pub fn draw_ahead(&mut self, f: &Fonts, info: &DeviceInfo) -> bool {
        let Some(frame) = &self.frame else { return false };
        let (held, h) = (frame.canvas.held(), frame.canvas.h);
        if self.cols < self.w {
            self.draw_cols(f, info, self.w);
        } else if held.end < h {
            self.draw_rows(f, info, held.end as f32, (held.end + 1) as f32);
        } else if held.start > 0 {
            self.draw_rows(f, info, (held.start - 1) as f32, held.start as f32);
        } else if self.dither.is_none() {
            self.dither = Some(glow::Dither::new(self.w as usize, self.h as usize));
        }
        self.cols < self.w || self.frame.as_ref().is_some_and(|fr| !fr.canvas.whole()) || self.dither.is_none()
    }

    /// Draw the columns of the current page's rows drawn from where they
    /// were drawn to, on to `wide`.
    fn draw_cols(&mut self, f: &Fonts, info: &DeviceInfo, wide: u32) {
        let wide = wide.min(self.w);
        let Some(frame) = &self.frame else { return };
        if self.cols >= wide {
            return;
        }
        let held = frame.canvas.held();
        let t = self.theme();
        let band = Band { rows: (held.start, held.end), cols: (self.cols, wide) };
        let cx = Ctx { t: &t, f, info, state: &self.state, term: self.term.as_ref(), band: Some(band) };
        let part = pages::render(self.screen(), &cx).canvas;
        self.frame.as_mut().expect("drawn above").canvas.replace(&part);
        self.cols = wide;
        self.layer = None;
    }

    /// Draw the rows `from..to` of the current page it has not drawn yet,
    /// a band at least a quarter of a screen tall at a time; the rows drawn
    /// are drawn across first.
    fn draw_rows(&mut self, f: &Fonts, info: &DeviceInfo, from: f32, to: f32) {
        let Some(frame) = &self.frame else { return };
        let h = frame.canvas.h;
        let (from, to) = ((from.max(0.0) as u32).min(h), (to.max(0.0).ceil() as u32).min(h));
        let held = frame.canvas.held();
        let least = self.h / 4;
        let mut bands = Vec::new();
        if from < held.start {
            bands.push((from.min(held.start.saturating_sub(least)), held.start));
        }
        if to > held.end {
            bands.push((held.end, to.max(held.end + least).min(h)));
        }
        if bands.is_empty() {
            return;
        }
        self.draw_cols(f, info, self.w);
        let t = self.theme();
        let screen = self.screen();
        for rows in bands {
            let band = Band { rows, cols: (0, self.w) };
            let cx = Ctx { t: &t, f, info, state: &self.state, term: self.term.as_ref(), band: Some(band) };
            let part = pages::render(screen, &cx).canvas;
            self.frame.as_mut().expect("drawn above").canvas.join(&part);
        }
        self.layer = None;
    }

    pub(super) fn max_scroll(&self) -> f32 {
        (self.height - self.h as f32).max(0.0)
    }

    fn clamp_scroll(&mut self) {
        let max = self.max_scroll();
        let top = self.top_mut();
        top.1 = top.1.clamp(0.0, max);
    }

    /// Draw what the panel shows now: the page, the charging screen over it
    /// while a cable has just gone in, both as dark as the content stands
    /// as the panel goes off or comes up.
    pub fn compose(&mut self, out: &mut Canvas, f: &Fonts, info: &DeviceInfo, clock: &str, now: u64) {
        let charge = self.charging.as_ref().map_or(0.0, |c| c.shown.level(now));
        if charge < 1.0 {
            self.compose_page(out, f, info, clock, now);
        }
        if charge > 0.0 {
            self.compose_charging(out, f, info, charge, now);
        }
        if self.shade > 0.0 {
            self.paint_shade(out, f, info, clock, now);
        }
        // The card asking to delete sits over the page, fixed where the eye
        // already is: drawn with the page it would scroll with it.
        // Only the pages whose asking the card carries: an entry's, a backup's,
        // and several at once. The systems page asks with its own swipe.
        let card_page = matches!(self.screen(), Screen::FileInfo | Screen::RestoreBackup);
        let many = self.state.asking_many.is_some() && self.screen() == Screen::Files;
        if (self.state.asking_delete && card_page) || many {
            self.compose_asking(out, f);
        }
        // Keeping the screen lock is a risk taken on: the card saying so sits
        // over the settings page until it is taken across or put away.
        if self.state.asking_remember && matches!(self.screen(), Screen::Settings | Screen::Decrypt) {
            self.compose_remember_risk(out, f);
        }
        // 复制或移动问到哪儿去：两个格子浮在页面中央，点别处就收起来。
        if self.state.place_ask.is_some() {
            self.compose_place(out, f);
        }
        // 镜像的备份、写回和删除都要先滑一下：写在分区上的东西不该被误触。
        if self.state.image_asking.is_some() && self.screen() == Screen::ImageBackup {
            self.compose_image_ask(out, f);
        }
        // The page comes up from black as the panel lights, and goes dark as
        // it goes off. The lock is drawn while the panel is lit, and through
        // the fade if the lock was what the panel was showing when it began to
        // go dark: the page under it is not what was on the screen, and showing
        // it for the moment the backlight takes to close reads as an unlock.
        let shown = self.display_on || self.lock_shown;
        if shown
            && let Some(lock) = &self.lock
        {
            self.compose_lock(out, f, info, clock, lock.drag);
        }
        // The recording's timer is drawn last: over the sheet, over the lock
        // and over any card asking something, so a take is always seen to be
        // running. With no take, its rectangles are cleared -- otherwise that
        // part of the screen would still take a finger.
        if self.state.record.on {
            self.compose_record_pill(out, f);
        } else {
            self.record_pill = None;
        }
        out.dim(self.curtain.level(now));
    }

    /// The card that asks before an entry, or a backup, is deleted: the words,
    /// and the swipe that does it, over the middle of the screen.
    /// The card that says what keeping the screen lock puts at risk, over the
    /// page the switch is on.
    pub(super) fn compose_remember_risk(&mut self, out: &mut Canvas, f: &Fonts) {
        let t = self.theme();
        let l = self.state.lang;
        let what = l.pick([
            "开启后，锁屏密码会保存在本机，每次开机自动解密 data 分区。任何拿到这台手机的人都能借此读取其中全部文件，丢失、被盗或送修时尤其危险。开启即表示你接受这一风险。",
            "開啟後，螢幕鎖定密碼會保存在本機，每次開機自動解密 data 分割區。任何拿到這台手機的人都能藉此讀取其中全部檔案，遺失、被盜或送修時尤其危險。開啟即表示你接受這項風險。",
            "Kept, the screen lock is saved on this phone and opens the data partition by itself every start. Anyone who gets hold of the phone can read every file on it with it — worst of all if it is lost, stolen or sent for repair. Turning it on means you accept that.",
        ]);
        let (label, danger) = crate::pages::confirm_label(Confirm::KeepLock, &self.state);
        let (card, track) = widgets::confirm_card(out, &t, f, what, label, danger, self.state.confirm);
        self.asking = Some((card, Confirm::KeepLock));
        self.asking_track = Some(track);
    }

    /// The card that asks before a partition is written out as an image, an
    /// image is written back over one, or an image is deleted: what is to be
    /// done, and the swipe that does it, over the middle of the screen.
    pub(super) fn compose_image_ask(&mut self, out: &mut Canvas, f: &Fonts) {
        let t = self.theme();
        let l = self.state.lang;
        let Some(which) = self.state.image_asking else { return };
        let what = match which {
            Confirm::ExportImage(k) => {
                let name = match &self.state.image_parts {
                    Ok(parts) => parts.get(k).map(|p| p.name.clone()).unwrap_or_default(),
                    Err(_) => String::new(),
                };
                tr!(l, "把 {name} 分区写成镜像文件？分区有多大，镜像就有多大。", "把 {name} 分割區寫成映像檔？分割區有多大，映像檔就有多大。", "Write {name} out as an image file? The image is as large as the partition.")
            }
            Confirm::ImportImage(k) => {
                let name = match &self.state.image_files {
                    Ok(files) => files.get(k).map(|file| file.partition.clone()).unwrap_or_default(),
                    Err(_) => String::new(),
                };
                tr!(l, "把镜像写回 {name} 分区？该分区现在的内容会被整个覆盖。", "把映像檔寫回 {name} 分割區？該分割區現在的內容會被整個覆蓋。", "Write the image back over {name}? Everything on that partition is replaced.")
            }
            Confirm::DeleteImage(k) => {
                let name = match &self.state.image_files {
                    Ok(files) => files.get(k).map(|file| file.name.clone()).unwrap_or_default(),
                    Err(_) => String::new(),
                };
                tr!(l, "删除镜像\"{name}\"？删除后无法恢复。", "刪除映像檔「{name}」？刪除後無法復原。", "Delete the image “{name}”? It can't be brought back.")
            }
            _ => return,
        };
        let (label, danger) = crate::pages::confirm_label(which, &self.state);
        let (card, track) = widgets::confirm_card(out, &t, f, &what, label, danger, self.state.confirm);
        self.asking = Some((card, which));
        self.asking_track = Some(track);
    }

    /// The question a copy or a move asks first: the phone's own storage, or
    /// the drive. Two pills in a card over the page.
    pub(super) fn compose_place(&mut self, out: &mut Canvas, f: &Fonts) {
        let t = self.theme();
        let m = &t.m;
        let l = self.state.lang;
        let (w, h) = (m.w as f32, m.h as f32);
        out.fill_rect(0.0, 0.0, w, h, Rgba::rgb(0, 0, 0).with_alpha(96));
        let op = self.state.place_ask.expect("asked only when asked");
        let what = match op {
            crate::app::FileOp::Move => l.pick(["移动到哪里？", "移動到哪裡？", "Where does it move to?"]),
            _ => l.pick(["复制到哪里？", "複製到哪裡？", "Where does it copy to?"]),
        };
        let (card_w, card_h) = (w - m.margin * 2.0, m.dp(126.0));
        let (cx, cy) = (m.margin, (h - card_h) / 2.0);
        out.fill_round_rect(cx, cy, card_w, card_h, m.card_radius, t.pal.card);
        out.text_center(&f.heading, what, w / 2.0, cy + m.dp(32.0), m.row_title_px, t.pal.text);
        let ph = m.dp(crate::widgets::PILL_DP);
        let gap = m.dp(12.0);
        let cell = (card_w - m.inner * 2.0 - gap) / 2.0;
        let y = cy + m.dp(52.0);
        let mut hits: Vec<(crate::geom::Rect, crate::app::Action)> = Vec::new();
        for (k, (label, action)) in
            [(l.pick(["内部存储", "內部儲存空間", "The phone"]), crate::app::Action::ToPhone(op)), (l.pick(["U 盘", "USB 隨身碟", "The drive"]), crate::app::Action::ToDrive(op))]
                .into_iter()
                .enumerate()
        {
            let x = cx + m.inner + k as f32 * (cell + gap);
            let fill = t.pal.search;
            crate::widgets::pill_sized(out, f, x, y, ph, m.row_title_px, m.inner, label, fill, t.pal.text);
            hits.push((crate::geom::Rect::new(x, y, cell, ph), action));
        }
        let note = l.pick(["点别处取消", "點別處取消", "Tap elsewhere to cancel"]);
        out.text_center(&f.body, note, w / 2.0, y + ph + m.dp(24.0), m.sub_px, t.pal.subtitle);
        self.place_hits = hits;
    }

    /// The capsule that shows a take running, drawn over everything: over the
    /// sheet, the lock and any card asking something, so it is always seen.
    ///
    /// Its left cell pauses or carries on, its right one ends the take, and a
    /// finger may drag the whole of it anywhere. The rectangles are kept as it
    /// is drawn, since a finger finds it where it is, not where it started.
    pub(super) fn compose_record_pill(&mut self, out: &mut Canvas, f: &Fonts) {
        let t = self.theme();
        let m = &t.m;
        let l = self.state.lang;
        let paused = self.state.record.paused;
        let busy = if self.state.record.elapsed.is_empty() { "0:00" } else { self.state.record.elapsed.as_str() };
        let px = m.dp(13.5);
        let dot = m.dp(6.5);
        let pad = m.dp(11.0);
        let gap = m.dp(8.0);
        let h = m.dp(27.0);
        let label = if paused { Some(l.pick(["已暂停", "已暫停", "Paused"])) } else { None };
        let label_px = m.dp(11.5);
        let wide = out.text_width(&f.strong, busy, px)
            + label.map_or(0.0, |said| gap * 0.6 + out.text_width(&f.value, said, label_px));
        // The right cell: a thin rule, and a white square -- the box a camera
        // stops recording with.
        let btn = dot * 2.0;
        let sep = gap * 0.9;
        let w = pad * 2.0 + dot * 2.0 + gap + wide + sep * 2.0 + btn;
        // Its usual place is the upper right; dragged, it stands where it was
        // put, and a drag past the screen's edge is pushed back inside.
        //
        // Over the sheet it moves above the clock's line, where nothing is in
        // its way: the sheet's own pencil is at the upper right, and the
        // capsule must not cover it while the take still has to be seen. Its
        // left edge follows the date's, so it reads as part of that line.
        let (x, y) = if self.shade > 0.0 {
            let top = self.shade_top();
            let clock_x = m.margin + m.dp(15.0);
            let clock_px = m.dp(44.0);
            let clock_y = top + m.dp(88.0);
            let (tt, _) = f.heading.ink(clock_px);
            let clock_wide = out.text_width(&f.heading, "00:00", clock_px);
            let fallback = (clock_x + clock_wide + m.dp(12.0), clock_y + tt);
            let (left, date_top) = self.shade_date.unwrap_or(fallback);
            let left = left.min(m.w as f32 - m.margin - w).max(m.margin);
            // Its foot stands 8 dp above the date's ink, and never above the
            // sheet's own top edge.
            let y = (date_top - h - m.dp(8.0)).max(top + m.dp(4.0));
            (left, y)
        } else {
            match self.state.record.pill {
                Some((x, y)) => ((x.min(m.w as f32 - w)).max(0.0), (y.min(m.h as f32 - h)).max(0.0)),
                None => (m.w as f32 - m.margin - w, m.dp(46.0)),
            }
        };
        let ink = Rgba::rgb(255, 255, 255);
        out.shadow_round_rect(x, y, w, h, h / 2.0, m.dp(7.0), Rgba::rgb(0, 0, 0).with_alpha(72));
        out.fill_round_rect(x, y, w, h, h / 2.0, Rgba::rgb(16, 16, 18).with_alpha(if paused { 196 } else { 226 }));
        let cx = x + pad + dot;
        let cy = y + h / 2.0;
        if paused {
            // A play triangle: paused, the red dot gives way to the way back
            // into recording.
            out.fill_convex(&[(cx - dot * 0.66, cy - dot), (cx + dot * 0.9, cy), (cx - dot * 0.66, cy + dot)], ink);
        } else {
            out.fill_circle(cx, cy, dot, Rgba::rgb(240, 64, 64));
        }
        let mut left = x + pad + dot * 2.0 + gap;
        out.text_vc(&f.strong, busy, left, cy, px, if paused { ink.with_alpha(190) } else { ink }, None);
        left += out.text_width(&f.strong, busy, px);
        if let Some(said) = label {
            left += gap * 0.6;
            out.text_vc(&f.value, said, left, cy, label_px, ink.with_alpha(150), None);
        }
        // The end of it: a rule between the time and the box, and the box set
        // half a cell to the right of it.
        let cut = x + w - pad - btn - sep * 2.0;
        out.fill_round_rect(cut, cy - m.dp(7.0), m.dp(0.7), m.dp(14.0), m.dp(0.35), ink.with_alpha(56));
        let side = dot * 1.45;
        out.fill_round_rect(cut + sep * 2.0 + (btn - side) / 2.0, cy - side / 2.0, side, side, m.dp(2.0), ink.with_alpha(232));
        self.record_pill = Some(Pill {
            body: crate::geom::Rect::new(x, y, w, h),
            dot: crate::geom::Rect::new(x, y, pad + dot * 2.0 + gap * 0.5, h),
            stop: crate::geom::Rect::new(cut, y, x + w - cut, h),
        });
    }

    pub(super) fn compose_asking(&mut self, out: &mut Canvas, f: &Fonts) {
        // The pages whose asking it carries, and no other: the systems page
        // asks with its own swipe on its own page.
        let many = self.state.asking_many.is_some() && self.screen() == Screen::Files;
        let one = self.state.asking_delete && matches!(self.screen(), Screen::FileInfo | Screen::RestoreBackup);
        if !many && !one {
            return;
        }
        let t = self.theme();
        let l = self.state.lang;
        // Several entries at once are named by how many there are; one by its
        // own name.
        if let Some(paths) = self.state.asking_many.clone() {
            let n = paths.len();
            let what = tr!(l, "删除选中的 {n} 项？删除后无法恢复。", "刪除選取的 {n} 項？刪除後無法復原。", "Delete the {n} entries chosen? They can't be brought back.");
            let (label, danger) = crate::pages::confirm_label(Confirm::DeleteEntry, &self.state);
            let (card, track) = widgets::confirm_card(out, &t, f, &what, label, danger, self.state.confirm);
            self.asking = Some((card, Confirm::DeleteEntry));
            self.asking_track = Some(track);
            return;
        }
        let (name, folder) = match self.screen() {
            Screen::RestoreBackup => {
                let entry = self.state.restore.as_ref().map(|(e, _)| e.name.clone()).unwrap_or_default();
                (entry, false)
            }
            _ => {
                let e = self.state.file.as_ref().expect("the file page shows an entry");
                (e.name.clone(), matches!(e.kind, FileKind::Folder { .. } | FileKind::Link { folder: true, .. }))
            }
        };
        let what = match (self.screen(), folder) {
            (Screen::RestoreBackup, _) => tr!(l, "删除备份\"{name}\"？删除后无法恢复。", "刪除備份「{name}」？刪除後無法復原。", "Delete the backup “{name}”? It can't be brought back."),
            (_, true) => {
                tr!(l, "删除\"{name}\"及其中的全部内容？删除后无法恢复。", "刪除「{name}」及其中的全部內容？刪除後無法復原。", "Delete “{name}” and everything in it? It can't be brought back.")
            }
            _ => tr!(l, "删除\"{name}\"？删除后无法恢复。", "刪除「{name}」？刪除後無法復原。", "Delete “{name}”? It can't be brought back."),
        };
        let confirm = if self.screen() == Screen::RestoreBackup { Confirm::DeleteBackup } else { Confirm::DeleteEntry };
        let (label, danger) = crate::pages::confirm_label(confirm, &self.state);
        let (card, track) = widgets::confirm_card(out, &t, f, &what, label, danger, self.state.confirm);
        self.asking = Some((card, confirm));
        self.asking_track = Some(track);
    }

    /// The control centre, pulled down from the top edge: the time and the
    /// battery, the toggles a recovery has a use for, and the two sliders.
    fn paint_shade(&mut self, out: &mut Canvas, f: &Fonts, info: &DeviceInfo, clock: &str, now: u64) {
        self.shade_hits.clear();
        if self.shade <= 0.0 {
            // Nothing of the sheet shows, so nothing of it rolls either.
            self.shade_roll = None;
            return;
        }
        let t = self.theme();
        let m = t.m;
        let w = m.w as f32;
        let l = self.state.lang;
        let white = Rgba::rgb(255, 255, 255);
        let tall = shade_height(&t, self.state.tiles.len(), self.shade_editing);
        let sheet = tall * self.shade;
        let top = sheet - tall;
        let pane = Rgba::rgb(20, 20, 26).with_alpha(238);
        // What the WLAN tile says: the network joined where one is, and how the
        // WLAN stands otherwise. Kept as its own string, since the tile records
        // where the name has rolled to while it draws it.
        let wlan = self.state.wlan.as_ref().map_or_else(|| l.pick(["已关闭", "已關閉", "Off"]), |w| fmt::wlan_value(l, w)).to_owned();
        // The sheet itself, its foot rounded off.
        //
        // One shape, drawn once. It was a body, two circles and a strip between
        // them, and each of the four laid the pane at its own alpha: where they
        // met along the foot the corners came out laid twice, which is darker
        // than the rest, and the circle's antialiased edge left a wedge of the
        // page showing beside it. One rounded rect has no joins to show.
        //
        // The top corners are square, since the sheet comes down from the top
        // edge; only the foot is rounded, so the radius is given to the bottom
        // alone by drawing the shape a radius taller than it shows and letting
        // the screen's edge cut the top.
        //
        // A barely opened sheet is shorter than the radius, and a radius that
        // large would round its top corners too, leaving the screen's edge
        // showing through them. The radius gives way to the sheet's height.
        let radius = m.dp(30.0).min(sheet.max(0.0));
        out.fill_round_rect(0.0, top - radius, w, sheet - top + radius, radius, pane);
        // A grabber at the foot of the sheet.
        out.fill_round_rect(w / 2.0 - m.dp(18.0), sheet - m.dp(13.0), m.dp(36.0), m.dp(5.0), m.dp(2.5), white.with_alpha(80));
        // The time, with the date beside it on the same line: the date set a
        // size smaller and taking its width from the time, so the two read as
        // one line rather than two blocks. The battery's percentage used to sit
        // at the far right; it is on the card below now, beside the rest of
        // what the phone says of itself.
        let inset = m.dp(15.0);
        let time_px = m.dp(44.0);
        out.text(&f.heading, clock, m.margin + inset, top + m.dp(88.0), time_px, white, None);
        let date = self.state.date.as_str();
        self.shade_date = None;
        if !date.is_empty() {
            let wide = out.text_width(&f.heading, clock, time_px);
            let date_x = m.margin + inset + wide + m.dp(12.0);
            let date_y = top + m.dp(86.0);
            out.text(&f.body, date, date_x, date_y, m.dp(19.0), white.with_alpha(170), None);
            // Where the date begins and where its ink reaches: the recording
            // capsule is set over it, its left edge to the date's and its foot
            // on the date's ink.
            let (dt, _) = f.body.ink(m.dp(19.0));
            self.shade_date = Some((date_x, date_y + dt));
        }
        // The tiles the owner has laid out, four to a row, as many rows as
        // there are: a tile is a disc with its mark in it and its word under
        // it, and the ones that turn something on show lit while it is on.
        //
        // How many there are is the owner's to say, so the sheet's height
        // comes from the count rather than from a fixed number of rows: four
        // tiles take one row and the sheet closes up under them.
        let grid = TileGrid::of(&t, top);
        let (disc, word_gap) = (grid.dia, m.dp(7.0));
        let mut hits = Vec::new();
        for (i, &tile) in self.state.tiles.iter().enumerate() {
            let (cx, cy) = grid.centre(i);
            let lit = tile.lit(&self.state);
            let fill = if lit { t.pal.accent.with_alpha(232) } else { white.with_alpha(30) };
            out.fill_circle(cx, cy, disc / 2.0, fill);
            let ink = if lit { white } else { white.with_alpha(238) };
            crate::icons::glyph(out, tile.icon(), cx, cy, m.dp(26.0), ink);
            // The tile's word, kept inside its cell however long it is: a name
            // wider than the cell would run under the tile beside it.
            let label = if tile == Tile::Wlan { wlan.as_str() } else { tile.label(l) };
            let px = widgets::fitted_px(out, &f.body, label, m.dp(SHADE_LABEL_DP), m.dp(SHADE_NAME_LEAST_DP), grid.cell);
            let travel = out.text_width(&f.body, label, px);
            let baseline = cy + disc / 2.0 + word_gap + m.dp(14.0);
            if travel <= grid.cell {
                if tile == Tile::Wlan {
                    self.shade_roll = None;
                }
                let x = cx - travel / 2.0;
                out.text(&f.body, label, x, baseline, px, ink, None);
            } else if tile == Tile::Wlan {
                // A name still too long at the least size rolls, rather than
                // being cut: the whole of it is read as it passes.
                let since = match self.shade_roll {
                    Some(r) if (r.travel - travel).abs() < 0.5 => r.since,
                    _ => now,
                };
                let roll = Roll { since, travel, room: grid.cell, pace: roll_pace(m.dp(1.0)) };
                self.shade_roll = Some(roll);
                let x = cx - grid.cell / 2.0;
                Self::paint_rolled(out, &f.body, label, x, baseline, px, grid.cell, ink, roll.offset(now));
            } else {
                // Anything else too long is set from its middle and let past
                // its cell, since it names a page rather than a network.
                let x = cx - travel / 2.0;
                out.text(&f.body, label, x, baseline, px, ink, None);
            }
            // While the panel is being put in order every tile carries a small
            // cross at its upper right, and the one a finger has picked up is
            // drawn under the finger rather than in its place.
            //
            // The cross answers first: it stands over the tile's own disc, and
            // a list searched from its front would give every touch of the
            // cross to the tile under it.
            if self.shade_editing {
                let (kx, ky) = (cx + disc / 2.0 - m.dp(3.0), cy - disc / 2.0 + m.dp(3.0));
                out.fill_circle(kx, ky, m.dp(9.0), Rgba::rgb(40, 40, 46));
                let arm = m.dp(3.4);
                let stroke = m.dp(1.6);
                out.line(kx - arm, ky - arm, kx + arm, ky + arm, stroke, white.with_alpha(230));
                out.line(kx + arm, ky - arm, kx - arm, ky + arm, stroke, white.with_alpha(230));
                hits.push((Rect::new(kx - m.dp(15.0), ky - m.dp(15.0), m.dp(30.0), m.dp(30.0)), ShadeHit::TileRemove(tile)));
                // The disc alone answers while editing: a whole cell would take
                // the cross's own touch with it.
                hits.push((Rect::new(cx - disc / 2.0, cy - disc / 2.0, disc, disc), ShadeHit::Tile(tile)));
            } else {
                // The whole cell answers a touch, not the disc alone: a finger
                // is not asked to find the middle of it.
                hits.push((grid.rect(i), ShadeHit::Tile(tile)));
            }
        }
        // The card saying how the phone stands: whether its data is open, the
        // slot it starts from, what is left in the battery and how warm it is.
        let rows = self.state.tiles.len().div_ceil(4).max(1);
        let card_top = grid.top + rows as f32 * grid.row_h + m.dp(4.0);
        let card = Rect::new(m.margin, card_top, w - m.margin * 2.0, m.dp(58.0));
        out.fill_round_rect(card.x, card.y, card.w, card.h, m.dp(18.0), white.with_alpha(20));
        let cells: [String; 4] = [
            fmt::data(l, &self.state.data).to_owned(),
            info.slot.map_or_else(|| "\u{2014}".to_owned(), |s| s.letter().to_owned()),
            info.battery.as_ref().map_or_else(|| "\u{2014}".to_owned(), |b| format!("{}%", b.level)),
            info.battery.as_ref().and_then(|b| b.temp).map_or_else(|| "\u{2014}".to_owned(), fmt::celsius),
        ];
        let caps = [
            l.pick(["数据", "資料", "Data"]),
            l.pick(["槽位", "槽位", "Slot"]),
            l.pick(["电量", "電量", "Battery"]),
            l.pick(["温度", "溫度", "Temp"]),
        ];
        let cell_w = card.w / 4.0;
        for (i, (value, cap)) in cells.iter().zip(caps).enumerate() {
            let cx = card.x + cell_w * i as f32 + cell_w / 2.0;
            let wide = out.text_width(&f.body, value, m.row_title_px);
            out.text(&f.body, value, cx - wide / 2.0, card.y + m.dp(26.0), m.row_title_px, white, None);
            let small = m.dp(12.0);
            let wide = out.text_width(&f.body, cap, small);
            out.text(&f.body, cap, cx - wide / 2.0, card.y + m.dp(45.0), small, white.with_alpha(150), None);
        }
        // The backlight and the motor: two capsules side by side, each filled
        // from its left end as far as its value goes, with the mark alone at
        // its middle -- the name inside was never level with anything. A finger
        // anywhere on one slides it.
        let cap_h = m.dp(46.0);
        let gap = m.dp(10.0);
        let cap_w = (w - m.margin * 2.0 - gap) / 2.0;
        let row = card.y + card.h + m.dp(14.0);
        for (col, (which, hit)) in [(Slider::Brightness, ShadeHit::Brightness), (Slider::Vibration, ShadeHit::Vibration)].into_iter().enumerate() {
            let value = match which {
                Slider::Brightness => self.state.brightness,
                // The sheet's two are the only ones it holds; the charge
                // page's are drawn on the charge page.
                _ => self.state.vibration,
            }
            .clamp(0.0, 1.0);
            let cap = Rect::new(m.margin + col as f32 * (cap_w + gap), row, cap_w, cap_h);
            out.fill_round_rect(cap.x, cap.y, cap.w, cap.h, cap.h / 2.0, white.with_alpha(26));
            // The fill keeps the capsule's own ends: narrow, it is still a
            // capsule, not a bar with one round corner.
            let filled = (cap.w * value).max(cap.h);
            out.fill_round_rect(cap.x, cap.y, filled, cap.h, cap.h / 2.0, t.pal.accent.with_alpha(210));
            // The mark stands at the capsule's left end, level with its
            // middle, as a control's own icon does; the rest of the capsule is
            // the level it is set to.
            Self::shade_glyph(out, &t, hit, cap.x + m.dp(26.0), cap.y + cap.h / 2.0, m.dp(23.0), white.with_alpha(235));
            hits.push((cap, hit));
        }
        // At the right end of the clock's line, a round button: a pencil as a
        // rule, which opens the editor, and a tick while editing, which closes
        // it. It is the one way in -- a tile is no longer held down for it --
        // so it is put where it can be seen and reached.
        //
        // Its middle stands over the middle of the last column's tiles rather
        // than over the middle of their cells: a cell is wider than the disc
        // and the disc is centred in it, so a cell's middle sits to the right.
        {
            let d = m.dp(34.0);
            let (last_cx, _) = grid.centre(3);
            // On the clock's own line: the ink reaches above and below the
            // baseline the time is set on, and the middle of that is the line.
            let clock_px = m.dp(44.0);
            let clock_y = top + m.dp(88.0);
            let (tt, tb) = f.heading.ink(clock_px);
            let (cx, cy) = (last_cx, clock_y + (tt + tb) / 2.0);
            out.fill_circle(cx, cy, d / 2.0, t.pal.accent.with_alpha(215));
            let stroke = m.dp(2.2);
            if self.shade_editing {
                // A tick: two strokes, the short one down and the long one up
                // to the right.
                let s = d * 0.42;
                out.line(cx - s * 0.52, cy + s * 0.04, cx - s * 0.08, cy + s * 0.46, stroke, white);
                out.line(cx - s * 0.08, cy + s * 0.46, cx + s * 0.56, cy - s * 0.40, stroke, white);
            } else {
                icons::glyph(out, Icon::Pencil, cx, cy, d * 0.56, white);
            }
            // A finger is asked to find a disc, not to hit it exactly: the
            // place that answers is wider than the disc by 6 dp on each side.
            let reach = d + m.dp(12.0);
            let which = if self.shade_editing { ShadeHit::TilesDone } else { ShadeHit::TilesEdit };
            hits.push((Rect::new(cx - reach / 2.0, cy - reach / 2.0, reach, reach), which));
        }
        // With no tile laid out that band is empty: a line of words says so,
        // rather than leaving what looks like a page that failed to draw.
        if self.state.tiles.is_empty() {
            let said = if self.shade_editing {
                l.pick(["从下面挑几枚摆上来", "從下面挑幾枚擺上來", "Pick a few from below"])
            } else {
                l.pick(["长按这里可以摆磁贴", "長按這裡可以擺磁貼", "Hold here to choose tiles"])
            };
            out.text(&f.body, said, m.margin, grid.top + m.dp(20.0), m.dp(12.0), white.with_alpha(150), None);
        }
        // While the panel is being put in order the status card gives way: the
        // tiles not laid out yet are listed under it, six to a row.
        if self.shade_editing {
            let add_top = card.y + card.h + m.dp(4.0);
            let left = self.state.tiles_left();
            if left.is_empty() {
                let said = l.pick(["磁贴都摆上了", "磁貼都擺上了", "Every tile is on the sheet"]);
                out.text(&f.body, said, m.margin, add_top + m.dp(18.0), m.dp(12.5), white.with_alpha(150), None);
            } else {
                out.text(&f.body, l.pick(["添加磁贴", "新增磁貼", "Add a tile"]), m.margin, add_top + m.dp(20.0), m.dp(12.5), white.with_alpha(150), None);
                // There are more than twenty to offer, and four to a row would
                // take seven: six to a row, a size smaller, fits them in three.
                let small = m.dp(32.0);
                let label_px = m.dp(9.0);
                let slot_w = (w - m.margin * 2.0) / 6.0;
                let pitch = small + m.dp(24.0);
                for (k, &tile) in left.iter().enumerate() {
                    let (col, line) = (k % 6, k / 6);
                    let cx = m.margin + slot_w * (col as f32 + 0.5);
                    let cy = add_top + m.dp(54.0) + line as f32 * pitch + small / 2.0;
                    out.stroke_circle(cx, cy, small / 2.0, m.dp(1.3), white.with_alpha(115));
                    icons::glyph(out, tile.icon(), cx, cy, small * 0.46, white.with_alpha(200));
                    let name = tile.label(l);
                    // A name too long is cut to the cell: six to a row leaves a
                    // little over 200 pixels for each.
                    let room = slot_w - m.dp(6.0);
                    let shown = if out.text_width(&f.body, name, label_px) <= room {
                        name.to_string()
                    } else {
                        let mut cut = name.to_string();
                        while !cut.is_empty() && out.text_width(&f.body, &cut, label_px) > room {
                            cut.pop();
                        }
                        cut
                    };
                    out.text_center(&f.body, &shown, cx, cy + small / 2.0 + m.dp(10.0), label_px, white.with_alpha(180));
                    let reach = Rect::new(cx - slot_w / 2.0, cy - small / 2.0 - m.dp(3.0), slot_w, pitch - m.dp(6.0));
                    hits.push((reach, ShadeHit::TileAdd(tile)));
                }
            }
        }
        // The tile a finger has picked up follows it, drawn over everything
        // else.
        if let Some(drag) = self.shade_drag {
            let (cx, cy) = (drag.x, drag.y);
            let fill = if drag.tile.lit(&self.state) { t.pal.accent } else { Rgba::rgb(53, 53, 55) };
            out.fill_circle(cx, cy, disc * 0.58, Rgba::rgb(0, 0, 0).with_alpha(60));
            out.fill_circle(cx, cy, disc / 2.0 * 1.08, fill);
            icons::glyph(out, drag.tile.icon(), cx, cy, disc * 0.46 * 1.08, white);
        }
        self.shade_hits = hits;
    }

    /// The WLAN tile's name, set from `x` at `px`, kept inside `room` however
    /// long it is: a name too long to fit is set smaller where that is enough,
    /// and rolled where it is not.
    ///
    /// The roll is the whole name drawn as ink and laid down shifted inside a
    /// strip the width of the tile, so no part of a name is ever cut by its
    /// bytes -- an SSID may be anything at all, and slicing one by index would
    /// split a character.
    #[allow(clippy::too_many_arguments)]
    fn paint_rolled(out: &mut Canvas, face: &crate::font::Face, name: &str, x: f32, baseline: f32, px: f32, room: f32, ink: Rgba, shift: f32) {
        let (_, top) = face.ink(px);
        // A sprite of the name as it stands shifted, in white, kept as
        // coverage alone: it is laid down in the tile's own ink.
        let strip = Sprite::capture(x, baseline + top, room.ceil() as u32, (px * 1.6).ceil() as u32, |c, dx, dy| {
            c.text(face, name, x - shift + dx, baseline + dy, px, Rgba::rgb(255, 255, 255), None);
        });
        strip.draw(out, 0.0, 0.0, ink, 1.0);
    }

    /// The mark a tile or a slider carries.
    fn shade_glyph(out: &mut Canvas, t: &Theme, hit: ShadeHit, cx: f32, cy: f32, size: f32, ink: Rgba) {
        let m = &t.m;
        let stroke = (size * 0.075).max(m.dp(1.6));
        match hit {
            ShadeHit::Tile(tile) => icons::glyph(out, tile.icon(), cx, cy, size, ink),
            // The small cross and the tiles offered below are drawn by
            // `paint_shade` itself, and are no mark in a disc.
            ShadeHit::TileRemove(_) | ShadeHit::TileAdd(_) | ShadeHit::TilesDone | ShadeHit::TilesEdit => {}
            ShadeHit::Vibration => {
                // Three bars, the middle one longer: a motor tapping.
                for (i, tall) in [0.36f32, 0.68, 0.36].iter().enumerate() {
                    let x = cx + (i as f32 - 1.0) * size * 0.24;
                    out.fill_round_rect(x - size * 0.06, cy - size * tall / 2.0, size * 0.12, size * tall, size * 0.06, ink);
                }
            }
            ShadeHit::Brightness => {
                // A sun: its disc and eight rays.
                out.fill_circle(cx, cy, size * 0.19, ink);
                for k in 0..8 {
                    let a = k as f32 * std::f32::consts::TAU / 8.0;
                    let (sin, cos) = a.sin_cos();
                    let (r0, r1) = (size * 0.30, size * 0.45);
                    out.line(cx + cos * r0, cy + sin * r0, cx + cos * r1, cy + sin * r1, stroke, ink);
                }
            }
        }
    }

    /// Where the lock's knob slides: the bar drawn along the foot of it.
    pub(super) fn lock_track(&self) -> Rect {
        // The rect the confirm control draws is the one a finger has to land
        // on: its margins and height, not an inset of our own (2026-11-15, the
        // knob answered nowhere while these differed).
        // It is shorter than the page is wide, and centred: a control the whole
        // width of the screen reads as a bar rather than something to slide.
        let m = &self.theme().m;
        let wide = (m.w as f32 - m.margin * 2.0).min(m.dp(260.0));
        Rect::new((m.w as f32 - wide) / 2.0, m.h as f32 - m.dp(176.0), wide, m.dp(56.0))
    }

    /// The lock over the page: the phone's time, its battery, and the knob to
    /// slide along.
    fn compose_lock(&mut self, out: &mut Canvas, f: &Fonts, info: &DeviceInfo, clock: &str, drag: f32) {
        let t = self.theme();
        let m = &t.m;
        let (w, h) = (m.w as f32, m.h as f32);
        out.fill_rect(0.0, 0.0, w, h, Rgba::rgb(6, 6, 8).with_alpha(226));
        // The date under which the time stands, as a phone's lock screen shows.
        if let Ok(face) = f.face(400.0)
            && !self.state.date.is_empty()
        {
            let px = m.dp(20.0);
            let width = out.text_width(&face, &self.state.date, px);
            out.text(&face, &self.state.date, (w - width) / 2.0, h * 0.245, px, Rgba::rgb(158, 162, 172), None);
        }
        if let Ok(face) = f.face(600.0) {
            let px = m.dp(64.0);
            let baseline = h * 0.335;
            let width = out.text_width(&face, clock, px);
            out.text(&face, clock, (w - width) / 2.0, baseline, px, Rgba::rgb(236, 238, 242), None);
        }
        if let (Ok(face), Some(b)) = (f.face(400.0), info.battery.as_ref()) {
            // The level is always said, and while a cable is in, what it is
            // taking: 85% · 充电中 · 18W.
            let percent = format!("{}%", b.level);
            let note = match b.charging() {
                true => match b.watts() {
                    Some(watts) => format!("{percent} · {} · {watts:.0}W", crate::tr!(self.state.lang, "充电中", "充電中", "Charging")),
                    None => format!("{percent} · {}", crate::tr!(self.state.lang, "充电中", "充電中", "Charging")),
                },
                false => match b.watts() {
                    Some(watts) => format!("{percent} · {watts:.0}W"),
                    None => percent,
                },
            };
            let px = m.dp(18.0);
            let width = out.text_width(&face, &note, px);
            out.text(&face, &note, (w - width) / 2.0, h * 0.375, px, Rgba::rgb(150, 154, 164), None);
        }
        let track = self.lock_track();
        let (label, _) = crate::pages::confirm_label(Confirm::Unlock, &self.state);
        // Drawn in the rect `lock_track` gives, which is also where the finger
        // has to land: a control across the whole page is a bar, not a slide.
        self.lock_rect = Some(widgets::confirm_in(out, &t, f, track, label, false, drag));
    }

    /// The charging screen, `level` of the way over the page drawn on `out`.
    fn compose_charging(&mut self, out: &mut Canvas, f: &Fonts, info: &DeviceInfo, level: f32, now: u64) {
        let battery = info.battery.as_ref().expect("a cable went in on a phone whose battery is read");
        let (t, lang) = (self.theme(), self.state.lang);
        if !self.charge_art.as_ref().is_some_and(|a| a.shows(lang, battery)) {
            self.charge_art = Some(charge::Art::new(f, &t.m, lang, battery).expect("the screens' face takes the charging screen's weight"));
        }
        let since = self.charging.as_ref().map_or(now, |c| c.since);
        let dither = self.dither.get_or_insert_with(|| glow::Dither::new(out.w as usize, out.h as usize));
        let art = self.charge_art.as_ref().expect("drawn above");
        if level >= 1.0 {
            charge::draw(out, art, &t.m, battery.level, now.saturating_sub(since), dither);
        } else {
            let screen = self.charge_canvas.get_or_insert_with(|| Canvas::new(out.w, out.h));
            charge::draw(screen, art, &t.m, battery.level, now.saturating_sub(since), dither);
            out.lay(screen, 0, 0, level);
        }
    }

    /// The visible part of the page, pulled past an end if it is, its large
    /// title fading as it scrolls, the bar and its title over a scrolled
    /// page, the `<`, the page sliding in or out over or under the page
    /// left, the status bar, any message, and the back cue while a swipe
    /// from an edge is under way.
    fn compose_page(&mut self, out: &mut Canvas, f: &Fonts, info: &DeviceInfo, clock: &str, now: u64) {
        // A page opened is drawn from its left edge as it slides in.
        let opening = self.turn.as_ref().is_some_and(|t| !t.back);
        self.render(f, info, if opening { 0 } else { self.w });
        let screen = self.screen();
        let scroll = self.top().1;
        let offset = (scroll + self.overscroll(now)).round();
        let caret_on = match self.frame.as_ref().is_some_and(|fr| fr.caret.is_some()) {
            true => self.caret_up(screen, now),
            false => {
                self.blink = None;
                self.caret_shown = false;
                false
            }
        };
        if let Some(m) = &self.morph {
            if !m.at(now).1 {
                return self.compose_morph(out, f, info, clock, now, caret_on);
            }
            self.morph = None;
        }
        let t = self.theme();
        if let Some((which, track)) = self.knob.take() {
            // 锁屏的滑轨不在页面上：它随锁屏一起画，所以按住电源键滑它时要
            // 重画的是整块锁屏，不是页面上的那一段。
            if which == Confirm::Unlock {
                self.lock_rect = None;
            } else {
                self.paint_knob(&t, f, which, track);
            }
        }
        if self.toast.as_ref().is_some_and(|m| now >= m.1) {
            self.toast = None;
        }
        let title = self.frame.as_ref().expect("rendered above").title;
        let key = (title, self.state.dark);
        if self.titles.as_ref().is_none_or(|(k, _, _)| *k != key) {
            self.titles = Some((key, Blurs::new(widgets::big_title(&t, f, title)), Blurs::new(widgets::small_title(&t, f, title))));
        }
        if let Some(fold) = self.frame.as_ref().and_then(|fr| fr.fold) {
            let key = (fold.hint, self.state.dark);
            if self.search_words.as_ref().is_none_or(|(k, _)| *k != key) {
                self.search_words = Some((key, widgets::search_words(&t, f, fold.bar, fold.hint)));
            }
        }
        if self.chip.as_ref().is_none_or(|c| c.0 != t.dark) {
            let (art, x, y) = widgets::back_chip(&t);
            self.chip = Some((t.dark, art, x, y));
        }
        let button = self.frame.as_ref().is_some_and(|fr| fr.button.is_some());
        if button && self.button_chip.as_ref().is_none_or(|c| c.0 != t.dark) {
            let (art, x, y) = widgets::button_chip(&t);
            self.button_chip = Some((t.dark, art, x, y));
        }
        let slide = self.slid(now);
        if self.frame.as_ref().is_some_and(|fr| fr.hero.is_none()) {
            // What of the page is on screen, drawn if it has not been yet.
            // While it slides in, that is its columns come in, and a
            // quarter of the screen more each frame, so it is drawn across
            // early in the slide; all of it once a row of it is held.
            let wide = match (slide, &self.turn) {
                (Some((at, _, _)), Some(turn)) if !turn.back && self.row_press.is_none() => (self.w - at.clamp(0, self.w as i32) as u32).max(self.cols + self.w / 4),
                _ => self.w,
            };
            self.draw_cols(f, info, wide);
            self.draw_rows(f, info, offset, offset + self.h as f32);
        }
        // The row a finger holds, drawn once more as held down when it
        // starts darkening, and only that row: a finger that moves on
        // before then never waits for it.
        if let (Some(press), None) = (self.row_press.filter(|p| now >= p.rise), &self.row_art) {
            let page = &self.frame.as_ref().expect("rendered above").canvas;
            let (x, y, w, h) = row_part(press.rect, page);
            // A row pressed where the page has nothing drawn has no part to
            // take: the row under the finger is off the rows in view.
            if w > 0 && h > 0 {
                let pressed = t.pressed();
                let band = Band { rows: (y, y + h), cols: (x, x + w) };
                let cx = Ctx { t: &pressed, f, info, state: &self.state, term: self.term.as_ref(), band: Some(band) };
                let down = pages::render(screen, &cx).canvas;
                assert_eq!((down.w, down.h), (page.w, page.h), "a page held down is laid out as it was");
                self.row_art = Some((page.crop(x, y, w, h), down.crop(x, y, w, h)));
            }
        }
        let view = View {
            scroll,
            offset,
            bar: self.bar.level(now),
            title: self.title.level(now),
            light: self.light(offset),
            caret: caret_on,
            row: self.row_press.map_or(0.0, |p| p.level(now)),
            pressing: self.pressing(),
        };
        match slide {
            Some((at, shift, dim)) => {
                // The page shown is drawn on its own canvas, once for as
                // long as it shows the same, and slid from there with the
                // page left.
                // A page opened shows nothing of itself until its edge comes
                // in from the right: the page left alone shows.
                let back = self.turn.as_ref().expect("sliding").back;
                if !back && at >= self.w as i32 {
                    let left = &self.turn.as_ref().expect("sliding").left;
                    out.slide(left, left, at, shift, dim, self.corners);
                } else {
                    let still = self.frame.as_ref().is_some_and(|fr| fr.hero.is_none());
                    let page = match self.layer.take() {
                        Some((shown, c)) if still && shown == view => c,
                        kept => {
                            let mut c = kept.map(|(_, c)| c).unwrap_or_else(|| Canvas::new(out.w, out.h));
                            self.paint_page(&mut c, &t, Some((f, info)), &view);
                            c
                        }
                    };
                    let turn = self.turn.as_ref().expect("sliding");
                    let (front, behind) = if back { (&turn.left, &page) } else { (&page, &turn.left) };
                    out.slide(front, behind, at, shift, dim, self.corners);
                    self.layer = Some((view, page));
                }
            }
            None => {
                self.layer = None;
                self.paint_page(out, &t, Some((f, info)), &view);
            }
        }
        self.shown = Some(view);

        widgets::status_bar(out, &t, f, &self.bar(clock, info), t.pal.status);
        if let Some((text, _)) = &self.toast {
            widgets::toast(out, &t, f, text);
        }
        match self.drag {
            Drag::Edge { start, from_left, armed, x, cy } => {
                let travel = if from_left { x - start } else { start - x };
                widgets::back_cue(out, &t, from_left, cy, widgets::back_cue_depth(&t, travel), armed);
            }
            _ => {
                if let Some((from_left, cy, s)) = self.cue {
                    widgets::back_cue(out, &t, from_left, cy, s.at(now).0.max(0.0), false);
                }
            }
        }
    }

    /// Draw the page shown as `view` has it: all of it but the status bar,
    /// any message and the back cue, which lie over every page. Rows of its
    /// long list come into view are drawn with `fresh`; a page drawn again
    /// as it last showed has them drawn already.
    pub(super) fn paint_page(&mut self, out: &mut Canvas, t: &Theme, fresh: Option<(&Fonts, &DeviceInfo)>, view: &View) {
        let m = t.m;
        // The chips beside the title are placed as the page is drawn, so the
        // places from the last drawing are of no use.
        self.bar_chip_rects.clear();
        let screen = self.screen();
        let (scroll, offset, pressing) = (view.scroll, view.offset, view.pressing);
        // The light goes over its control, in the part of the screen the
        // control is drawn in, under whatever is laid over that part.
        let shine = |out: &mut Canvas, part: Group| {
            if let Some((g, r, radius, tint)) = view.light
                && g == part
            {
                out.fill_round_rect(r.x, r.y, r.w, r.h, radius, tint);
            }
        };
        let frame = self.frame.as_ref().expect("a page is drawn from its frame");
        let (_, big, small) = self.titles.as_mut().expect("a page's titles are drawn with it");
        // The row a finger holds darkens as part of the page, under what is
        // laid over the page.
        let press_row = |out: &mut Canvas| {
            if let (Some(press), Some((was, is))) = (self.row_press, &self.row_art) {
                let (x, y, _, _) = row_part(press.rect, &frame.canvas);
                out.relay(was, is, x as i32, y as i32 - offset as i32, view.row);
            }
        };

        if let Some(art) = &frame.hero {
            // The field fills the screen and fades as the page scrolls; the
            // cards scroll over it. The version and the author's line fade
            // and shrink away first, then the wordmark.
            let glow = self.glow.as_ref().expect("the About page keeps its field");
            let fade = (1.0 - scroll / m.dp(ABOUT_FIELD_DP)).clamp(0.0, 1.0);
            let top = (widgets::bar_bottom(t) + m.dp(ABOUT_FIELD_DP)) / m.h as f32;
            let field = glow::sample(glow, out.w as usize, out.h as usize, FIELD_CELL, top, t.pal.bg, fade);
            let dither = self.dither.get_or_insert_with(|| glow::Dither::new(out.w as usize, out.h as usize));
            glow::paint(out, &field, fade, &glow::Layer { canvas: &frame.canvas, runs: &art.runs, offset: offset as isize }, dither);
            press_row(out);
            let range = m.dp(ABOUT_LOGO_DP) - widgets::bar_bottom(t);
            let v = (scroll / range).clamp(0.0, 1.0);
            art.version.draw_scaled(out, 0.0, 0.0, 1.0 - 0.1 * v, t.hero.version, 1.0 - v);
            art.author.draw_scaled(out, 0.0, 0.0, 1.0 - 0.1 * v, t.hero.version, 1.0 - v);
            let l = ((scroll - range) / m.dp(ABOUT_LOGO_FADE_DP)).clamp(0.0, 1.0);
            glow::ink_wordmark(out, &field, &glow.look().ink, &art.wordmark, 1.0 - 0.1 * l, 1.0 - l, fade, dither);
        } else {
            // The page rows on screen above a long list's rows, those rows,
            // and the page colour past the ends; every pixel is written
            // once.
            let h = out.h as f32;
            let span = frame.rows.as_ref().map(|r| ((r.edges[0] - offset).clamp(0.0, h), (r.bottom() - offset).clamp(0.0, h)));
            let (list_top, list_end) = span.unwrap_or((h, h));
            let first = (-offset).clamp(0.0, h) as usize;
            let last = (frame.canvas.h as f32 - offset).clamp(first as f32, list_top) as usize;
            if first < last {
                let from = offset as i32 + first as i32;
                out.put_rows(&frame.canvas, from, from + (last - first) as i32, first as i32);
            }
            out.fill_rect(0.0, 0.0, out.w as f32, first as f32, frame.bg);
            out.fill_rect(0.0, last as f32, out.w as f32, list_top - last as f32, frame.bg);
            let below = list_end.max(last as f32);
            out.fill_rect(0.0, below, out.w as f32, h - below, frame.bg);
            if let Some(rows) = &frame.rows {
                // A long list's rows on screen, each drawn when it first
                // shows. Those more than half a screen away are let go, and
                // their canvases drawn over again for the rows coming in, as
                // a list view recycles its rows.
                let near = rows.within(offset - h * NEAR_SCREENS, offset + h * (1.0 + NEAR_SCREENS));
                self.spare.extend(self.drawn.extract_if(.., |(i, _)| !near.contains(i)).map(|(_, c)| c));
                for i in rows.within(offset, offset + h) {
                    let k = match self.drawn.iter().position(|(j, _)| *j == i) {
                        Some(k) => k,
                        None => {
                            let (f, info) = fresh.expect("a page drawn again as it last showed has its rows in view drawn");
                            let mut c = self.spare.pop().unwrap_or_else(|| Canvas::new(out.w, 0));
                            c.resize((rows.edges[i + 1] - rows.edges[i]) as u32);
                            let cx = Ctx { t, f, info, state: &self.state, term: self.term.as_ref(), band: None };
                            pages::row(screen, &cx, i, &mut c);
                            self.drawn.push((i, c));
                            self.drawn.len() - 1
                        }
                    };
                    out.put(&self.drawn[k].1, (rows.edges[i] - offset) as i32);
                }
            } else {
                // Off a long list, its rows' canvases are not kept.
                self.spare.clear();
            }
            press_row(out);
            if let Some(fold) = &frame.fold {
                let (_, words) = self.search_words.as_ref().expect("a folding search bar's words are made with its page");
                fold.draw(out, t, words, offset);
            }
            if frame.header {
                let level = (1.0 - frame.title_lift(scroll) / m.dp(TITLE_FADE_DP)).clamp(0.0, 1.0);
                draw_faded(out, big, 0.0, -frame.title_lift(offset), t.pal.text, level, (1.0 - level) * m.dp(TITLE_BLUR_DP));
                // Art that sits beside the title is drawn over the page rather
                // than in it, so its touches are answered from here: the page's
                // own hits are where the art is not.
                if let Some((art, x, y)) = &frame.title_art {
                    let lift = frame.title_lift(offset);
                    out.lay(art, *x as i32, (*y - lift) as i32, level);
                    for hit in &frame.bar_hits {
                        self.bar_chip_rects.push((
                            Rect { x: hit.rect.x + *x, y: hit.rect.y + *y - lift, ..hit.rect },
                            hit.action,
                        ));
                    }
                }
            }
        }
        let bar = view.bar;
        let strip = frame.pinned.as_ref().map(|p| (p, pinned_top(t, p, offset)));
        shine(out, Group::Page);
        if bar > 0.0 {
            match strip {
                // The strip under the bar is veiled with it, and the list
                // thins out below the strip.
                Some((p, top)) => widgets::strip_mask(out, t, frame.bg, bar, top.round(), top.round() + p.canvas.h as f32),
                None => {
                    let stops = if frame.hero.is_some() { &widgets::ABOUT_BAR_MASK[..] } else { &widgets::BAR_MASK[..] };
                    widgets::bar_mask(out, t, stops, frame.bg, bar);
                }
            }
        }
        if let Some((p, top)) = strip {
            out.put(&p.canvas, top.round() as i32);
            shine(out, Group::Strip);
        }
        // What is held down on the strip, and the strip's cursor, go over it:
        // they are drawn after the strip or the strip would cover them.
        if let Some(a) = pressing.filter(|a| matches!(a, Action::CloseSearch | Action::Open(Screen::About))) {
            // The search page's button is on the strip held at the top, in the
            // strip's own coordinates; the home page's bar is on the page.
            let on_strip = frame.pinned.as_ref().and_then(|p| p.hits.iter().find(|h| h.action == a).map(|h| (h.rect, p)));
            match on_strip {
                Some((r, p)) => {
                    let top = pinned_top(t, p, offset);
                    widgets::cancel_held(out, t, Rect::new(r.x, r.y + top, r.w, r.h));
                }
                None => {
                    let r = frame.hits.iter().find(|h| h.action == a).expect("a button held down is on its page").rect;
                    match frame.shown(r, offset) {
                        Some(r) if a == Action::CloseSearch => widgets::cancel_held(out, t, r),
                        Some(r) => widgets::search_bar_held(out, t, r),
                        None => {}
                    }
                }
            }
        }
        // A cursor in a held strip stays with it: the field does not go up
        // with the page.
        if let Some(caret) = frame.caret.as_ref().filter(|_| view.caret) {
            // The cursor is in the strip's own coordinates, so it goes where
            // the strip shows: the strip's top on screen plus its own offset.
            let y = match strip {
                Some((_, top)) => caret.y + top.round() as i32,
                None => caret.y - offset as i32,
            };
            out.lay(&caret.art, caret.x, y, 1.0);
        }
        if let Some(d) = &frame.dock {
            out.put(&d.canvas, (out.h - d.canvas.h) as i32);
            shine(out, Group::Dock);
        }
        if frame.back {
            let (_, chip, x, y) = self.chip.as_ref().expect("a page's `<` is drawn with it");
            out.lay(chip, *x, *y, bar);
            if pressing == Some(Action::Back) {
                widgets::back_held(out, t);
            } else {
                widgets::back_arrow(out, t, t.pal.text);
            }
        }
        if let Some((glyph, action)) = frame.button {
            let (_, chip, x, y) = self.button_chip.as_ref().expect("a page's round button is drawn with its chip");
            out.lay(chip, *x, *y, bar);
            if pressing == Some(action) {
                widgets::button_held(out, t, glyph);
            } else {
                widgets::button_glyph(out, t, glyph, t.pal.text);
            }
        }
        if frame.back || frame.button.is_some() {
            shine(out, Group::Fixed);
        }
        let level = view.title;
        let blur = if frame.hero.is_some() { 0.0 } else { (1.0 - level) * m.dp(TITLE_BLUR_DP) };
        draw_faded(out, small, 0.0, 0.0, t.pal.text, level, blur);
    }

    /// Where a page sliding in over the page left, or the page left
    /// sliding off, stands at `now`: how many pixels from the left edge the
    /// page in front stands, how many left of where it rests the page
    /// behind, and how dark that is; nothing once the slide is over. See
    /// [`Turn`].
    fn slid(&mut self, now: u64) -> Option<(i32, i32, f32)> {
        let turn = self.turn.as_mut()?;
        let spring = *turn.spring.get_or_insert_with(|| if turn.back { Spring::new(now, -1000.0, 0.0, BACK_PERIOD) } else { Spring::new(now, 1000.0, -OPEN_SPEED * 1000.0, OPEN_PERIOD) });
        let (x, done) = spring.at(now);
        if done {
            self.backdrop = self.turn.take().map(|t| t.left);
            return None;
        }
        // Where the page in front stands, in screen widths from the left
        // edge.
        let along = if turn.back { 1.0 + x / 1000.0 } else { x / 1000.0 };
        let w = self.w as f32;
        Some(((along * w).round() as i32, (BEHIND_SHIFT * w * (1.0 - along)).round() as i32, BEHIND_DIM * (1.0 - along)))
    }

    /// Whether the cursor of the text field on `screen` shows at `now`: it
    /// comes up whole on each page it shows on and at each edit, then
    /// blinks.
    fn caret_up(&mut self, screen: Screen, now: u64) -> bool {
        let key = (screen, self.edits);
        if self.blink.is_none_or(|(s, e, _)| (s, e) != key) {
            self.blink = Some((screen, self.edits, now));
        }
        self.caret_shown = self.blink_at(now).is_some_and(|(on, _)| on);
        self.caret_shown
    }

    /// Draw the search bar on its way between the home page and the search
    /// page; see [`Morph`].
    fn compose_morph(&mut self, out: &mut Canvas, f: &Fonts, info: &DeviceInfo, clock: &str, now: u64, caret_on: bool) {
        let t = self.theme();
        let m = t.m;
        let mut morph = self.morph.take().expect("composing it");
        let (q, _) = morph.at(now);
        let (field, _) = widgets::search_layout(&t);
        let home = Rect::new(m.margin, m.content_top - morph.scroll, m.w as f32 - m.margin * 2.0, m.search_h);
        let along = |a: f32, b: f32| a + (b - a) * q;
        let bar = Rect::new(m.margin, along(home.y, field.y), along(home.w, field.w), m.search_h);
        let rise = bar.y - home.y;
        if !morph.into {
            // The home page's rows coming down with its bar.
            self.draw_rows(f, info, morph.scroll - rise, morph.scroll - rise + m.h as f32);
        }
        let frame = self.frame.as_ref().expect("rendered above");
        out.fill_rect(0.0, 0.0, out.w as f32, out.h as f32, frame.bg);
        if morph.into {
            // The search page's keyboard and what it shows under its
            // field, from its frame; that comes in once the bar is nearly
            // there.
            morph.keyboard.get_or_insert_with(|| keyboard_strip(&t, frame));
            let (body, top) = morph.body.get_or_insert_with(|| {
                let field_bottom = (field.y + field.h).ceil() as i32;
                // What the page shows under the field, down to the last row it
                // holds: the keys are no longer on the page, and the page is
                // longer than the screen, so only the rows it has drawn are
                // there to take.
                let held = frame.canvas.held();
                let start = field_bottom.max(held.start as i32);
                let end = (frame.canvas.h as i32).min(held.end as i32);
                let mut body = Canvas::new(frame.canvas.w, (end - start).max(0) as u32);
                body.put_rows(&frame.canvas, start, end, 0);
                (body, start)
            });
            let (delay, span) = MORPH_BODY_MS;
            out.lay(body, 0, *top, (now.saturating_sub(morph.start + delay) as f32 / span as f32).min(1.0));
        } else {
            // The home page under its bar comes down with it.
            let below = m.content_top + m.search_h;
            let held = frame.canvas.held();
            let from = (below.floor() as i32).max(held.start as i32);
            let to = (frame.canvas.h as i32).min(held.end as i32);
            out.put_rows(&frame.canvas, from, to, (below - morph.scroll + rise).floor() as i32);
        }
        let title = morph.title.get_or_insert_with(|| Blurs::new(widgets::big_title(&t, f, pages::HOME_TITLE)));
        let level = (1.0 - q / MORPH_TITLE_GONE).clamp(0.0, 1.0);
        draw_faded(out, title, 0.0, rise - morph.scroll, t.pal.text, level, (1.0 - level) * m.dp(TITLE_BLUR_DP));
        if let Some((keys, top)) = &morph.keyboard {
            out.put(keys, (*top as f32 + (1.0 - q) * keys.h as f32).round() as i32);
        }
        widgets::search_capsule(out, &t, f, bar, self.state.lang.pick(pages::SEARCH_HINT));
        // The cursor shows from the start, going up with the field. It stands
        // in the field's own band, so it goes where the bar has got to.
        if let Some(caret) = frame.caret.as_ref().filter(|_| morph.into && caret_on) {
            out.lay(&caret.art, caret.x, bar.y.round() as i32 + caret.y, 1.0);
        }
        let (disc, cross) = morph.cancel.get_or_insert_with(|| {
            let (disc, cross) = widgets::cancel_art(&t);
            (Blurs::new(disc), Blurs::new(cross))
        });
        let shown = ((q - MORPH_CANCEL.0) / (MORPH_CANCEL.1 - MORPH_CANCEL.0)).clamp(0.0, 1.0);
        let blur = (1.0 - shown) * m.dp(MORPH_CANCEL_BLUR_DP);
        draw_faded(out, disc, 0.0, 0.0, t.pal.search, shown, blur);
        draw_faded(out, cross, 0.0, 0.0, t.pal.text, shown, blur);
        self.morph = Some(morph);
        widgets::status_bar(out, &t, f, &self.bar(clock, info), t.pal.status);
        if let Some((text, _)) = &self.toast {
            widgets::toast(out, &t, f, text);
        }
    }

    /// Put the page back where it rests, for a page just opened or returned
    /// to; its first frame shows the bar as its scroll has it.
    pub(super) fn settle_page(&mut self) {
        self.height = 0.0;
        self.fling = None;
        self.glide = None;
        self.pull = 0.0;
        self.bounce = None;
        self.seek = None;
        self.adjusting = false;
        self.titles = None;
        self.glow = None;
        self.row_press = None;
        self.row_art = None;
        let hero = self.screen() == Screen::About;
        self.title = title_fade(hero);
        self.bar = bar_fade(hero);
        self.snap = true;
    }

    /// Show the user what the runtime has to say about work it was asked
    /// to do.
    pub fn report(&mut self, text: &str, now: u64) {
        self.say(text, now);
    }

    /// The screenshot [`Effect::Screenshot`] asked for went where `saved`
    /// says, or did not, as the system says why.
    pub fn screenshot_saved(&mut self, saved: Result<Shot, String>, now: u64) {
        let l = self.state.lang;
        let text = match saved {
            Ok(Shot::Album) => l.pick(fmt::SCREENSHOT_SAVED).to_string(),
            Ok(Shot::Held) => l.pick(fmt::SCREENSHOT_HELD).to_string(),
            Err(why) => fmt::screenshot_failed(l, &why),
        };
        self.say(&text, now);
    }

    /// The screenshots held while the owner's storage was closed did not go
    /// into the gallery's once it opened, as the system says why.
    pub fn screenshots_not_kept(&mut self, why: &str, now: u64) {
        self.say(&fmt::screenshots_not_kept(self.state.lang, why), now);
    }

    /// File transfer could not be turned on or off as the switch says: it
    /// shows how it stands, and why.
    pub fn file_transfer_failed(&mut self, on: bool, why: &str, now: u64) {
        self.state.file_transfer = on;
        self.refresh();
        self.say(why, now);
    }

    pub(super) fn say(&mut self, text: &str, now: u64) {
        self.toast = Some((text.to_string(), now + TOAST_MS));
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::testing::*;
    use skyward_info::Charge;

    #[test]
    fn a_page_drawn_a_band_at_a_time_is_the_page_drawn_whole() {
        let (f, i) = (fonts(), info());
        let state = State::default();
        // The About page lies over its field and is drawn whole.
        let screens = [
            Screen::Decrypt,
            Screen::Home,
            Screen::Settings,
            Screen::Reboot,
            Screen::Wipe,
            Screen::Mount,
            Screen::Install,
            Screen::Choose(Place::Storage),
            Screen::Specs,
            Screen::ScreenOff,
            Screen::Search,
            Screen::Backup,
            Screen::Restore,
            Screen::Advanced,
            Screen::Files,
            Screen::Terminal,
            Screen::Language,
            Screen::TimeZone,
            Screen::TimeZones(Region::Asia),
            Screen::Logs,
            Screen::Update,
            Screen::SkywardUpdate,
            Screen::License,
            Screen::FontLicense,
            Screen::Slot,
            Screen::Fastboot,
            Screen::Erase,
            Screen::CantStart,
            Screen::Unhandled,
        ];
        for t in [Theme::light(W, H, PPD), Theme::dark(W, H, PPD).pressed()] {
            for screen in screens {
                let cx = |band| Ctx { t: &t, f: &f, info: &i, state: &state, term: None, band };
                let whole = pages::render(screen, &cx(None)).canvas;
                let row = whole.w as usize * 4;
                let differs = |c: &Canvas| c.data.chunks(row).zip(whole.data.chunks(row)).position(|(a, b)| a != b);
                // Cuts through cards, text and icons, and a band one row
                // tall, the bands joined one under another.
                let mut cuts: Vec<u32> = [0, 1, 377, 1201, 3000, 3001, 4444, u32::MAX].map(|k| k.min(whole.h)).to_vec();
                cuts.dedup();
                let mut joined: Option<Canvas> = None;
                for rows in cuts.windows(2) {
                    let part = pages::render(screen, &cx(Some(Band { rows: (rows[0], rows[1]), cols: (0, W) }))).canvas;
                    assert_eq!((part.h, part.held()), (whole.h, rows[0]..rows[1]), "{screen:?}: a band is laid out as the whole");
                    match &mut joined {
                        Some(c) => c.join(&part),
                        None => joined = Some(part),
                    }
                }
                let joined = joined.expect("a page has rows");
                assert!(joined.whole(), "{screen:?}: the bands make the whole");
                assert_eq!(differs(&joined), None, "{screen:?}: the first row drawn otherwise in bands");
                // The page drawn across a part at a time, as one sliding
                // in, into a canvas holding all of it: cuts through cards
                // and text, and a part one column wide.
                let mut across = Canvas::unfilled(W, whole.h, 0..whole.h);
                for cols in [0, 1, 333, 720, W - 1, W].windows(2) {
                    let part = pages::render(screen, &cx(Some(Band { rows: (0, whole.h), cols: (cols[0], cols[1]) }))).canvas;
                    assert_eq!(part.cols(), cols[0]..cols[1], "{screen:?}: a part across holds its columns");
                    across.replace(&part);
                }
                assert_eq!(differs(&across), None, "{screen:?}: the first row drawn otherwise across");
            }
        }
    }

    #[test]
    fn a_page_at_rest_draws_the_rows_below_the_screen_a_band_at_a_time() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Settings, W, H, PPD);
        app.compose(&mut Canvas::new(W, H), &f, &i, "12:00", 0);
        assert_eq!(app.frame.as_ref().unwrap().canvas.held(), 0..H, "the page shown has the rows on screen drawn");
        let mut bands = 1;
        while app.draw_ahead(&f, &i) {
            bands += 1;
        }
        let t = app.theme();
        let whole = pages::render(Screen::Settings, &Ctx { t: &t, f: &f, info: &i, state: &app.state, term: None, band: None }).canvas;
        let page = &app.frame.as_ref().unwrap().canvas;
        assert!(whole.h > H, "a page longer than the screen, drawn in {bands} more bands");
        assert!(page.whole() && page.data == whole.data, "the page drawn a band at a time is the page drawn whole");
    }

    #[test]
    fn the_search_page_comes_up_from_the_home_page_bar_whole() {
        // The bar on the home page lifts the search page up over it. The
        // search page is longer than the screen and holds its keyboard as a
        // dock, so what it shows under the field is only as much of it as
        // has been drawn: taking the rest of it is a panic, and a panic here
        // is the screen going black mid-gesture.
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        let mut out = Canvas::new(W, H);
        app.compose(&mut out, &f, &i, "12:00", 0);
        // The bar on the home page, touched where it stands.
        let bar = app.frame.as_ref().and_then(|fr| fr.fold.map(|fd| fd.bar)).expect("the home page has its bar");
        let (x, y) = bar.centre();
        app.touch(Touch::Down(x, y), &f, 1000);
        app.touch(Touch::Up, &f, 1000);
        assert_eq!(app.screen(), Screen::Search, "the search page is the page shown");
        // Every frame of the way up, the page as far along as it has come.
        for now in [1000, 1050, 1150, 1300, 1600] {
            app.compose(&mut out, &f, &i, "12:00", now);
        }
        let frame = app.frame(&f, &i);
        assert!(frame.dock.is_some(), "the keys are held at the foot of the screen");
        assert!(frame.canvas.h > H, "and the page runs on past the screen");
        assert!(frame.canvas.whole() || !frame.canvas.held().is_empty(), "with its rows drawn");
    }

    #[test]
    fn a_row_pressed_past_the_rows_in_view_is_left_alone() {
        // A page longer than the screen holds only the rows in view. A finger
        // held on a row that has been scrolled past has nothing to take, and
        // taking it anyway read past the rows that were drawn.
        let page = Canvas::band(1440, 19200, 3200..6400, 0..1440, Rgba::rgb(0, 0, 0));
        // A row well past the rows held: no rows of it are there to take.
        let (_, _, _, h) = row_part(Rect::new(0.0, 12000.0, 1440.0, 250.0), &page);
        assert_eq!(h, 0, "a row off the rows drawn has no part");
        // A row that is there is taken whole.
        let (_, y, w, h) = row_part(Rect::new(0.0, 4000.0, 1440.0, 250.0), &page);
        assert_eq!((y, w, h), (4000, 1440, 250), "a row in view is taken whole");
        // One that runs off the bottom is cut at the last row held.
        let (_, y, _, h) = row_part(Rect::new(0.0, 6300.0, 1440.0, 250.0), &page);
        assert_eq!((y, h), (6300, 100), "a row running off the end stops there");
    }

    #[test]
    fn a_page_typed_on_draws_again_only_what_changed_and_shows_what_it_would_drawn_anew() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Search, W, H, PPD);
        let mut out = Canvas::new(W, H);
        app.compose(&mut out, &f, &i, "12:00", 0);
        let anew = |app: &App| {
            let t = app.theme();
            // The rows the page holds, drawn anew: the search page is longer
            // than the screen and draws the rest as it scrolls, so the part
            // drawn again is what is compared.
            let band = app.frame.as_ref().map(|fr| {
                let held = fr.canvas.held();
                Band { rows: (held.start, held.end), cols: (0, W) }
            });
            pages::render(Screen::Search, &Ctx { t: &t, f: &f, info: &i, state: &app.state, term: None, band })
        };
        for cap in [Cap::Char('b'), Cap::Char('f'), Cap::Space, Cap::Backspace, Cap::Shift, Cap::Char('q')] {
            let (x, y) = spot(&mut app, &f, &i, Action::Type(cap));
            app.touch(Touch::Down(x, y), &f, 0);
            assert!(app.frame.is_some() && !app.stale.is_empty() || cap == Cap::Shift, "{cap:?}: the key held is drawn again on its own");
            app.compose(&mut out, &f, &i, "12:00", 0);
            assert!(app.frame.as_ref().unwrap().canvas.data == anew(&app).canvas.data, "{cap:?} held down");
            app.touch(Touch::Up, &f, 0);
            app.compose(&mut out, &f, &i, "12:00", 0);
            let (drawn, fresh) = (app.frame.as_ref().unwrap(), anew(&app));
            assert!(drawn.canvas.data == fresh.canvas.data, "{cap:?} typed");
            assert_eq!(drawn.hits.iter().map(|h| h.action).collect::<Vec<_>>(), fresh.hits.iter().map(|h| h.action).collect::<Vec<_>>(), "{cap:?} typed");
        }
        assert_eq!(app.state.search, "bfQ");
    }

    #[test]
    fn the_control_centre_lays_out_the_tiles_it_holds_and_every_one_answers_a_touch() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        let mut c = Canvas::new(W, H);
        app.shade = 1.0;
        app.compose(&mut c, &f, &i, "12:00", 1000);
        // The twelve tiles and the two capsules, and nothing else.
        assert_eq!(app.shade_hits.len(), 14, "eleven tiles, the pencil and two capsules");
        let tiles: Vec<ShadeHit> = app
            .shade_hits
            .iter()
            .map(|(_, h)| *h)
            .filter(|h| matches!(h, ShadeHit::Tile(_)))
            .collect();
        let wanted: Vec<ShadeHit> = app.state.tiles.iter().map(|&t| ShadeHit::Tile(t)).collect();
        assert_eq!(tiles, wanted, "the tiles in the order the state lays them out");
        // Every one of them sits inside the sheet, and no two overlap: a tile
        // a finger cannot reach, or one that takes another's touch, is a tile
        // that does nothing.
        let theme = app.theme();
        let sheet = shade_height(&theme, app.state.tiles.len(), app.shade_editing);
        for (k, (rect, hit)) in app.shade_hits.iter().enumerate() {
            assert!(rect.y >= 0.0 && rect.y + rect.h <= sheet, "{hit:?} runs past the sheet: {rect:?}");
            assert!(rect.x >= 0.0 && rect.x + rect.w <= W as f32, "{hit:?} runs off the side: {rect:?}");
            for (o, oh) in app.shade_hits.iter().skip(k + 1) {
                let apart = rect.x + rect.w <= o.x || o.x + o.w <= rect.x || rect.y + rect.h <= o.y || o.y + o.h <= rect.y;
                assert!(apart, "{hit:?} and {oh:?} overlap");
            }
        }
        // The rows are three, and the four of each sit on the same line.
        let ys: Vec<f32> = app.shade_hits.iter().filter(|(_, h)| matches!(h, ShadeHit::Tile(_))).map(|(r, _)| r.y).collect();
        assert_eq!(ys.chunks(4).count(), 3, "three rows of four");
        for row in ys.chunks(4) {
            assert!(row.windows(2).all(|w| (w[0] - w[1]).abs() < 0.01), "a row is level: {row:?}");
        }
        assert!(ys[0] < ys[4] && ys[4] < ys[8], "and the rows run down the sheet");
    }

    #[test]
    fn the_control_centre_lights_the_tiles_that_turn_something_on() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        let mut c = Canvas::new(W, H);
        app.shade = 1.0;
        // Nothing on: the WLAN is off, the motor still, file transfer off and
        // no drive plugged in.
        app.state.wlan = None;
        app.state.vibrate = false;
        app.state.file_transfer = false;
        app.state.usb_drive = None;
        app.compose(&mut c, &f, &i, "12:00", 1000);
        let lit = |app: &App| -> Vec<Tile> { TILES.into_iter().filter(|t| t.lit(&app.state)).collect() };
        assert!(lit(&app).is_empty(), "nothing is lit with everything off");
        // Turned on, each shows lit; the ones that open a page never do.
        app.state.wlan = Some(connected("Redmi-AX6000"));
        app.state.vibrate = true;
        app.state.file_transfer = true;
        app.state.usb_drive = Some(UsbDrive::Shown);
        app.compose(&mut c, &f, &i, "12:00", 1000);
        assert_eq!(lit(&app), vec![Tile::Wlan, Tile::Vibrate, Tile::FileTransfer]);
        assert!(!Tile::Files.lit(&app.state) && !Tile::Settings.lit(&app.state), "a page's tile is never lit");
    }

    #[test]
    fn a_name_too_long_for_its_tile_is_set_smaller_before_it_is_rolled() {
        let (f, i) = (fonts(), info());
        let t = Theme::light(W, H, PPD);
        let mut app = App::new(Screen::Home, W, H, PPD);
        let mut c = Canvas::new(W, H);
        // A name that fits at the tile's own size is not rolled. The tiles
        // are a quarter of the sheet wide, so the name has to be short.
        app.state.wlan = Some(connected("Mi"));
        app.shade = 1.0;
        app.compose(&mut c, &f, &i, "12:00", 1000);
        assert!(app.shade_roll.is_none(), "a name that fits is still");

        // One far too long for any size that fits: it rolls.
        let long = "一个很长很长的无线网络名称用来看看跑马灯";
        app.state.wlan = Some(connected(long));
        app.compose(&mut c, &f, &i, "12:00", 2000);
        let roll = app.shade_roll.expect("a name too long to set whole rolls");
        assert!(roll.travel > roll.room, "wider than the tile at its least size");
        assert_eq!(roll.since, 2000, "it sets off the first time it is drawn");
        // The least size is the floor, not smaller.
        let least = t.m.dp(SHADE_NAME_LEAST_DP);
        let room = roll.room;
        let at_least = c.text_width(&f.body, long, least);
        assert!(at_least > room, "the floor does not fit either, so it rolls");
    }

    #[test]
    fn a_rolling_name_keeps_its_place_and_asks_for_frames_only_while_it_moves() {
        let (f, i) = (fonts(), info());
        let mut app = App::new(Screen::Home, W, H, PPD);
        let mut c = Canvas::new(W, H);
        app.state.wlan = Some(connected("一个很长很长的无线网络名称用来看看跑马灯"));
        app.shade = 1.0;
        app.compose(&mut c, &f, &i, "12:00", 1000);
        let first = app.shade_roll.expect("rolling");
        // A frame later the name has not set off again.
        app.compose(&mut c, &f, &i, "12:00", 1040);
        assert_eq!(app.shade_roll.expect("still rolling").since, first.since, "it keeps its place");

        // Resting at its start, it asks for nothing: the sheet is drawn every
        // turn while it is out, and a frame a turn would copy the panel for a
        // name that is standing still.
        app.tick(first.since + 100);
        let _ = app.take_dirty();
        app.tick(first.since + 200);
        assert!(!app.take_dirty(), "at rest it does not have the panel copied");

        // On the move it does, every turn it moves.
        let moving = first.since + SHADE_ROLL_REST_MS + 100;
        app.tick(moving);
        assert!(app.take_dirty(), "a moving name has the panel copied");
        app.tick(moving + 100);
        assert!(app.take_dirty(), "and again while it is still moving");
    }

    #[test]
    fn a_rolled_name_never_leaves_its_tile() {
        // The offset stays between the name's start and its end, and comes
        // back: nothing of the round trip runs past the tile.
        let (travel, room) = (900.0, 400.0);
        let over = travel - room;
        for ms in (0..40_000).step_by(7) {
            let at = roll_offset(travel, room, 66.0, ms);
            assert!((0.0..=over).contains(&at), "{ms} ms: {at}");
        }
        // It rests at the start, then at the end, and comes back.
        assert_eq!(roll_offset(travel, room, 66.0, 0), 0.0);
        let roll = (over / 66.0 * 1000.0) as u64;
        assert!(roll_offset(travel, room, 66.0, SHADE_ROLL_REST_MS + roll) >= over - 0.01, "the end shows");
        assert!(roll_offset(travel, room, 66.0, SHADE_ROLL_REST_MS + roll + SHADE_ROLL_REST_MS + roll) <= 0.01, "and it comes back");
        // A name that fits does not move at all.
        assert_eq!(roll_offset(room, room, 66.0, 5000), 0.0);
        assert_eq!(roll_offset(room - 1.0, room, 66.0, 5000), 0.0);
    }

    #[test]
    fn the_home_card_tells_of_a_battery_too_low_to_install_on() {
        let f = fonts();
        // Where the first row under the card starts: the card is that much
        // taller with a notice in it.
        let under = |i: &DeviceInfo| {
            let mut app = App::new(Screen::Home, W, H, PPD);
            app.frame(&f, i).hits.iter().find(|h| h.action == Action::Open(Screen::Install)).expect("an install row").rect.y
        };
        let plain = under(&info());
        assert!(under(&DeviceInfo { battery: Some(battery_at(19, Charge::Discharging)), ..info() }) > plain, "short of 20% unplugged");
        assert_eq!(under(&DeviceInfo { battery: Some(battery_at(20, Charge::Discharging)), ..info() }), plain);
        assert!(under(&DeviceInfo { battery: Some(battery_at(14, Charge::Charging)), ..info() }) > plain, "short of 15% charging");
        assert_eq!(under(&DeviceInfo { battery: Some(battery_at(16, Charge::Charging)), ..info() }), plain);
        let (title, note) = fmt::battery_low(Lang::Hans, 14, true);
        assert_eq!((title.as_str(), note.as_str()), ("电量仅剩 14%", "充电至 15% 以上才能安装"));
    }

    #[test]
    fn the_home_card_tells_of_a_restart_into_the_other_slot_and_opens_the_slot_page() {
        let f = fonts();
        let i = DeviceInfo { slot: Some(Slot::B), ..info() };
        let mut app = App::new(Screen::Home, W, H, PPD);
        let slots = |active| BootSlots { active, bootable: [true, true], successful: [true, true] };
        app.set_boot_slots(Ok(slots(Slot::B)));
        assert!(app.frame(&f, &i).hits.iter().all(|h| h.action != Action::Open(Screen::Slot)), "the slot it runs from starts next");
        app.set_boot_slots(Ok(slots(Slot::A)));
        assert_eq!(tap(&mut app, &f, &i, Action::Open(Screen::Slot)), vec![Effect::ReadSlots]);
        assert_eq!(app.screen(), Screen::Slot);
        assert_eq!(fmt::next_boot(Lang::Hans, Slot::A, Slot::B), ("重启后从 A 槽位启动".into(), "当前运行的是 B 槽位".into()));
    }

    #[test]
    fn the_status_bar_carries_the_slot_the_data_partitions_lock_and_the_temperature() {
        let i = DeviceInfo { slot: Some(Slot::B), battery: Some(battery_at(64, Charge::Charging)), ..info() };
        let mut app = App::new(Screen::Home, W, H, PPD);
        let bar = app.bar("12:00", &i);
        assert_eq!((bar.slot, bar.temp, bar.battery), (Some("B"), Some(352), Some((64, Some(Bolt::Charging)))), "the temperature to the tenth");
        let quick = DeviceInfo { battery: Some(skyward_info::Battery { quick: true, ..battery_at(64, Charge::Charging) }), ..info() };
        assert_eq!(app.bar("12:00", &quick).battery, Some((64, Some(Bolt::Quick))), "a quick charger draws the stock bar's quick bolt");
        let out = DeviceInfo { battery: Some(battery_at(64, Charge::Discharging)), ..info() };
        assert_eq!(app.bar("12:00", &out).battery, Some((64, None)), "no bolt with the cable out");
        let lock = |app: &mut App, data: Data| {
            app.state.data = data;
            app.bar("12:00", &i).data
        };
        assert_eq!(lock(&mut app, Data::Locked { user: 0, keys: Keys::Digits, checking: false }), Some(false), "shut while its files are out of reach");
        assert_eq!(lock(&mut app, Data::Open), Some(true));
        assert_eq!(lock(&mut app, Data::Erased), None, "nothing to open once wiped");
        let none = app.bar("12:00", &info());
        assert_eq!((none.slot, none.temp), (None, None), "a phone without slots or a fuel gauge");
    }

    #[test]
    fn the_cursor_blinks_and_comes_up_whole_at_each_edit() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Search, &f, &i);
        let mut screen = Canvas::new(W, H);
        let mut shows = |app: &mut App, now: u64| {
            app.compose(&mut screen, &f, &i, "12:00", now);
            app.caret_shown
        };
        assert!(shows(&mut app, 1000), "it comes up whole on the page");
        assert_eq!(app.deadline(1200), Some(1500), "the loop wakes when it turns");
        assert!(!shows(&mut app, 1500), "half a second on it hides");
        assert!(shows(&mut app, 2000));
        assert!(!shows(&mut app, 2600));
        tap(&mut app, &f, &i, Action::Type(Cap::Char('b')));
        assert!(shows(&mut app, 2700), "an edit brings it back whole at once");
        assert!(!shows(&mut app, 3200));
        tap(&mut app, &f, &i, Action::CloseSearch);
        assert_eq!(app.screen(), Screen::Home, "the round button leaves the search");
        assert!(!shows(&mut app, 3300), "no cursor off the page");
        assert!(app.deadline(3300).is_none_or(|d| d > 10_000), "nor a wake for it, only for the screen going off");
    }
}
