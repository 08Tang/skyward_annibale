"""Drop what the merged pages no longer read, and the borrows they don't need."""

import io
import re
import subprocess


def read(p):
    return io.open(p, encoding='utf-8').read()


def write(p, s):
    io.open(p, 'w', encoding='utf-8', newline='\n').write(s)


def drop_item(src, name):
    """The item and the doc comment over it, gone."""
    lines = src.split('\n')
    for i, line in enumerate(lines):
        if re.match(r'(pub(\(crate\))? )?(fn|struct|const|enum) %s\b' % re.escape(name), line):
            start = i
            while start > 0 and (lines[start - 1].startswith('///') or lines[start - 1].startswith('#[')):
                start -= 1
            depth = 0
            for j in range(i, len(lines)):
                depth += lines[j].count('{') - lines[j].count('}')
                if depth == 0 and j > i:
                    end = j
                    break
                if depth == 0 and lines[j].rstrip().endswith(';'):
                    end = j
                    break
            else:
                raise SystemExit('unterminated: ' + name)
            # Swallow the blank line that followed it.
            if end + 1 < len(lines) and lines[end + 1].strip() == '':
                end += 1
            return '\n'.join(lines[:start] + lines[end + 1:])
    raise SystemExit('not found: ' + name)


p = 'crates/skyward-ui/src/fmt.rs'
s = read(p)

# Imports the merged bodies no longer name.
s = s.replace('    use texts::backup::refused;\n', '')
s = s.replace('    use texts::wipe::{failed, stopped};\n', '    use texts::wipe::failed;\n')
s = s.replace('    use texts::image::{self, holds};\n', '')
s = s.replace('    use texts::install::failed;\n', '')

# Helpers only the old page code called.
for name in ('before_characters', 'sent'):
    s = drop_item(s, name)

# `in_sentence` took the language for a rule the three now share.
s = s.replace('fn in_sentence(l: Lang, name: &str) -> String {', 'fn in_sentence(name: &str) -> String {')
s = s.replace('in_sentence(l, ', 'in_sentence(')

write(p, s)
print('fmt.rs trimmed')
