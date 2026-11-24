"""Take the flash targets and the damage the way the merged APIs hand them."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

old = 'map(|(k, p)| Row::new(p).subtitle(fmt::partition_role(l, p)).trail(Trail::Check(k == choice.partition))).collect();'
new = 'map(|(k, p)| Row::new(p.name.as_str()).subtitle(fmt::partition_role(l, p)).trail(Trail::Check(k == choice.partition))).collect();'
assert old in s, 'partition row not found'
s = s.replace(old, new, 1)

old = 'Install::Finished { outcome, damage } => finished(&mut c, cx, top, outcome, damage.as_ref(), &mut out, &mut rows),'
new = 'Install::Finished { outcome, damage } => finished(&mut c, cx, top, outcome, damage, &mut out, &mut rows),'
assert old in s, 'finished call not found'
s = s.replace(old, new, 1)

old = 'y = widgets::footnote(&mut c, t, f, y, l.pick(note));'
new = 'y = widgets::footnote(&mut c, t, f, y, &note);'
assert old in s, 'note footnote not found'
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('three type errors fixed')
