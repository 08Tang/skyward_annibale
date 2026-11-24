import io
import subprocess

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

# These constants were defined beside the page titles before the merge and
# are still named by the code below; upstream moved its own into `texts`,
# and this project keeps its own here.
original = subprocess.run(
    ['git', 'show', '63af689:crates/skyward-ui/src/pages.rs'],
    capture_output=True, check=True,
).stdout.decode('utf-8')

names = [
    'MODEL', 'DATA_PART', 'UNREADABLE', 'REFLASHING', 'READING_FILE',
    'RUN_ON_COMPUTER', 'GETTING_READY', 'WIPING', 'KEEP_POWER', 'VERIFIED',
    'VERIFYING', 'WAITING_TURN', 'RESTORE_FAILED', 'ACTIONS', 'CHECKING',
    'DOWNLOADING', 'CHECKING_PART', 'NOTES', 'CANCEL', 'RENAME',
    'LICENSE_TERMS', 'POWERS_OFF_WHEN_DONE', 'REBOOTS_WHEN_DONE',
]

wanted = []
for name in names:
    for line in original.splitlines():
        if line.startswith('const %s: Text = ' % name):
            wanted.append(line)
            break
    else:
        raise SystemExit('not in the original: ' + name)

missing = [line for line in wanted if line not in s]
if not missing:
    print('all present already')
else:
    anchor = 'const UPDATE: Text = ["系统更新", "系統更新", "System update"];\n'
    assert anchor in s, 'anchor not found'
    block = '\n'.join(missing) + '\n'
    s = s.replace(anchor, anchor + '\n' + block, 1)
    io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
    print('restored', len(missing), 'constants')
