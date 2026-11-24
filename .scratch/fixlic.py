"""Clear the tail of the licence note the trim left behind."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

start = s.find('发布 Skyward 是希望它有用，但不附带任何担保')
assert start != -1, 'leftover text not found'
end = s.find('];\n', start)
assert end != -1, 'closing bracket not found'
s = s[:start] + s[end + 3:]

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('licence tail cleared')
