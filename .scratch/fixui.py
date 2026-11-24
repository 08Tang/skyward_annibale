"""Make the zone type public, and hand the clock's zone to the formatters."""

import io

p = 'crates/skyward-ui/src/fmt.rs'
s = io.open(p, encoding='utf-8').read()

old = 'use jiff::tz::TimeZone;'
new = 'pub use jiff::tz::TimeZone;'
assert old in s, 'TimeZone import not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('TimeZone made public')

p2 = 'crates/skyward/src/ui.rs'
t = io.open(p2, encoding='utf-8').read()

pairs = [
    ('skyward_ui::fmt::date(self.app.state.lang, unix_now()?, self.app.state.time_zone)',
     'skyward_ui::fmt::date(self.app.state.lang, unix_now()?, self.app.state.zone())'),
    ('skyward_ui::fmt::stamp(unix_now()?, self.app.state.time_zone)',
     'skyward_ui::fmt::stamp(unix_now()?, self.app.state.zone())'),
]
for old, new in pairs:
    assert old in t, 'not found: ' + old[:60]
    t = t.replace(old, new, 1)

io.open(p2, 'w', encoding='utf-8', newline='\n').write(t)
print('ui.rs zone calls fixed')
