"""Pass the clock's zone the way the formatting functions now take it."""

import io
import re

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

# Where a formatter is given the zone, it is the one the clock keeps, which is
# the installed system's where the phone follows it.
pairs = [
    ('fmt::datetime(l, a.updated, s.time_zone)', 'fmt::datetime(l, a.updated, s.zone())'),
    ('fmt::date(l, file.made, s.time_zone)', 'fmt::date(l, file.made, s.zone())'),
    ('&entries[i], folder.listed, cx.state.time_zone, chosen)', '&entries[i], folder.listed, cx.state.zone(), chosen)'),
]

for old, new in pairs:
    if old in s:
        s = s.replace(old, new)
        print('fixed:', old[:50])
    else:
        print('not found:', old[:50])

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
