"""The image words belong above the match that reads them."""

import io

p = 'crates/skyward-ui/src/fmt.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    let (name, holds) = match kind {
    const RECOVERY: Text = ["恢复模式镜像", "復原模式映像檔", "Recovery image"];
    const BOOT: Text = ["启动镜像", "啟動映像檔", "Boot image"];
'''
new = '''    const RECOVERY: Text = ["恢复模式镜像", "復原模式映像檔", "Recovery image"];
    const BOOT: Text = ["启动镜像", "啟動映像檔", "Boot image"];
    let (name, holds) = match kind {
'''

assert old in s, 'misplaced constants not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('image constants moved above the match')
