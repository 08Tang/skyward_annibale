"""Restore the words and the login variable the update failure still names."""

import io
import re
import subprocess

original = subprocess.run(
    ['git', 'show', '63af689:crates/skyward-ui/src/fmt.rs'],
    capture_output=True, check=True,
).stdout.decode('utf-8')
orig_lines = original.splitlines()

p = 'crates/skyward-ui/src/fmt.rs'
s = io.open(p, encoding='utf-8').read()

# Module-level words.
wanted = []
for name in ('UNEXPECTED', 'LOGIN'):
    if re.search(r'^const %s\b' % name, s, re.M):
        print('present:', name)
        continue
    for line in orig_lines:
        if line.startswith('const %s: Text = ' % name):
            wanted.append(line)
            print('restoring:', name)
            break
    else:
        raise SystemExit('not in the original: ' + name)

if wanted:
    anchor = '/// The rules of each of [`TIME_ZONES`], read once from the ones the\n'
    assert anchor in s, 'anchor not found'
    s = s.replace(anchor, '\n'.join(wanted) + '\n\n' + anchor, 1)

# `RECOVERY` and `BOOT` are constants of the function that names an image.
local = []
for name in ('RECOVERY', 'BOOT'):
    if re.search(r'const %s: Text' % name, s):
        print('present:', name)
        continue
    for line in orig_lines:
        if line.strip().startswith('const %s: Text = ' % name):
            local.append(line)
            print('restoring local:', name)
            break
    else:
        raise SystemExit('not in the original: ' + name)

if local:
    anchor = '        ImageKind::Boot { kernel: true, ramdisk: Ramdisk::Recovery } => (RECOVERY,'
    assert anchor in s, 'image arm not found'
    s = s.replace(anchor, '\n'.join(local) + '\n' + anchor, 1)

# The update failure reads the joined network's name once, above the match.
old = '''    match why {
        Unanswered::Login => tr!(l, '''
new = '''    let login = l.pick(LOGIN);
    match why {
        Unanswered::Login => tr!(l, '''
if 'let login = l.pick(LOGIN);' not in s:
    assert old in s, 'login match not found'
    s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('fmt.rs words restored')
