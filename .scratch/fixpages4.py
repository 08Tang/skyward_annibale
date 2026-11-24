"""Put back job_words and the words beside it, and point part_state at them."""

import io
import re
import subprocess

original = subprocess.run(
    ['git', 'show', '63af689:crates/skyward-ui/src/pages.rs'],
    capture_output=True, check=True,
).stdout.decode('utf-8')


def grab_item(start_pattern, from_lines):
    lines = from_lines
    for i, line in enumerate(lines):
        if re.match(start_pattern, line):
            start = i
            while start > 0 and (lines[start - 1].startswith('///') or lines[start - 1].startswith('#[')):
                start -= 1
            depth = 0
            for j in range(i, len(lines)):
                depth += lines[j].count('{') - lines[j].count('}')
                if depth == 0 and j > i:
                    return '\n'.join(lines[start:j + 1])
            raise SystemExit('unterminated')
    raise SystemExit('not found: ' + start_pattern)


p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()
lines = s.split('\n')

wanted = []
if not re.search(r'fn job_words\b', s):
    wanted.append(grab_item(r'fn job_words\b', original.splitlines()))
if not re.search(r'^const DONE: Text', s, re.M):
    for line in original.splitlines():
        if line.startswith('const DONE: Text'):
            wanted.append(line)
            break

assert wanted, 'nothing to restore'
anchor = '/// Where part `k` of a backup or restore stands, as its row says it.\n'
assert anchor in s, 'anchor not found'
s = s.replace(anchor, '\n'.join(wanted) + '\n\n' + anchor, 1)

# `part_state` reads the two words from `job_words`, as it did before.
old = '''    use std::cmp::Ordering;
    use texts::backup::job as said;
    let wrote = Pass::writing(job.restore).done();
'''
new = '''    use std::cmp::Ordering;
    use texts::backup::job as said;
    let (did, doing) = job_words(job.restore);
    let wrote = Pass::writing(job.restore).done();
'''
assert old in s, 'part_state head not found'
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('restored job_words and DONE')
