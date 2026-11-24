"""Bring the two formatting tests to what the merged formatters write."""

import io

p = 'crates/skyward-ui/src/fmt.rs'
s = io.open(p, encoding='utf-8').read()

old = '        assert_eq!(date(Lang::En, built, china()), "8 May 2026");'
new = '        assert_eq!(date(Lang::En, built, china()), "May 8, 2026");'
assert old in s, 'date assertion not found'
s = s.replace(old, new, 1)

old = '        assert_eq!(list(Lang::En, ["ART cache", "App data"]), "ART cache, App data");'
new = '        assert_eq!(list(Lang::En, ["ART cache", "App data"]), "ART cache, app data", "only the first keeps its capital");'
assert old in s, 'list assertion not found'
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('two tests aligned')
