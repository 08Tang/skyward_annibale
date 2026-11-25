//! What a control does once it is tapped, typed on or swiped.

use super::*;
use crate::tr;

impl App {
    /// Carry out a tapped control.
    pub(super) fn act(&mut self, action: Action, now: u64) -> Vec<Effect> {
        let l = self.state.lang;
        match action {
            Action::Open(s) => {
                // The served folder is chosen once: opening the page it is
                // served on ends the choosing, however it was got to.
                if s == Screen::Serve {
                    self.state.picking = false;
                }
                // The second system's place in Dual boot opens the install
                // page for a package that installs a system beside the
                // installed one, and says so: a package of any other kind is
                // refused there rather than started (2026-11-21).
                if s == Screen::Install {
                    self.state.second_system_asked = self.screen() == Screen::Systems;
                }
                if s == Screen::Search && self.screen() == Screen::Home {
                    // The bar on the home page looks through the whole of
                    // what the phone offers, whatever page the field was last
                    // opened from.
                    self.state.search_scope = SearchScope::Features;
                    // The bar moves up from where it stands unfolded: a
                    // page easing there for the keys gets there at once.
                    if let Some((rest, _)) = self.seek.take() {
                        self.top_mut().1 = rest;
                    }
                    self.start_morph(true, now);
                    self.push(s);
                } else {
                    self.open(s);
                }
                return self.arrive_at(s);
            }
            Action::OpenData => return self.open_data(),
            // The order the file manager shows entries in: the key cycles, the
            // side turns, and the folders stay on top or mix in.
            Action::CycleSort => {
                self.state.sort = match self.state.sort {
                    SortKey::Name => SortKey::Size,
                    SortKey::Size => SortKey::Modified,
                    SortKey::Modified => SortKey::Type,
                    SortKey::Type => SortKey::Name,
                };
                self.resort_files();
                return vec![Effect::KeepSort(self.state.sort, self.state.sort_desc, self.state.folders_first)];
            }
            Action::ToggleSortOrder => {
                self.state.sort_desc = !self.state.sort_desc;
                self.resort_files();
                return vec![Effect::KeepSort(self.state.sort, self.state.sort_desc, self.state.folders_first)];
            }
            Action::ToggleFoldersFirst => {
                self.state.folders_first = !self.state.folders_first;
                self.resort_files();
                return vec![Effect::KeepSort(self.state.sort, self.state.sort_desc, self.state.folders_first)];
            }
            // The order is chosen from a panel of ticks rather than cycled
            // through a button, since there are four keys and two ways and a
            // button makes the owner walk past the one they want.
            Action::SortPanel(on) => {
                self.state.sort_panel = on;
                self.refresh();
            }
            Action::SortTo(key) => {
                self.state.sort = key;
                self.state.sort_panel = false;
                self.resort_files();
                return vec![Effect::KeepSort(self.state.sort, self.state.sort_desc, self.state.folders_first)];
            }
            Action::SortDesc(desc) => {
                self.state.sort_desc = desc;
                self.state.sort_panel = false;
                self.resort_files();
                return vec![Effect::KeepSort(self.state.sort, self.state.sort_desc, self.state.folders_first)];
            }
            Action::FetchStart => {
                let url = self.state.fetch_url.trim().to_string();
                if url.is_empty() {
                    self.say(self.state.lang.pick(["请先输入网址", "請先輸入網址", "Type an address first"]), now);
                } else if self.fetching() {
                    self.say(self.state.lang.pick(["已有下载在进行", "已有下載在進行", "A download is under way"]), now);
                } else {
                    self.state.fetch = Fetch::Running { done: 0, total: None };
                    return vec![Effect::Fetch(url)];
                }
            }
            Action::FetchStop => {
                if self.fetching() {
                    return vec![Effect::StopFetch];
                }
            }
            // An address too long to type can be put in a file from a computer
            // and taken from there with one tap.
            Action::FetchFromFile => {
                let at = "/sdcard/Download/link.txt";
                match std::fs::read_to_string(at) {
                    Ok(text) => {
                        let url = text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or("").to_string();
                        let said = match url.is_empty() {
                            true => self.state.lang.pick(["文件里没有链接", "檔案裡沒有連結", "The file holds no link"]).to_string(),
                            false => {
                                self.state.fetch_url = url;
                                tr!(self.state.lang, "已从文件读入链接", "已從檔案讀入連結", "The link was taken from the file").to_string()
                            }
                        };
                        self.say(&said, now);
                    }
                    Err(e) => {
                        let said = format!("{at}: {e}");
                        self.say(&said, now);
                    }
                }
                self.dirty = true;
            }
            // The folder served is the file manager's own: the switch serves
            // it or stops serving it, and changing it opens the file manager,
            // which the page returns to.
            Action::ServeToggle => {
                let on = !matches!(self.state.serving, Serving::Running { .. });
                return vec![Effect::Serve(on)];
            }
            Action::ServePick => {
                self.state.picking = true;
                return self.act(Action::Open(Screen::Files), now);
            }
            // The folder shown is the one that will be served: the bar at the
            // foot of the file manager takes it.
            Action::ServeUseHere => {
                self.state.picking = false;
                let mut fx = self.act(Action::Open(Screen::Serve), now);
                fx.insert(0, Effect::Serve(true));
                return fx;
            }
            Action::Skip => {
                self.leave(true);
                if self.stack.len() > 1 {
                    self.stack.pop();
                } else {
                    *self.top_mut() = (Screen::Home, 0.0, Focus::Unset);
                }
                self.settle_page();
                self.refresh();
            }
            // The card asking to delete is what the page shows: back puts it
            // away before it leaves the page.
            // The sort panel is not a page of its own: going back closes it
            // rather than leaving the file manager.
            Action::Back if self.state.sort_panel => {
                self.state.sort_panel = false;
                self.refresh();
                return Vec::new();
            }
            Action::Back if self.state.asking_delete || self.state.asking_many.is_some() || self.state.image_asking.is_some() => {
                self.state.asking_delete = false;
                self.state.asking_many = None;
                self.state.image_asking = None;
                self.state.confirm = 0.0;
                self.refresh();
            }
            Action::Back | Action::CloseSearch => return self.back(now),
            Action::Theme { dark } => {
                if self.state.dark != dark {
                    self.state.dark = dark;
                    self.refresh();
                }
            }
            // The switch moves at once; the runtime then tells how the WLAN
            // came up, or went off.
            Action::Toggle(Switch::Wlan) => {
                let w = self.state.wlan.as_mut().expect("the WLAN switch shows on a phone whose WLAN is brought up");
                let on = matches!(w.power, Power::Off | Power::Failed(_));
                *w = Wlan { power: if on { Power::Starting } else { Power::Off }, ..Wlan::default() };
                self.refresh();
                return vec![Effect::Wlan(on)];
            }
            // The zone the system's gave the clock stays as the one chosen,
            // where it is one the pages list.
            Action::Toggle(Switch::SystemTimeZone) => {
                if self.state.follow_system_zone
                    && let Some(at) = self.state.zone_listed()
                {
                    self.state.time_zone = at;
                }
                self.state.follow_system_zone.toggle();
                self.refresh();
            }
            Action::Toggle(switch) => {
                // Keeping the screen lock is a risk taken on, and it is said
                // every time it is turned on, not once: the switch stays off
                // until the slider is taken across. A second tap while the
                // risk is shown puts it away again.
                if switch == Switch::Remember {
                    if self.state.asking_remember {
                        self.state.asking_remember = false;
                        self.refresh();
                        return Vec::new();
                    }
                    if !self.state.remember {
                        self.state.asking_remember = true;
                        self.refresh();
                        return Vec::new();
                    }
                    self.state.remember = false;
                    self.refresh();
                    return vec![Effect::Remember(false)];
                }
                // A charge step is chosen, not switched: tapping the one that
                // is already chosen does nothing. The programs that settle how
                // fast a charge may run are started as the recovery comes up,
                // so the page says the step reaches the phone at the next
                // start.
                if let Switch::ChargeStep(mode) = switch {
                    if self.state.charge_mode == mode {
                        return Vec::new();
                    }
                    self.state.charge_mode = mode;
                    self.refresh();
                    return vec![Effect::ChargeStep(mode)];
                }
                // The guard is not offered on the plain step, and does nothing
                // there: nothing a charge draws at that step can warm the
                // battery enough to be worth watching.
                if switch == Switch::ThermalGuard && !self.state.charge_mode.guarded() {
                    return Vec::new();
                }
                let on = match switch {
                    Switch::Vibrate => &mut self.state.vibrate,
                    Switch::FileTransfer => &mut self.state.file_transfer,
                    Switch::Compress => &mut self.state.compress,
                    Switch::Verify => &mut self.state.verify,
                    Switch::Signature => &mut self.state.check_signature,
                    Switch::KeepRecovery => &mut self.state.keep_recovery,
                    Switch::ShowPassword => &mut self.state.show_password,
                    Switch::Remember => &mut self.state.remember,
                    Switch::WlanAuto => &mut self.state.wlan_auto,
                    Switch::WlanKeep => &mut self.state.wlan_keep,
                    Switch::ThermalGuard => &mut self.state.thermal_guard,
                    Switch::Wlan | Switch::SystemTimeZone => unreachable!("the switch is turned above"),
                    Switch::ChargeStep(_) => unreachable!("a charge step is chosen above"),
                };
                *on = !*on;
                let now_on = *on;
                self.refresh();
                if switch == Switch::FileTransfer {
                    return vec![Effect::FileTransfer(now_on)];
                }
                if switch == Switch::Remember {
                    return vec![Effect::Remember(now_on)];
                }
                if switch == Switch::WlanAuto {
                    return vec![Effect::WlanAuto(now_on)];
                }
                if switch == Switch::WlanKeep {
                    return vec![Effect::WlanKeep(now_on)];
                }
            }
            Action::MountToggle(i) => {
                if self.state.mount_failure.take().is_some() {
                    self.refresh();
                }
                let row = &self.state.mounts.as_ref().expect("a partition toggled is one the page lists")[i];
                let base = row.base.clone();
                return vec![if row.mounted { Effect::Unmount(base) } else { Effect::Mount(base) }];
            }
            Action::RebootTarget(i) => {
                self.state.reboot = i;
                self.refresh();
            }
            // Everything takes in every part, so choosing it leaves the
            // parts out, and choosing a part on its own leaves it out.
            Action::WipeItem(item) => {
                let k = WIPES.iter().position(|&w| w == item).expect("the page offers only what it lists");
                let on = !self.state.wipe[k];
                match (on, item) {
                    (true, Wipe::All) => {
                        self.state.wipe = [false; WIPES.len()];
                        self.state.wipe[k] = true;
                    }
                    (true, _) => {
                        self.state.wipe[0] = false;
                        self.state.wipe[k] = true;
                    }
                    (false, _) => self.state.wipe[k] = false,
                }
                self.refresh();
            }
            Action::Confirm(_) => self.say(l.pick(texts::toast::SLIDE_RIGHT), now),
            Action::Slider(_) | Action::Pattern => {}
            Action::Sideload => match self.state.install {
                Install::Idle | Install::Finished { .. } => {
                    self.begin_install(Install::Starting, Source::Computer, None);
                    if self.screen() == Screen::Installing {
                        self.refresh();
                    } else {
                        self.open(Screen::Installing);
                    }
                    return vec![Effect::Sideload];
                }
                _ => self.say(l.pick(INSTALL_UNFINISHED), now),
            },
            Action::FlashPartition(k) => {
                if let Install::Choosing(choice) = &mut self.state.install {
                    choice.partition = k;
                    self.refresh();
                }
            }
            Action::FlashSlots(pick) => {
                if let Install::Choosing(choice) = &mut self.state.install {
                    choice.pick = pick;
                    self.refresh();
                }
            }
            Action::SystemSize(k) => {
                if let Install::SystemChoosing(choice) = &mut self.state.install {
                    choice.size = k;
                    self.refresh();
                }
            }
            Action::PickSystem(who) => {
                if self.state.systems_work.is_none() {
                    self.state.system_pick = Some(who);
                    self.state.asking_delete = false;
                    self.state.confirm = 0.0;
                    self.refresh();
                }
            }
            Action::RebootTo(target) => return vec![Effect::Reboot(target)],
            Action::EnterRecovery => return vec![Effect::EnterRecovery],
            Action::Dismiss => {
                self.stack = vec![(Screen::Home, 0.0, Focus::Unset)];
                self.settle_page();
                self.refresh();
                return vec![Effect::Dismiss];
            }
            Action::PickEntry(op) => {
                let path = self.entry_path();
                self.state.clip = Some(Clip { op, path });
                return self.back(now);
            }
            // A folder cannot go inside itself, and an entry moved to the
            // folder it is in stays put: the entry stays picked for another
            // folder.
            Action::Paste => {
                let clip = self.state.clip.clone().expect("only a picked entry is pasted");
                let here = self.state.folder_here(self.screen());
                let (path, folder) = (std::path::Path::new(&clip.path), std::path::Path::new(&here));
                let name = path.file_name().map_or_else(|| clip.path.clone(), |n| n.to_string_lossy().into_owned());
                if folder.starts_with(path) {
                    self.say(&texts::toast::inside_itself(l, name), now);
                } else if clip.op == FileOp::Move && path.parent() == Some(folder) {
                    self.say(&texts::toast::already_there(l, name), now);
                } else {
                    return vec![Effect::Paste { op: clip.op, path: clip.path, into: here, clash: None }];
                }
            }
            Action::Unpick => {
                self.state.clip = None;
                self.refresh();
            }
            Action::RenameEntry => {
                self.state.new_name = self.state.file.as_ref().expect("the file page shows an entry").name.clone();
                self.state.layer = Layer::Letters;
                self.open(Screen::Rename);
            }
            Action::EditEntry => return vec![Effect::OpenText(self.entry_path())],
            Action::InstallEntry => {
                let size = self.state.file.as_ref().expect("the file page shows an entry").size;
                return self.install_opened(self.entry_path(), size, now);
            }
            Action::SaveText => {
                let ed = self.state.editor.as_ref().expect("the editor page shows a file");
                return vec![Effect::SaveText { path: ed.path.clone(), text: ed.text() }];
            }
            Action::OpenPermissions => self.open(Screen::Permissions),
            Action::ModeBit(bit) => {
                let mode = self.state.file.as_ref().expect("the permissions page shows an entry").mode;
                return vec![Effect::SetMode { path: self.entry_path(), mode: (mode ^ bit) & 0o7777 }];
            }
            Action::AskDelete => {
                self.state.asking_delete = true;
                self.state.confirm = 0.0;
                self.refresh();
            }
            // Choosing several entries of the folder shown: a tick beside each,
            // and the bar at the foot carrying what can be done with them.
            Action::ChooseMany(on) => {
                self.state.choosing = on;
                self.state.chosen.clear();
                self.refresh();
            }
            Action::ChooseEntryAt(i) => {
                let folder = &self.state.files;
                if let Ok(entries) = &folder.entries
                    && let Some(e) = entries.get(i)
                {
                    let name = e.name.clone();
                    if !self.state.chosen.remove(&name) {
                        self.state.chosen.insert(name);
                    }
                    self.refresh();
                }
            }
            Action::ChooseAll => {
                let all: Vec<String> = self.state.files.entries.as_ref().map(|e| e.iter().map(|r| r.name.clone()).collect()).unwrap_or_default();
                if self.state.chosen.len() == all.len() {
                    self.state.chosen.clear();
                } else {
                    self.state.chosen = all.into_iter().collect();
                }
                self.refresh();
            }
            Action::PasteChosen(op) => {
                let into = self.state.folder_here(self.screen());
                let paths: Vec<String> = self.state.chosen.iter().map(|n| std::path::Path::new(&into).join(n).to_string_lossy().into_owned()).collect();
                if paths.is_empty() {
                    return Vec::new();
                }
                self.state.choosing = false;
                self.state.chosen.clear();
                self.refresh();
                return vec![Effect::PasteMany { op, paths, into, clash: None }];
            }
            // Deleting several is as destructive as deleting one, and asks the
            // same way: a swipe confirms, and nothing goes before it.
            Action::AskDeleteChosen => {
                let into = self.state.files.path.clone();
                let paths: Vec<String> = self.state.chosen.iter().map(|n| std::path::Path::new(&into).join(n).to_string_lossy().into_owned()).collect();
                if paths.is_empty() {
                    return Vec::new();
                }
                self.state.asking_many = Some(paths);
                self.state.confirm = 0.0;
                self.refresh();
            }
            Action::CancelDelete => {
                self.state.asking_delete = false;
                self.state.asking_many = None;
                self.state.confirm = 0.0;
                self.refresh();
            }
            Action::Resolve(choice) => {
                let (clip, into, _) = self.state.clash.take().expect("a choice answers a clash");
                self.leave(true);
                self.stack.pop();
                self.settle_page();
                self.refresh();
                // Skipping the one entry picked leaves nothing to do.
                if choice == ClashChoice::Skip {
                    self.state.clip = None;
                    return Vec::new();
                }
                return vec![Effect::Paste { op: clip.op, path: clip.path, into, clash: Some(choice) }];
            }
            Action::StopJob => {
                let job = self.state.job.as_mut().expect("only work under way is stopped");
                if job.outcome.is_none() && !job.stopping {
                    job.stopping = true;
                    self.refresh();
                    return vec![Effect::StopJob];
                }
            }
            Action::StopRelabel => {
                let job = self.state.relabel.as_mut().expect("only work under way is stopped");
                if job.outcome.is_none() && !job.stopping {
                    job.stopping = true;
                    self.refresh();
                    return vec![Effect::StopRelabel];
                }
            }
            Action::OpenFilesystem(k) => {
                self.state.fs_chosen = k;
                self.state.fs_action = FsAction::Repair;
                self.open(Screen::Filesystem);
            }
            // A tap on a row of the image page asks with a swipe before
            // anything is written or deleted: writing a partition out, or
            // over one, is not something a stray touch should start.
            Action::ExportImage(k) => self.ask_image(Confirm::ExportImage(k)),
            Action::ImportImage(k) => self.ask_image(Confirm::ImportImage(k)),
            Action::DeleteImage(k) => self.ask_image(Confirm::DeleteImage(k)),
            Action::StopImage => {
                if let Some(job) = self.state.image_job.as_mut()
                    && job.outcome.is_none()
                {
                    job.stopping = true;
                    self.refresh();
                    return vec![Effect::StopImage];
                }
            }
            Action::ImageShows(k) => {
                self.state.image_showing_parts = k;
                self.refresh();
            }
            Action::ImageTo(drive) => {
                self.state.image_to_drive = drive;
                self.refresh();
            }
            Action::ChooseFs(choice) => {
                let row = self.fs_row().expect("the page of a partition shows one that was read");
                let action = match choice {
                    FsChoice::Repair => FsAction::Repair,
                    FsChoice::Remake(k) => FsAction::Remake(row.others[k].clone()),
                };
                self.state.fs_action = action;
                self.refresh();
            }
            Action::ScreenOff(k) => {
                self.state.screen_off = k;
                self.refresh();
            }
            Action::TimeZone(k) => {
                self.state.time_zone = k;
                self.state.follow_system_zone = false;
                self.refresh();
            }
            Action::Language(l) => {
                self.state.lang = l;
                self.state.language_chosen = true;
                self.refresh();
            }
            Action::BackupPart(k) => {
                self.state.backup[k].toggle();
                self.refresh();
            }
            Action::BackupTo(place) => {
                self.state.backup_to = place;
                self.refresh();
            }
            // All a backup holds is ticked to restore at first.
            Action::OpenBackup(i) => {
                let entry = self.state.backups[i].clone();
                let parts = entry.held.as_ref().map_or([false; 5], |h| h.parts);
                self.state.restore = Some((entry, parts));
                self.state.asking_delete = false;
                self.open(Screen::RestoreBackup);
            }
            Action::RestorePart(k) => {
                let (_, parts) = self.state.restore.as_mut().expect("the restore page shows a backup");
                parts[k].toggle();
                self.refresh();
            }
            Action::StopBackup => {
                let job = self.state.backup_job.as_mut().expect("only work under way is stopped");
                if job.outcome.is_none() && !job.stopping {
                    job.stopping = true;
                    self.refresh();
                    return vec![Effect::StopBackup];
                }
            }
            Action::LogPart(k) => {
                self.state.logs[k].toggle();
                self.refresh();
            }
            Action::LogsTo(k) => {
                self.state.logs_to = k;
                self.refresh();
            }
            Action::ExportLogs => {
                let ([recovery, kernel], storage) = (self.state.logs, self.state.logs_to == 1);
                if !recovery && !kernel {
                    self.say(l.pick(texts::toast::CHOOSE_LOGS), now);
                } else if storage && !self.state.data.storage_open() {
                    self.say(l.pick(DECRYPT_FIRST), now);
                } else {
                    return vec![Effect::ExportLogs { recovery, kernel, storage }];
                }
            }
            Action::SendReport | Action::SendReportAgain => {
                if self.state.feedback.report.sent() {
                    self.say(l.pick(["本次已反馈过，重启后才能再反馈", "本次已回報過，重啟後才能再回報", "A report has been sent this start; start again to send another"]), now);
                } else if self.state.feedback.report.sending() {
                    self.say(l.pick(["正在上传，请稍候", "正在上傳，請稍候", "A report is on its way"]), now);
                } else if !self.state.feedback.ready() {
                    self.say(l.pick(["请先描述问题，再点提交", "請先描述問題，再點提交", "Write about the bug before sending it"]), now);
                } else if matches!(action, Action::SendReportAgain) {
                    return vec![Effect::SendReportAgain];
                } else {
                    return vec![Effect::SendReport];
                }
            }
            Action::WriteReportAgain => self.report_again(),
            Action::PickSlot(slot) => {
                self.state.next_slot = Some(slot);
                self.refresh();
            }
            Action::Type(cap) if self.screen() == Screen::Terminal => return self.term_typed(cap),
            Action::Type(cap) if self.screen() == Screen::Editor => self.edit_typed(cap),
            Action::Type(cap) => return self.type_key(cap, now),
            Action::TermKey(key) if self.screen() == Screen::Editor => {
                let ed = self.state.editor.as_mut().expect("the editor page shows a file");
                match key {
                    TermKey::Tab => ed.insert('\t'),
                    TermKey::Left => ed.left(),
                    TermKey::Up => ed.up(),
                    TermKey::Down => ed.down(),
                    TermKey::Right => ed.right(),
                    TermKey::Esc | TermKey::Ctrl => unreachable!("the editor's row has no {key:?}"),
                }
                self.edits += 1;
                self.refresh_text();
            }
            Action::TermKey(key) => {
                let Some(term) = self.term.as_mut() else {
                    return Vec::new();
                };
                let bytes = term.key(key);
                let running = term.running;
                self.refresh();
                if running && !bytes.is_empty() {
                    return vec![Effect::TermInput(bytes)];
                }
            }
            Action::ClearSearch => {
                self.state.search.clear();
                self.edits += 1;
                self.refresh();
            }
            Action::OpenEntry(i) => {
                let entry = self.state.files.entry(i).clone();
                match entry.kind {
                    FileKind::Folder { .. } | FileKind::Link { folder: true, .. } => {
                        let scroll = self.top().1;
                        self.state.files.leave(scroll);
                        return vec![Effect::ReadDir(self.state.files.inside(&entry.name))];
                    }
                    FileKind::File if matches!(files::file_type(&entry.name), FileType::Text | FileType::Code) => {
                        return vec![Effect::OpenText(self.state.files.inside(&entry.name))];
                    }
                    _ if installs(&entry) => {
                        let path = self.state.files.inside(&entry.name);
                        return self.install_opened(path, entry.size, now);
                    }
                    FileKind::File | FileKind::Link { folder: false, .. } | FileKind::Special => {
                        self.state.file = Some(entry);
                        self.open(Screen::FileInfo);
                    }
                }
            }
            // A shell already running goes to the folder; otherwise one starts
            // there.
            Action::TerminalHere => {
                let folder = self.state.files.path.clone();
                let running = self.term.as_ref().is_some_and(|term| term.running);
                self.open(Screen::Terminal);
                self.state.layer = Layer::Letters;
                if running {
                    return vec![Effect::ShellTo(folder)];
                }
                self.shell_folder = folder.clone();
                return vec![Effect::StartShell(folder)];
            }
            Action::ChooseEntry(i) => {
                let screen = self.screen();
                // A tap that lands as the page turns is nobody's mistake: the
                // place's page is the only one that chooses, and one that is
                // not showing has nothing to choose.
                let Screen::Choose(place) = screen else { return Vec::new() };
                let folder = self.state.folder(screen);
                let Ok(entries) = &folder.entries else { return Vec::new() };
                let Some(entry) = entries.get(i).cloned() else { return Vec::new() };
                let path = folder.inside(&entry.name);
                match entry.kind {
                    FileKind::Folder { .. } | FileKind::Link { folder: true, .. } => {
                        let scroll = self.top().1;
                        self.state.folder_mut(screen).leave(scroll);
                        return vec![Effect::ReadPlace(place, path)];
                    }
                    FileKind::File | FileKind::Link { folder: false, .. } => {
                        return self.install_file(place, path, entry.size, now);
                    }
                    FileKind::Special => unreachable!("a place lists no special files"),
                }
            }
            Action::OpenAncestor(depth) => {
                let screen = self.screen();
                if let Some(path) = self.state.folder(screen).ancestor(depth) {
                    return vec![match screen {
                        Screen::Choose(place) => Effect::ReadPlace(place, path),
                        _ => Effect::ReadDir(path),
                    }];
                }
            }
            Action::JoinNetwork(i) => {
                let w = self.state.wlan.as_ref().expect("networks show on a phone whose WLAN is brought up");
                // A scan may have changed the list since the page was drawn.
                let Some(network) = w.networks.get(i).cloned() else {
                    return Vec::new();
                };
                if w.joined.as_ref().is_some_and(|j| j.ssid == network.ssid) {
                    return self.act(Action::OpenJoined, now);
                }
                match network.security {
                    Security::Wep => self.say(l.pick(["暂不支持 WEP 加密的网络", "尚不支援 WEP 加密的網路", "WEP networks aren't supported"]), now),
                    Security::Enterprise => self.say(l.pick(["暂不支持需要账号登录的网络", "尚不支援需要帳號登入的網路", "Networks that need an account sign-in aren't supported"]), now),
                    s if s.password() => self.ask_password(network),
                    _ => {
                        return vec![Effect::JoinWlan { ssid: network.ssid, name: network.name, security: network.security, password: Secret::default() }];
                    }
                }
            }
            Action::NetworkDetails(i) => {
                let w = self.state.wlan.as_ref().expect("networks show on a phone whose WLAN is brought up");
                let Some(network) = w.networks.get(i).cloned() else {
                    return Vec::new();
                };
                self.state.details = Some(network);
                self.open(Screen::WlanNetwork);
            }
            Action::OpenJoined => {
                let w = self.state.wlan.as_ref().expect("a network is chosen on a phone whose WLAN is brought up");
                let Some(j) = w.joined.clone() else {
                    return Vec::new();
                };
                // As found, where it still is; as it was chosen otherwise.
                let found = w.networks.iter().find(|n| n.ssid == j.ssid).cloned();
                let network = found.unwrap_or(Network { ssid: j.ssid, name: j.name, signal: 0, frequency: 0, bands: Default::default(), security: j.security });
                if j.stage == Joining::WrongPassword {
                    self.ask_password(network);
                } else {
                    self.state.details = Some(network);
                    self.open(Screen::WlanNetwork);
                }
            }
            Action::LeaveNetwork => {
                let mut fx = self.back(now);
                fx.push(Effect::LeaveWlan);
                return fx;
            }
            Action::Rescan => return vec![Effect::ScanWlan],
            Action::AddNetwork => {
                self.state.adding = Some(Adding { name: String::new(), security: Security::Open, field: Field::Name, choosing: false });
                self.state.password.clear();
                self.state.show_password = false;
                self.state.layer = Layer::Letters;
                self.open(Screen::WlanAdd);
            }
            Action::TypeInto(field) => {
                let a = self.state.adding.as_mut().expect("the fields are on the page that adds a network");
                if a.field != field {
                    a.field = field;
                    self.refresh();
                }
            }
            Action::ChooseSecurity => {
                self.state.adding.as_mut().expect("the protection is on the page that adds a network").choosing = true;
                // The page draws the row held down while its list is open.
                self.row_press = None;
                self.row_art = None;
                self.refresh();
            }
            Action::PickSecurity(security) => {
                let a = self.state.adding.as_mut().expect("the protections open on the page that adds a network");
                a.security = security;
                a.choosing = false;
                // With no password to type, the keyboard types the name.
                if !security.password() {
                    a.field = Field::Name;
                }
                self.refresh();
            }
            Action::CloseChoices => {
                self.state.adding.as_mut().expect("the protections open on the page that adds a network").choosing = false;
                self.refresh();
            }
            // Asking takes a network, which the WLAN page joins.
            Action::CheckUpdate => {
                if !self.state.online() {
                    self.open(Screen::Wlan);
                    return self.arrive_at(Screen::Wlan);
                }
                if self.state.check != Check::Asking {
                    self.state.check = Check::Asking;
                    self.refresh();
                    return vec![Effect::CheckUpdate];
                }
            }
            // The package goes to internal storage, which the decrypt page
            // opens, over a network the WLAN page joins.
            Action::Download => {
                let release = self.state.newest().expect("the download is offered for the newest system").clone();
                if !self.state.data.storage_open() {
                    self.open(Screen::Decrypt);
                    return self.arrive_at(Screen::Decrypt);
                }
                if !self.state.online() {
                    self.open(Screen::Wlan);
                    return self.arrive_at(Screen::Wlan);
                }
                if !self.state.downloading() {
                    self.state.download = Some(DownloadJob { progress: None, rate: 0, outcome: None, stopping: false });
                    self.refresh();
                    return vec![Effect::Download(release)];
                }
            }
            Action::StopDownload => {
                let job = self.state.download.as_mut().expect("only a download under way is stopped");
                if job.outcome.is_none() && !job.stopping {
                    job.stopping = true;
                    self.refresh();
                    return vec![Effect::StopDownload];
                }
            }
            Action::InstallDownloaded => {
                let name = self.state.newest().expect("the package installed is the newest system's").name.clone();
                let Have::Whole(size) = self.state.have else { unreachable!("only a whole package is offered to install") };
                return self.install_file(Place::Storage, format!("{DOWNLOADS}/{name}"), size, now);
            }
            // Asking takes a network, which the WLAN page joins.
            Action::CheckSkyward => {
                if !self.state.online() {
                    self.open(Screen::Wlan);
                    return self.arrive_at(Screen::Wlan);
                }
                if self.state.skyward_check != SkywardCheck::Asking {
                    self.state.skyward_check = SkywardCheck::Asking;
                    self.refresh();
                    return vec![Effect::CheckSkyward];
                }
            }
            // The build goes into the recovery partitions an install may
            // write too, so one waits for the other.
            Action::UpdateSkyward => {
                let build = self.state.skyward_newest().and_then(|n| n.build.clone()).expect("the update is offered for the newest release's build");
                if !self.state.online() {
                    self.open(Screen::Wlan);
                    return self.arrive_at(Screen::Wlan);
                }
                if !matches!(self.state.install, Install::Idle | Install::Finished { .. }) {
                    self.say(l.pick(INSTALL_UNFINISHED), now);
                    return Vec::new();
                }
                if !self.state.updating_skyward() {
                    self.state.skyward_job = Some(SkywardJob { stage: None, rate: 0, outcome: None, stopping: false });
                    self.refresh();
                    return vec![Effect::UpdateSkyward(build)];
                }
            }
            // Only the download stops: writing, once begun, goes on to its
            // end.
            Action::StopSkywardUpdate => {
                let job = self.state.skyward_job.as_mut().expect("only an update under way is stopped");
                if job.outcome.is_none() && !job.stopping && !matches!(job.stage, Some(SkywardStage::Write { .. })) {
                    job.stopping = true;
                    self.refresh();
                    return vec![Effect::StopSkywardUpdate];
                }
            }
            Action::ReadAvb => return vec![Effect::ReadAvb],
            Action::ToggleAvb(check) => {
                self.state.avb_notice = None;
                self.avb_started(check);
                return vec![Effect::ToggleAvb(check)];
            }
            Action::BackupVbmeta => {
                self.state.avb_notice = None;
                self.state.avb_notice = Some(l.pick(["正在备份 vbmeta…", "正在備份 vbmeta…", "Backing up vbmeta…"]).into());
                self.refresh();
                return vec![Effect::BackupVbmeta];
            }
            Action::RestoreVbmeta => {
                self.state.avb_notice = None;
                self.state.avb_notice = Some(l.pick(["正在恢复 vbmeta…", "正在還原 vbmeta…", "Restoring vbmeta…"]).into());
                self.refresh();
                return vec![Effect::RestoreVbmeta];
            }
            // The OTG drive's files: listed on a page of their own, and an
            // entry in it opens as any other folder's does.
            Action::FilesOtg => {
                if self.state.place_open(Place::UsbDrive) {
                    self.state.otg = Folder::new("");
                    self.open(Screen::FilesOtg);
                    return vec![Effect::ReadOtg(String::new())];
                }
                self.say(l.pick(["未连接 U 盘", "未連接 USB 隨身碟", "No USB drive"]), now);
            }
            Action::OpenOtgEntry(i) => {
                let entry = self.state.otg.entry(i).clone();
                match entry.kind {
                    FileKind::Folder { .. } | FileKind::Link { folder: true, .. } => {
                        self.state.otg.leave(self.top().1);
                        return vec![Effect::ReadOtg(self.state.otg.inside(&entry.name))];
                    }
                    _ => {
                        self.state.file_on_drive = true;
                        self.state.file = Some(entry);
                        self.open(Screen::FileInfo);
                    }
                }
            }
            // Across to the other side, the file kept picked: the page it
            // goes to is opened so that the folder it lands in can be walked
            // to, and the bar at the foot puts it down there.
            // 列表点到一行：打开这一行的详情，那儿才有开关和卸载。
            Action::ModuleOpen(k) => {
                self.state.module_at = Some(k);
                return self.act(Action::Open(Screen::ModuleInfo), now);
            }
            // The apps page shows a screen at a time: turning it is only a
            // change of which screen, and the page redraws. The screen to turn
            // to is given outright rather than as a step, so that a list that
            // has grown shorter -- an app taken off it -- cannot leave the
            // page out past the end of it.
            Action::AppPage(to) => {
                self.state.app_page = to;
                self.refresh();
                return Vec::new();
            }
            Action::SysPage(to) => {
                self.state.sys_page = to;
                self.refresh();
                return Vec::new();
            }
            // A system app's row is tapped: it turns over from what it is to
            // what it is headed for, and the change is queued. The package
            // manager's own record is a binary format that writing by hand
            // would break, so the system itself makes the change, at its next
            // start; the page shows the app as waiting until then.
            Action::SysAppToggle(i) => {
                let package = match &self.state.sys_apps {
                    Some(list) => list.get(i).map(|a| a.package.clone()),
                    None => None,
                };
                if let Some(package) = package {
                    // What it is headed for already, if it was tapped before;
                    // otherwise what it is now.
                    let now_disabled = match self.state.app_state_queued.get(&package) {
                        Some(&d) => d,
                        None => self.state.app_disabled.as_ref().and_then(|m| m.get(&package)).copied().unwrap_or(false),
                    };
                    return vec![Effect::QueueAppState { package, disable: !now_disabled }];
                }
            }
            Action::DataAppOpen(k) => {
                self.state.app_at = Some(k);
                return self.act(Action::Open(Screen::DataAppInfo), now);
            }
            // The search page is opened to look through one page's entries:
            // the query is cleared, and the page it was opened from says what
            // it is looking through.
            Action::SearchApps | Action::SearchModules | Action::SearchFiles => {
                self.state.search_scope = match action {
                    Action::SearchApps => SearchScope::Apps,
                    Action::SearchModules => SearchScope::Modules,
                    _ => SearchScope::Files,
                };
                self.state.search.clear();
                return self.act(Action::Open(Screen::Search), now);
            }
            // 标记删除（或取消标记）：KSU/Magisk 都是看模块目录里有没有
            // `remove` 文件，有就在下次启动时清掉。
            Action::ModuleRemove(k) => {
                let Some(Ok(list)) = &self.state.modules else { return Vec::new() };
                let Some(m) = list.get(k) else { return Vec::new() };
                return vec![Effect::ModuleRemove(m.id.clone(), !m.removing)];
            }
            // 文件页上那个 zip 当成模块装：zip 里必须有 module.prop。
            Action::InstallAsModule => {
                let Some(e) = &self.state.file else { return Vec::new() };
                let zip = e.name.to_lowercase().ends_with(".zip");
                if e.kind != FileKind::File || !zip {
                    self.say(l.pick(["只有 zip 能当模块安装", "只有 zip 能當模組安裝", "Only a zip can be a module"]), now);
                    return Vec::new();
                }
                let path = std::path::Path::new(&self.state.files.path).join(&e.name).to_string_lossy().into_owned();
                return vec![Effect::ModuleInstall(path)];
            }
            // 模块的开关交给 runtime 去做：写或删那个 disable 文件，再读一遍。
            Action::ModuleToggle(k) => {
                let Some(Ok(list)) = &self.state.modules else { return Vec::new() };
                let Some(m) = list.get(k) else { return Vec::new() };
                return vec![Effect::ModuleSwitch(m.id.clone(), !m.enabled)];
            }
            Action::AskPlace(op) => {
                self.state.place_ask = Some(op);
                self.refresh();
                return Vec::new();
            }
            Action::DismissPlace => {
                self.state.place_ask = None;
                self.refresh();
                return Vec::new();
            }
            Action::ToDrive(op) | Action::ToPhone(op) => {
                self.state.place_ask = None;
                let to_drive = matches!(action, Action::ToDrive(_));
                if to_drive && !self.state.place_open(Place::UsbDrive) {
                    self.say(l.pick(["请先连接 U 盘", "請先連接 USB 隨身碟", "Connect a USB drive first"]), now);
                    return Vec::new();
                }
                let path = self.entry_path();
                // The entry's own folder is no place to put it down again, and
                // a folder cannot go inside itself.
                let here = match to_drive {
                    true => self.state.otg.path.trim_matches('/').to_string(),
                    false => self.state.files.path.clone(),
                };
                let nothing_to_do = match to_drive {
                    true => false,
                    false => op == FileOp::Move && std::path::Path::new(&path).parent() == Some(std::path::Path::new(&here)),
                };
                if nothing_to_do {
                    let name = std::path::Path::new(&path).file_name().map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());
                    self.say(&tr!(l, "\"{name}\"已在该文件夹中", "「{name}」已在這個資料夾中", "“{name}” is already in this folder"), now);
                    return Vec::new();
                }
                self.state.clip = Some(Clip { op, path });
                self.refresh();
                return match to_drive {
                    true => {
                        self.open(Screen::FilesOtg);
                        vec![Effect::ReadOtg(self.state.otg.path.clone())]
                    }
                    false => {
                        self.state.file_on_drive = false;
                        self.open(Screen::Files);
                        vec![Effect::ReadDir(self.state.files.path.clone())]
                    }
                };
            }
            Action::Unavailable(text) => self.say(text, now),
        }
        Vec::new()
    }

    /// The text the keyboard types into on this page.
    fn typing(&mut self) -> &mut String {
        match self.screen() {
            Screen::Search => &mut self.state.search,
            Screen::Decrypt => &mut self.state.code,
            Screen::Rename => &mut self.state.new_name,
            Screen::WlanPassword => &mut self.state.password,
            Screen::Fetch => &mut self.state.fetch_url,
            Screen::Feedback => &mut self.state.feedback.words,
            Screen::WlanAdd => {
                let a = self.state.adding.as_mut().expect("the page adds a network");
                match a.field {
                    Field::Name => &mut a.name,
                    Field::Password => &mut self.state.password,
                }
            }
            s => panic!("{s:?} has no text field"),
        }
    }

    /// A key of the on-screen keyboard. A capital lasts one letter; return
    /// opens the first result of a search, and runs a terminal's command.
    fn type_key(&mut self, cap: Cap, now: u64) -> Vec<Effect> {
        let (l, layer) = (self.state.lang, self.state.layer);
        match cap {
            Cap::Char(ch) => {
                let shifted = self.state.layer == Layer::Shifted;
                self.typing().push(if shifted { ch.to_ascii_uppercase() } else { ch });
                self.edits += 1;
                if shifted {
                    self.state.layer = Layer::Letters;
                }
            }
            Cap::Space => {
                self.typing().push(' ');
                self.edits += 1;
            }
            Cap::Backspace => {
                self.typing().pop();
                self.edits += 1;
            }
            Cap::Shift => {
                self.state.layer = if self.state.layer == Layer::Shifted { Layer::Letters } else { Layer::Shifted };
            }
            Cap::Show(layer) => self.state.layer = layer,
            Cap::Enter => match self.screen() {
                Screen::Search => {
                    let query = self.state.search.trim().to_lowercase();
                    match self.state.search_scope {
                        SearchScope::Features => {
                            if let Some(first) = search::find_on(&self.state.search, l, |screen| self.state.offers(screen)).first() {
                                return self.act(Action::Open(first.screen), now);
                            }
                        }
                        // Looking through a page's entries, the first that
                        // answers is the one opened. The file manager's own
                        // page filters as the name is typed, so return has
                        // nothing left to open.
                        scope => {
                            let Some(first) = self.first_match(scope, &query) else { return Vec::new() };
                            return match scope {
                                SearchScope::Apps => self.act(Action::DataAppOpen(first), now),
                                SearchScope::Files => Vec::new(),
                                _ => self.act(Action::ModuleOpen(first), now),
                            };
                        }
                    }
                }
                // Return starts the download, or stops the one under way.
                Screen::Fetch if self.fetching() => return self.act(Action::FetchStop, now),
                Screen::Fetch => return self.act(Action::FetchStart, now),
                Screen::Decrypt if self.state.code.is_empty() => self.say(l.pick(["请输入锁屏密码", "請輸入螢幕鎖定密碼", "Enter the screen lock"]), now),
                Screen::Decrypt => {
                    let code = std::mem::take(&mut self.state.code);
                    return self.unlock(code);
                }
                // A name the folder cannot take is turned down here, in
                // the user's words; an unchanged one leaves the page.
                Screen::Rename => {
                    let name = self.state.new_name.clone();
                    let current = &self.state.file.as_ref().expect("the file page shows an entry").name;
                    let taken = self.state.files.entries.as_ref().is_ok_and(|entries| entries.iter().any(|e| e.name == name));
                    match name.as_str() {
                        "" => self.say(l.pick(texts::toast::ENTER_NAME), now),
                        "." | ".." => self.say(l.pick(texts::toast::NAME_DOTS), now),
                        _ if name.contains('/') => self.say(l.pick(texts::toast::NAME_SLASH), now),
                        _ if name == *current => return self.back(now),
                        _ if taken => self.say_taken(now),
                        _ => {
                            return vec![Effect::Rename { path: self.entry_path(), name }];
                        }
                    }
                }
                // A password the network's protection cannot take is turned
                // down here; one it can goes, and the WLAN page shows how
                // joining goes.
                Screen::WlanPassword => {
                    let network = self.state.joining.clone().expect("the password page is for a network");
                    match refused(l, network.security, &self.state.password) {
                        Some(why) => self.say(why, now),
                        None => {
                            let password = Secret(std::mem::take(&mut self.state.password));
                            let mut fx = self.back(now);
                            fx.push(Effect::JoinWlan { ssid: network.ssid, name: network.name, security: network.security, password });
                            return fx;
                        }
                    }
                }
                // So is a name no network can have; the keyboard then types
                // into the field turned down.
                Screen::WlanAdd => {
                    let a = self.state.adding.as_mut().expect("the page adds a network");
                    let (field, why) = match a.name.len() {
                        0 => (Field::Name, Some(l.pick(["请输入网络名称", "請輸入網路名稱", "Enter the network's name"]).to_string())),
                        n if n > SSID_MAX => {
                            (Field::Name, Some(tr!(l, "网络名称不能超过 {SSID_MAX} 个字符", "網路名稱不能超過 {SSID_MAX} 個字元", "A network's name can't be over {SSID_MAX} characters")))
                        }
                        _ if a.security.password() => (Field::Password, refused(l, a.security, &self.state.password).map(str::to_string)),
                        _ => (Field::Name, None),
                    };
                    match why {
                        Some(why) => {
                            if a.field != field {
                                a.field = field;
                                self.refresh();
                            }
                            self.say(&why, now)
                        }
                        None => {
                            let (name, security) = (a.name.clone(), a.security);
                            let password = match security.password() {
                                true => Secret(std::mem::take(&mut self.state.password)),
                                false => Secret::default(),
                            };
                            let mut fx = self.back(now);
                            fx.push(Effect::JoinWlan { ssid: name.clone().into_bytes(), name, security, password });
                            return fx;
                        }
                    }
                }
                // The report is written over as many lines as it takes: the
                // return key starts another, and goes nowhere else.
                Screen::Feedback => {
                    self.state.feedback.push('\n');
                    self.edits += 1;
                }
                other => unreachable!("{other:?} shows no keyboard"),
            },
        }
        // What is typed shows above the keyboard, which stays as it was
        // while its keys do.
        if matches!(cap, Cap::Char(_) | Cap::Space | Cap::Backspace) && self.state.layer == layer {
            let t = self.theme();
            self.refresh_rows(0, (t.m.h as f32 - widgets::keyboard_height(&t)).floor() as u32);
        } else {
            self.refresh();
        }
        Vec::new()
    }

    /// The row of the image page tapped: its swipe comes up over the page, and
    /// nothing is written until it is taken across.
    fn ask_image(&mut self, which: Confirm) {
        self.state.image_asking = Some(which);
        self.state.confirm = 0.0;
        self.drag = Drag::Idle;
        self.refresh();
    }

    /// Write the partition at `k` out as an image file, the swipe taken across
    /// saying so.
    pub(super) fn export_image(&mut self, k: usize) -> Vec<Effect> {
        let parts = match &self.state.image_parts {
            Ok(parts) => parts,
            Err(_) => return Vec::new(),
        };
        let Some(part) = parts.get(k) else { return Vec::new() };
        let (partition, size) = (part.name.clone(), part.size);
        self.state.image_job = Some(ImageJob { kind: ImageFlow::Export, partition: partition.clone(), done: 0, of: size, stopping: false, outcome: None });
        self.refresh();
        vec![Effect::ExportImage { partition, size, drive: self.state.image_to_drive }]
    }

    /// Write the image at `k` back over the partition it came from.
    pub(super) fn import_image(&mut self, k: usize) -> Vec<Effect> {
        let files = match &self.state.image_files {
            Ok(files) => files,
            Err(_) => return Vec::new(),
        };
        let Some(file) = files.get(k) else { return Vec::new() };
        let (partition, path, sha256) = (file.partition.clone(), file.path.clone(), file.sha256.clone());
        self.state.image_job = Some(ImageJob { kind: ImageFlow::Import, partition: partition.clone(), done: 0, of: file.size, stopping: false, outcome: None });
        self.refresh();
        vec![Effect::ImportImage { partition, path, sha256 }]
    }

    /// Delete the image at `k`.
    pub(super) fn delete_image(&mut self, k: usize) -> Vec<Effect> {
        let files = match &self.state.image_files {
            Ok(files) => files,
            Err(_) => return Vec::new(),
        };
        let Some(file) = files.get(k) else { return Vec::new() };
        let path = file.path.clone();
        self.refresh();
        vec![Effect::DeleteImage(path)]
    }

    /// What a swipe-to-confirm control slid to its end does.
    pub(super) fn confirmed(&mut self, which: Confirm, now: u64) -> Vec<Effect> {
        let (l, mut fx) = (self.state.lang, Vec::new());
        // Every slider taken across is answered with a click of the motor,
        // whatever it was asking: the finger is told the question was taken
        // without looking at the page it was taken on.
        if self.state.vibrate {
            fx.push(Effect::Haptic);
        }
        match which {
            // 锁屏被滑开：锁去了，页面回到手上。
            Confirm::Unlock => {
                self.lock = None;
                self.refresh();
            }
            Confirm::Reboot => fx.push(Effect::Reboot(TARGETS[self.state.reboot])),
            Confirm::PowerOff => fx.push(Effect::Reboot(Target::PowerOff)),
            // The slider taken across: the lock is kept from now on. The risk
            // is asked again the next time it is turned on.
            Confirm::KeepLock => {
                self.state.asking_remember = false;
                self.state.remember = true;
                self.refresh();
                fx.push(Effect::Remember(true));
            }
            Confirm::Wipe => {
                let chosen: Vec<Wipe> = WIPES.iter().zip(self.state.wipe).filter(|(_, on)| *on).map(|(&w, _)| w).collect();
                match chosen.is_empty() {
                    true => self.say(l.pick(texts::toast::CHOOSE_WIPE), now),
                    false => fx.push(Effect::Wipe(chosen)),
                }
            }
            // The image page's own swipes do what their rows used to do at a
            // tap, now that a tap alone cannot start one by mistake.
            Confirm::ExportImage(k) => {
                self.state.image_asking = None;
                fx.extend(self.export_image(k));
            }
            Confirm::ImportImage(k) => {
                self.state.image_asking = None;
                fx.extend(self.import_image(k));
            }
            Confirm::DeleteImage(k) => {
                self.state.image_asking = None;
                fx.extend(self.delete_image(k));
            }
            Confirm::Install | Confirm::InstallWipe => {
                if let Install::Confirming { version, .. } = &self.state.install {
                    self.state.install = Install::Updating { version: version.clone(), stage: None, overwriting: false };
                    fx.push(Effect::Install);
                    self.refresh();
                }
            }
            Confirm::RunScript => {
                if self.state.install == Install::ConfirmingScript {
                    self.state.install = Install::Scripting { stage: None, running: false };
                    fx.push(Effect::Install);
                    self.refresh();
                }
            }
            Confirm::Backup => {
                let s = &self.state;
                let refused = if !s.backup.contains(&true) {
                    Some(texts::toast::CHOOSE_BACKUP)
                } else if (s.backup[DATA_PART] || s.backup[MEDIA_PART]) && s.data != Data::Open {
                    Some(texts::toast::DECRYPT_TO_BACK_UP)
                } else if s.place_open(s.backup_to) {
                    None
                } else {
                    Some(match (s.backup_to, &s.usb_drive) {
                        (Place::Storage, _) if s.data == Data::Erased => DATA_ERASED,
                        (Place::Storage, _) => DECRYPT_FIRST,
                        (Place::UsbDrive, Some(UsbDrive::Unreadable(_))) => USB_DRIVE_UNREADABLE,
                        (Place::UsbDrive, _) => USB_DRIVE_WANTED,
                    })
                };
                let backup = Effect::Backup { parts: s.backup, place: s.backup_to, compress: s.compress, verify: s.verify };
                match refused {
                    Some(why) => self.say(l.pick(why), now),
                    None => fx.push(backup),
                }
            }
            Confirm::Restore => {
                let (entry, parts) = self.state.restore.as_ref().expect("the restore page shows a backup");
                let restore = Effect::Restore { place: entry.place, name: entry.name.clone(), parts: *parts };
                // The files of the data partition and of internal storage go
                // back into a data partition that is open: one erased is
                // made again by the system, as it next starts.
                let files = parts[DATA_PART] || parts[MEDIA_PART];
                if !parts.contains(&true) {
                    self.say(l.pick(texts::toast::CHOOSE_RESTORE), now);
                } else if files && self.state.data == Data::Erased {
                    self.say(l.pick(texts::toast::RESTORE_ONCE_MADE), now);
                } else if files && self.state.data != Data::Open {
                    self.say(l.pick(texts::toast::DECRYPT_TO_RESTORE), now);
                } else {
                    fx.push(restore);
                }
            }
            Confirm::DeleteBackup => {
                let (entry, _) = self.state.restore.as_ref().expect("the restore page shows a backup");
                fx.push(Effect::DeleteBackup { place: entry.place, name: entry.name.clone() });
            }
            // The app is taken off once the swipe has agreed to it: its folder
            // in the data partition, and what it kept there.
            Confirm::DeleteApp => {
                if let Some(at) = self.state.app_at
                    && let Some(Ok(list)) = &self.state.apps
                    && let Some(a) = list.get(at)
                {
                    fx.push(Effect::DataAppRemove(a.package.clone(), a.dir.clone()));
                }
            }
            Confirm::Slot => match (&self.state.boot_slots, self.state.next_slot) {
                (Some(Ok(slots)), Some(pick)) if pick != slots.active => fx.push(Effect::SetSlot(pick)),
                (Some(Ok(slots)), _) => self.say(&texts::toast::slot_already_next(l, slots.active.letter()), now),
                _ => self.say(l.pick(texts::slot::UNREADABLE), now),
            },
            Confirm::FactoryReset => fx.push(Effect::FactoryReset),
            Confirm::Merge => {
                if self.state.update == Standing::Waiting {
                    self.state.update = Standing::Merging(0.0);
                    self.refresh();
                    fx.push(Effect::Merge);
                }
            }
            Confirm::DeleteEntry => match self.state.asking_many.take() {
                Some(paths) => {
                    self.state.choosing = false;
                    self.state.chosen.clear();
                    self.refresh();
                    fx.push(Effect::DeleteMany(paths));
                }
                None => fx.push(Effect::Delete(self.entry_path())),
            },
            // The partition has to be open; whether its users have unlocked
            // their own files makes no difference.
            Confirm::Relabel => match self.state.data {
                Data::Open | Data::Locked { .. } => fx.push(Effect::Relabel),
                Data::Opening => self.say(l.pick(DATA_OPENING), now),
                Data::Erased => self.say(l.pick(DATA_ERASED), now),
                Data::Closed | Data::Failed(_) => self.say(l.pick(["请先在「解密」中打开 data 分区", "請先在「解密」中開啟 data 分割區", "Open the data partition in Decrypt first"]), now),
            },
            // The data partition is reached through its own key whether or
            // not it is open, but not while it is being opened or erased.
            Confirm::Filesystem => {
                let row = self.fs_row().expect("the page of a partition shows one that was read");
                let data = row.name == DATA_PARTITION;
                match (&self.state.fs_action, &self.state.data) {
                    (_, Data::Opening) if data => self.say(l.pick(DATA_OPENING), now),
                    (FsAction::Repair, Data::Erased) if data => self.say(l.pick(DATA_ERASED), now),
                    (action, _) => fx.push(Effect::Filesystem { name: row.name.clone(), action: action.clone() }),
                }
            }
            Confirm::Flash => {
                if let Install::Choosing(choice) = &self.state.install {
                    let FlashTarget { name, place } = choice.partitions[choice.partition].clone();
                    fx.push(Effect::Flash { partition: name.clone(), place, pick: choice.pick });
                    self.state.install = Install::Flashing { partition: name, stage: None };
                    self.refresh();
                }
            }
            Confirm::SwitchSystem => {
                if let (Some(Ok(systems)), Some(pick)) = (&self.state.systems, self.state.system_pick)
                    && pick != starting(systems)
                {
                    fx.push(Effect::SwitchSystem(pick));
                    self.state.systems_work = Some(SystemsWork::Switching(pick));
                    self.state.system_pick = None;
                    self.refresh();
                }
            }
            Confirm::RemoveSystem => {
                fx.push(Effect::RemoveSystem);
                self.state.systems_work = Some(SystemsWork::Removing);
                self.state.asking_delete = false;
                self.state.system_pick = None;
                self.refresh();
            }
            Confirm::InstallSystem => {
                if let Install::SystemChoosing(choice) = &self.state.install {
                    fx.push(Effect::InstallSystem { size: choice.sizes[choice.size] });
                    self.state.install = Install::Systeming { name: choice.name.clone(), stage: None };
                    self.refresh();
                }
            }
        }
        fx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::testing::*;

    #[test]
    fn search_finds_by_pinyin_typed_on_its_own_keyboard() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Search, &f, &i);
        type_text(&mut app, &f, &i, "bf");
        assert_eq!(app.state.search, "bf");
        let opens: Vec<Action> = app.frame(&f, &i).hits.iter().map(|h| h.action).filter(|a| matches!(a, Action::Open(_))).collect();
        assert_eq!(opens, [Action::Open(Screen::Backup)]);
        tap(&mut app, &f, &i, Action::Type(Cap::Backspace));
        assert_eq!(app.state.search, "b");
        tap(&mut app, &f, &i, Action::ClearSearch);
        assert_eq!(app.state.search, "");
        type_text(&mut app, &f, &i, "xpsj");
        tap(&mut app, &f, &i, Action::Type(Cap::Enter));
        assert_eq!(app.screen(), Screen::ScreenOff, "return opens the first result");
        app.key(Key::Back, 0);
        assert_eq!(app.screen(), Screen::Search);
    }

    #[test]
    fn shift_makes_one_capital_and_the_layers_switch() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Search, &f, &i);
        tap(&mut app, &f, &i, Action::Type(Cap::Shift));
        type_text(&mut app, &f, &i, "ab");
        assert_eq!(app.state.search, "Ab");
        tap(&mut app, &f, &i, Action::Type(Cap::Show(Layer::Digits)));
        type_text(&mut app, &f, &i, "1");
        tap(&mut app, &f, &i, Action::Type(Cap::Show(Layer::Symbols)));
        type_text(&mut app, &f, &i, "|");
        tap(&mut app, &f, &i, Action::Type(Cap::Space));
        assert_eq!(app.state.search, "Ab1| ");
        tap(&mut app, &f, &i, Action::Type(Cap::Show(Layer::Letters)));
        assert_eq!(app.state.layer, Layer::Letters);
    }

    #[test]
    fn a_key_shows_held_down_while_touched() {
        let (f, i) = (fonts(), info());
        let mut app = opened(Screen::Search, &f, &i);
        let (x, y) = spot(&mut app, &f, &i, Action::Type(Cap::Char('q')));
        app.touch(Touch::Down(x, y), &f, 0);
        assert_eq!(app.state.key_held(), Some(Cap::Char('q')));
        app.touch(Touch::Up, &f, 0);
        assert_eq!(app.state.held, None);
        assert_eq!(app.state.search, "q");
    }
}
