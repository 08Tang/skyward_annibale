"""Let the newer-Android refusal stand in place of the data reset one, as
upstream has it."""

import io

# The core refusal: nothing constructs `DataReset` any more.
p = 'crates/skyward/src/backup.rs'
s = io.open(p, encoding='utf-8').read()
old = '''    /// The data partition has been made afresh since the backup.
    DataReset,
'''
assert old in s, 'DataReset variant not found'
s = s.replace(old, '', 1)
old = '            Refusal::DataReset => write!(f, "the data partition has been made afresh since the backup"),\n'
assert old in s, 'DataReset display not found'
s = s.replace(old, '', 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('DataReset gone from the core')

# The page's own copy of the refusal, and the arm that mapped it.
p = 'crates/skyward-ui/src/app/backup.rs'
s = io.open(p, encoding='utf-8').read()
lines = s.split('\n')
for i, line in enumerate(lines):
    if line.strip() == 'DataReset,':
        start = i
        while start > 0 and lines[start - 1].strip().startswith('///'):
            start -= 1
        s = '\n'.join(lines[:start] + lines[i + 1:])
        break
else:
    raise SystemExit('DataReset not in the page copy')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('DataReset gone from the page copy')

p = 'crates/skyward/src/ui.rs'
s = io.open(p, encoding='utf-8').read()
old = '            backup::Refusal::DataReset => BackupRefusal::DataReset,\n'
assert old in s, 'ui mapping not found'
s = s.replace(old, '', 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('DataReset gone from the mapping')

p = 'crates/skyward-ui/src/fmt.rs'
s = io.open(p, encoding='utf-8').read()
old = '''        BackupRefusal::DataReset => l
            .pick([
                "备份后 data 分区已重新创建，例如恢复过出厂设置，该备份中的 data 分区无法恢复。",
                "備份後 data 分割區已重新建立，例如曾恢復原廠設定，這份備份中的 data 分割區無法還原。",
                "The data partition was made anew after the backup, as a factory reset does, so the backup's data partition can't be restored.",
            ])
            .into(),
'''
assert old in s, 'fmt arm not found'
s = s.replace(old, '', 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('DataReset gone from the words')
