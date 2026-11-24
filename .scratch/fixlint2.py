"""Drop what the merged pages no longer read."""

import io
import re


def read(p):
    return io.open(p, encoding='utf-8').read()


def write(p, s):
    io.open(p, 'w', encoding='utf-8', newline='\n').write(s)


def drop_item(src, name):
    lines = src.split('\n')
    for i, line in enumerate(lines):
        if re.match(r'(pub(\(crate\))? )?(fn|struct|const|enum) %s\b' % re.escape(name), line):
            start = i
            while start > 0 and (lines[start - 1].startswith('///') or lines[start - 1].startswith('#[')):
                start -= 1
            depth = 0
            end = None
            for j in range(i, len(lines)):
                depth += lines[j].count('{') - lines[j].count('}')
                if depth == 0 and j > i:
                    end = j
                    break
                if depth == 0 and lines[j].rstrip().endswith(';'):
                    end = j
                    break
            if end is None:
                raise SystemExit('unterminated: ' + name)
            if end + 1 < len(lines) and lines[end + 1].strip() == '':
                end += 1
            return '\n'.join(lines[:start] + lines[end + 1:])
    raise SystemExit('not found: ' + name)


p = 'crates/skyward-ui/src/pages.rs'
s = read(p)

# Words the merged settings page no longer names.
for name in ('ABOUT', 'SPECS', 'SCREEN_TIMEOUT', 'DONE', 'STOPPED', 'STOPPING', 'LICENSE_NOTICE'):
    s = drop_item(s, name)

# The file work's own word table, replaced by `texts::files`.
for name in ('FileWords', 'file_words'):
    s = drop_item(s, name)

# A share the progress card writes itself.
lines = s.split('\n')
for i, line in enumerate(lines):
    if re.match(r'    fn doing_share\(self, l: Lang, p: u32\) -> String \{', line):
        start = i
        while start > 0 and lines[start - 1].strip().startswith('///'):
            start -= 1
        depth = 0
        for j in range(i, len(lines)):
            depth += lines[j].count('{') - lines[j].count('}')
            if depth == 0 and j > i:
                end = j
                break
        if end + 1 < len(lines) and lines[end + 1].strip() == '':
            end += 1
        s = '\n'.join(lines[:start] + lines[end + 1:])
        break

# Imports the merged bodies no longer name.
s = s.replace('USB_DRIVE_UNREADABLE, ', '')
s = s.replace('USB_DRIVE_WANTED, ', '')
s = s.replace('    use texts::wipe::{part, what};\n', '')

# The clash page reads its verb from the same match it picks the skip from.
s = s.replace('''    let skip = l.pick(match clip.op {
        FileOp::Copy => said::SKIP_COPY,
        FileOp::Move => said::SKIP_MOVE,
    });
    let verb = l.pick(match clip.op {
        FileOp::Copy => ["复制", "複製", "copy"],
        FileOp::Move => ["移动", "移動", "move"],
    });
''', '''    let verb = l.pick(match clip.op {
        FileOp::Copy => ["复制", "複製", "copy"],
        FileOp::Move => ["移动", "移動", "move"],
    });
''')

# A reference the compiler dereferences at once.
s = s.replace('Row::new(&titles[k]).subtitle(&subs[k])', 'Row::new(titles[k]).subtitle(&subs[k])')

write(p, s)
print('pages.rs trimmed')
