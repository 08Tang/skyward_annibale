"""Put back the helpers the merge dropped from pages.rs."""

import io
import re
import subprocess

original = subprocess.run(
    ['git', 'show', '63af689:crates/skyward-ui/src/pages.rs'],
    capture_output=True, check=True,
).stdout.decode('utf-8')


def grab(name):
    lines = original.splitlines()
    for i, line in enumerate(lines):
        if re.match(r'(pub(\(crate\))? )?fn %s\b' % name, line):
            start = i
            while start > 0 and (lines[start - 1].startswith('///') or lines[start - 1].startswith('#[')):
                start -= 1
            depth = 0
            for j in range(i, len(lines)):
                depth += lines[j].count('{') - lines[j].count('}')
                if depth == 0 and j > i:
                    return '\n'.join(lines[start:j + 1])
            raise SystemExit('unterminated: ' + name)
    raise SystemExit('not in the original: ' + name)


p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

missing = []
for name in ('capitalized',):
    if re.search(r'fn %s\b' % name, s):
        print('present:', name)
    else:
        missing.append(grab(name))
        print('restoring:', name)

if missing:
    anchor = '/// Saving the logs, as a bug report asks for them.\n'
    assert anchor in s, 'anchor not found'
    s = s.replace(anchor, '\n\n'.join(missing) + '\n\n' + anchor, 1)
    io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
    print('restored', len(missing))
