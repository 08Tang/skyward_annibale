"""Let the generated texts keep the entries this project's pages don't read."""

import io

p = 'crates/skyward-ui/src/texts.rs'
s = io.open(p, encoding='utf-8').read()

old = 'include!(concat!(env!("OUT_DIR"), "/texts.rs"));'
new = '''// The file holds every text the languages carry, and the pages of this
// project read the ones they say: the rest are there for the pages to come,
// and are no dead weight to warn of.
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/texts.rs"));'''

assert old in s, 'include not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('dead code allowed in texts')
