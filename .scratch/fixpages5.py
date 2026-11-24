"""Put back the `keep` note the backup job page interpolates."""

import io
import subprocess

original = subprocess.run(
    ['git', 'show', '63af689:crates/skyward-ui/src/pages.rs'],
    capture_output=True, check=True,
).stdout.decode('utf-8')

lines = original.splitlines()
block = None
for i, line in enumerate(lines):
    if line.strip() == 'let keep = l.pick(match job.place {':
        depth = 0
        for j in range(i, len(lines)):
            depth += lines[j].count('{') - lines[j].count('}')
            depth += lines[j].count('(') - lines[j].count(')')
            if depth == 0 and j > i and lines[j].rstrip().endswith(';'):
                block = '\n'.join(lines[i:j + 1])
                break
        break
assert block, 'keep block not found'

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

anchor = '''    match &job.outcome {
        None => {
            let note = match (job.restore, job.at) {
'''
new = block + '''
    match &job.outcome {
        None => {
            let note = match (job.restore, job.at) {
'''
assert anchor in s, 'anchor not found'
assert 'let keep = l.pick(match job.place' not in s, 'already there'
s = s.replace(anchor, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('restored keep')
