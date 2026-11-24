"""Put back the two words the clash and install pages read."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

# The clash page names the operation before it reads the sentence about it.
old = '''    let skip = l.pick(match clip.op {
        FileOp::Copy => said::SKIP_COPY,
        FileOp::Move => said::SKIP_MOVE,
    });
    let mut c = page(cx);
'''
new = '''    let skip = l.pick(match clip.op {
        FileOp::Copy => said::SKIP_COPY,
        FileOp::Move => said::SKIP_MOVE,
    });
    let verb = l.pick(match clip.op {
        FileOp::Copy => ["复制", "複製", "copy"],
        FileOp::Move => ["移动", "移動", "move"],
    });
    let mut c = page(cx);
'''
assert old in s, 'clash head not found'
s = s.replace(old, new, 1)

# The install page says the system was installed.
old = '''            let y = widgets::list(c, t, f, top, &[standing(installed, GREEN, Icon::Systems, &detail)]).bottom;
'''
new = '''            let installed = l.pick(["安装成功", "安裝成功", "Installed"]);
            let y = widgets::list(c, t, f, top, &[standing(installed, GREEN, Icon::Systems, &detail)]).bottom;
'''
assert old in s, 'installed line not found'
s = s.replace(old, new, 1)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('verb and installed restored')
