"""Give confirm_label back its own INSTALL word, and import what the zone page
reads."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    let (label, danger) = match which {
        // 锁屏自己画它，这张只给按住电源键时的滑块用。
'''
new = '''    const INSTALL: Text = ["滑动以安装", "滑動以安裝", "Slide to install"];
    let (label, danger) = match which {
        // 锁屏自己画它，这张只给按住电源键时的滑块用。
'''
assert old in s, 'confirm_label head not found'
s = s.replace(old, new, 1)

# The zone page reads the listed zone and the universal one from the app.
head, rest = s.split('use crate::app::{', 1)
inside, tail = rest.split('};', 1)
for name in ('UTC', 'listed'):
    if name not in inside:
        inside = inside.rstrip()
        inside = inside + ' %s,' % name if inside.endswith(',') else inside + ', %s' % name
s = head + 'use crate::app::{' + inside + '};' + tail

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('INSTALL restored and zone imports added')
