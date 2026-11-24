"""Give the time zone page the region rows upstream added."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

old = '''pub fn time_zone(cx: &Ctx) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let mut out = Vec::new();
    let top = t.m.content_top;
    let rows: Vec<Row> = TIME_ZONES.iter().enumerate().map(|(k, z)| Row::new(l.pick(z.name)).subtitle(l.pick(z.cities)).trail(Trail::Check(k == s.time_zone))).collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = zones.iter().map(|&at| Action::TimeZone(at)).collect();
    Frame::new(t, fit(c, t, list.bottom), hits(&list.rows, &picks), l.pick(region.name()), top)
}
'''

new = '''pub fn time_zone(cx: &Ctx) -> Frame {
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
'''

assert old in s, 'time_zone not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('time_zone rewritten with regions')
