"""Read the refusal's words from the locale, as the other refusals do."""

import io

p = 'crates/skyward-ui/src/fmt.rs'
s = io.open(p, encoding='utf-8').read()

old = '''        BackupRefusal::OlderAndroid { was, now } => tr!(
            l,
            "该备份在 Android {was} 上创建，本机系统为 Android {now}，无法读取其中的 data 分区。",
            "該備份在 Android {was} 上建立，本機系統為 Android {now}，無法讀取其中的 data 分割區。",
            "The backup was made on Android {was}, and this phone runs Android {now}, which can't read its data partition."
        ),
'''
new = '        BackupRefusal::OlderAndroid { was, now } => refused::older_android(l, was, now),\n'
assert old in s, 'OlderAndroid arm not found'
s = s.replace(old, new, 1)

# The arm reads the locale's own words, so the import comes back.
old = '''pub fn backup_refusal(l: Lang, r: &BackupRefusal, restore: bool) -> String {
    match r {
'''
new = '''pub fn backup_refusal(l: Lang, r: &BackupRefusal, restore: bool) -> String {
    use texts::backup::refused;
    match r {
'''
assert old in s, 'backup_refusal head not found'
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('OlderAndroid reads the locale')
