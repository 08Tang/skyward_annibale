import io

p = 'crates/skyward-ui/src/search.rs'
s = io.open(p, encoding='utf-8').read()

# The keys are one list the three languages share: they are the pinyin and the
# English a page answers to, not a sentence that is translated.
old = '    pub keys: Text,\n'
new = '    pub keys: &\'static [&\'static str],\n'
assert old in s, 'keys field not found'
s = s.replace(old, new)

old = 'const fn entry(title: Text, place: Text, screen: Screen, keys: Text) -> Entry {\n'
new = 'const fn entry(title: Text, place: Text, screen: Screen, keys: &\'static [&\'static str]) -> Entry {\n'
assert old in s, 'entry signature not found'
s = s.replace(old, new)

old = 'Entry { title: same(name), place, screen, keys: same("") }\n'
new = 'Entry { title: same(name), place, screen, keys: &[] }\n'
assert old in s, 'partition keys not found'
s = s.replace(old, new)

old = '''        let keys = || l.pick(e.keys).split(' ').chain(Lang::En.pick(e.keys).split(' '));
'''
new = '''        let keys = || e.keys.iter().copied();
'''
assert old in s, 'rank keys not found'
s = s.replace(old, new)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('search.rs keys retyped')
