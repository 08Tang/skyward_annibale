"""List a region's zones by how far they are from UTC, as upstream does."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    let zones: Vec<usize> = (0..TIME_ZONES.len()).filter(|&at| TIME_ZONES[at].region == Some(region)).collect();
    let rows: Vec<Row> = zones
        .iter()
        .map(|&at| Row::new(l.pick(TIME_ZONES[at].name)).subtitle(l.pick(TIME_ZONES[at].cities)).trail(Trail::Check(at == s.time_zone)))
        .collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = zones.iter().map(|&at| Action::TimeZone(at)).collect();
    Frame::new(t, fit(c, t, list.bottom), hits(&list.rows, &picks), l.pick(region.name()), top)
'''

new = '''    let mut zones: Vec<usize> = (0..TIME_ZONES.len()).filter(|&at| TIME_ZONES[at].region == Some(region)).collect();
    zones.sort_by_key(|&at| fmt::utc_offset_seconds(s.now, fmt::rules(at)));
    let lines: Vec<String> = zones.iter().map(|&at| format!("{} · {}", fmt::utc_offset(s.now, fmt::rules(at)), l.pick(TIME_ZONES[at].cities))).collect();
    let kept = s.zone_listed();
    let rows: Vec<Row> = zones.iter().zip(&lines).map(|(&at, line)| Row::new(l.pick(TIME_ZONES[at].name)).subtitle(line).trail(Trail::Check(kept == Some(at)))).collect();
    let list = widgets::list(&mut c, t, f, top, &rows);
    let picks: Vec<Action> = zones.iter().map(|&at| Action::TimeZone(at)).collect();
    Frame::new(t, fit(c, t, list.bottom), hits(&list.rows, &picks), l.pick(region.name()), top)
'''

assert old in s, 'time_zones body not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('time_zones sorted by offset')
