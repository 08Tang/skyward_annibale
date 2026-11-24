"""Point two lines at the variables this page actually has."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

old = 'widgets::paragraph(&mut c, t, f, sign_cy + m.dp(88.0) + more, why);'
new = 'widgets::paragraph(&mut c, t, f, sign_cy + m.dp(88.0), why);'
assert old in s, 'more line not found'
s = s.replace(old, new, 1)

old = 'widgets::confirm(&mut c, t, f, bottom + t.m.dp(28.0), label, danger, s.confirm);'
new = 'widgets::confirm(&mut c, t, f, picks.bottom + t.m.dp(28.0), label, danger, s.confirm);'
assert old in s, 'bottom line not found'
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('fixed more and bottom')
