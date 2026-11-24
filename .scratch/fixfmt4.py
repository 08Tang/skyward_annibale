"""The refusal tests take the zone the way the function now asks for it."""

import io
import re

p = 'crates/skyward-ui/src/fmt.rs'
s = io.open(p, encoding='utf-8').read()

# Only inside the two refusal tests: `pc, 0)` becomes `pc, china())`.
before = s.count(', pc, 0)') + s.count(', file, 0)')
s = s.replace(', pc, 0)', ', pc, china())')
s = s.replace(', file, 0)', ', file, china())')
after = s.count(', pc, 0)') + s.count(', file, 0)')
print('replaced', before - after, 'zone arguments')

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
