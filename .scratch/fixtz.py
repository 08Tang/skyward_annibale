"""Rewrite the time zone pages for the zone records upstream introduced."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    let rows: Vec<Row> = TIME_ZONES.iter().enumerate().map(|(k, &(name, cities, _))| Row::new(l.pick(name)).subtitle(l.pick(cities)).trail(Trail::Check(k == s.time_zone))).collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = zones.iter().map(|&at| Action::TimeZone(at)).collect();
    Frame::new(t, fit(c, t, list.bottom), hits(&list.rows, &picks), l.pick(region.name()), top)
}
'''

new = '''    let rows: Vec<Row> = TIME_ZONES.iter().enumerate().map(|(k, z)| Row::new(l.pick(z.name)).subtitle(l.pick(z.cities)).trail(Trail::Check(k == s.time_zone))).collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = zones.iter().map(|&at| Action::TimeZone(at)).collect();
    Frame::new(t, fit(c, t, list.bottom), hits(&list.rows, &picks), l.pick(region.name()), top)
}

/// The zones of one part of the world, as its row on the time zone page opens
/// them.
pub fn time_zones(cx: &Ctx, region: Region) -> Frame {
    let (t, f, s, l) = (cx.t, cx.f, cx.state, cx.state.lang);
    let mut c = long_page(cx);
    let top = t.m.content_top;
    let zones: Vec<usize> = (0..TIME_ZONES.len()).filter(|&at| TIME_ZONES[at].region == Some(region)).collect();
    let rows: Vec<Row> = zones
        .iter()
        .map(|&at| Row::new(l.pick(TIME_ZONES[at].name)).subtitle(l.pick(TIME_ZONES[at].cities)).trail(Trail::Check(at == s.time_zone)))
        .collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = zones.iter().map(|&at| Action::TimeZone(at)).collect();
    Frame::new(t, fit(c, t, list.bottom), hits(&list.rows, &picks), l.pick(region.name()), top)
}
'''

assert old in s, 'time_zone body not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('time_zone rewritten, time_zones added')
