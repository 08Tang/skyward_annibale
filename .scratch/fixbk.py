"""Give the backup page back the row its parts list was missing."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

# The media partition's line went missing in the merge, so the list drew four
# rows for five parts and the fifth had no tap target at all.
old = '    let subs = [boot, system, data, modem];\n'
new = '''    let media = l.pick(match s.data == Data::Open {
        true => ["内部存储中的照片、视频和音乐", "內部儲存空間中的相片、影片和音樂", "Photos, videos and music in internal storage"],
        false => ["内部存储中的照片、视频和音乐，需先完成「解密」", "內部儲存空間中的相片、影片和音樂，需先完成「解密」", "Photos, videos and music in internal storage. Decrypt first"],
    });
    let subs = [boot, system, data, media, modem];
'''
assert old in s, 'subs line not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('media row restored')
