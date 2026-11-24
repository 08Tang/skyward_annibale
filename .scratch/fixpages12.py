"""Cover the resizing stage, the flash targets, and the upload detail."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

old = 'Some(Stage::Flashing { .. } | Stage::Script { .. } | Stage::Signature { .. } | Stage::Allocating { .. } | Stage::Root { .. } | Stage::Storing) | None =>'
new = 'Some(Stage::Resizing | Stage::Flashing { .. } | Stage::Script { .. } | Stage::Signature { .. } | Stage::Allocating { .. } | Stage::Root { .. } | Stage::Storing) | None =>'
assert old in s, 'update_stage fallback not found'
s = s.replace(old, new, 1)

# `pick` takes literals, and `tr!` formats them, so the counts go in the
# sentences themselves.
old = '''            let detail = l.pick([
                format!("已上传 {done} / {total} 字节"),
                format!("已上傳 {done} / {total} 位元組"),
                format!("{done} of {total} bytes sent"),
            ]);
'''
new = '''            let detail = tr!(l, "已上传 {done} / {total} 字节", "已上傳 {done} / {total} 位元組", "{done} of {total} bytes sent");
'''
assert old in s, 'upload detail not found'
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('pages fixed')

p2 = 'crates/skyward-ui/src/app/install.rs'
t = io.open(p2, encoding='utf-8').read()
old = 'size: 4096, partitions: vec!["vendor_boot"] }, PackageKind::Image),'
new = 'size: 4096, partitions: vec![Target { name: "vendor_boot".into(), place: Place::Logical }] }, PackageKind::Image),'
assert old in t, 'install test line not found'
t = t.replace(old, new, 1)
io.open(p2, 'w', encoding='utf-8', newline='\n').write(t)
print('install.rs test fixed')
