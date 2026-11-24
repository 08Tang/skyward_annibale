"""Separate the data partition's name from its index, which upstream named
`DATA_PART`."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

# The wipe page's own name for the partition.
old = 'const DATA_PART: Text = ["data 分区", "data 分割區", "data partition"];'
new = 'const DATA_PART_NAME: Text = ["data 分区", "data 分割區", "data partition"];'
assert old in s, 'local DATA_PART not found'
s = s.replace(old, new, 1)

# The wipe parts list reads it.
old = '[(DATA_PART, &[Wipe::AppData, Wipe::Media, Wipe::AppCaches, Wipe::SystemCaches, Wipe::Art, Wipe::Logs, Wipe::Updates])'
new = '[(DATA_PART_NAME, &[Wipe::AppData, Wipe::Media, Wipe::AppCaches, Wipe::SystemCaches, Wipe::Art, Wipe::Logs, Wipe::Updates])'
assert old in s, 'wipe list not found'
s = s.replace(old, new, 1)

# The restore page indexes the parts chosen by the app's numbering.
head, rest = s.split('use crate::app::{', 1)
inside, tail = rest.split('};', 1)
if 'DATA_PART,' not in inside and 'DATA_PART,' not in inside.replace('DATA_PARTITION', ''):
    inside = inside.rstrip()
    inside = inside + ' DATA_PART,' if inside.endswith(',') else inside + ', DATA_PART'
    s = head + 'use crate::app::{' + inside + '};' + tail

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('DATA_PART separated')
