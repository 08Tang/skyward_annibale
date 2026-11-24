"""Restore the confirm label's own INSTALL, import MEDIA_PART, and make the
zone page self-consistent."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

# `INSTALL` is a constant of `confirm_label` itself, not of the module.
old = '''pub fn confirm_label(which: Confirm, s: &State) -> (&'static str, bool) {
    let (label, danger) = match which {
'''
new = '''pub fn confirm_label(which: Confirm, s: &State) -> (&'static str, bool) {
    const INSTALL: Text = ["滑动以安装", "滑動以安裝", "Slide to install"];
    let (label, danger) = match which {
'''
if 'const INSTALL: Text' not in s:
    assert old in s, 'confirm_label head not found'
    s = s.replace(old, new, 1)
    print('restored INSTALL inside confirm_label')
else:
    print('INSTALL already present')

# `MEDIA_PART` lives with the other backup part indexes.
head, rest = s.split('use crate::app::{', 1)
inside, tail = rest.split('};', 1)
if 'MEDIA_PART' not in inside:
    inside = inside.rstrip()
    inside = inside + ' MEDIA_PART,' if inside.endswith(',') else inside + ', MEDIA_PART'
    s = head + 'use crate::app::{' + inside + '};' + tail
    print('imported MEDIA_PART')

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
