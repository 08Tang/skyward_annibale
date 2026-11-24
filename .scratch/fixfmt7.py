"""Cover the three image kinds upstream added, and hand the damage on."""

import io

p = 'crates/skyward-ui/src/fmt.rs'
s = io.open(p, encoding='utf-8').read()

old = '''        ImageKind::Vbmeta => (["vbmeta 镜像", "vbmeta 映像檔", "vbmeta image"], ["包含启动验证数据", "包含啟動驗證資料", "Holds verified boot data"]),
    };
'''
new = '''        ImageKind::Vbmeta => (["vbmeta 镜像", "vbmeta 映像檔", "vbmeta image"], ["包含启动验证数据", "包含啟動驗證資料", "Holds verified boot data"]),
        ImageKind::Super => (["super 镜像", "super 映像檔", "super image"], ["包含全部逻辑分区", "包含全部邏輯分割區", "Holds every logical partition"]),
        ImageKind::Filesystem(filesystem) => {
            return (l.pick(PARTITION), texts::image::holds::filesystem(l, filesystem.name()));
        }
        ImageKind::Other => (PARTITION, ["内容不明", "內容不明", "Holds what couldn't be identified"]),
    };
'''
assert old in s, 'image_kind tail not found'
s = s.replace(old, new, 1)

# A word for a partition whose contents say nothing of where it goes.
if 'const PARTITION: Text' not in s:
    anchor = '/// What a partition an image can go to is for.\n'
    assert anchor in s, 'partition_role anchor not found'
    s = s.replace(anchor, 'const PARTITION: Text = ["分区镜像", "分割區映像檔", "Partition image"];\n\n' + anchor, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('image kinds covered')

p2 = 'crates/skyward-ui/src/pages.rs'
t = io.open(p2, encoding='utf-8').read()
old = 'Install::Finished { outcome, damage } => finished(&mut c, cx, top, outcome, damage, &mut out, &mut rows),'
new = 'Install::Finished { outcome, damage } => finished(&mut c, cx, top, outcome, *damage, &mut out, &mut rows),'
assert old in t, 'finished call not found'
t = t.replace(old, new, 1)
io.open(p2, 'w', encoding='utf-8', newline='\n').write(t)
print('damage passed by value')
