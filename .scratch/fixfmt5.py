"""Cover the refusal upstream added for a backup made on a newer Android."""

import io

p = 'crates/skyward-ui/src/fmt.rs'
s = io.open(p, encoding='utf-8').read()

old = '''        BackupRefusal::Nothing => l
            .pick([
                "所选内容在这台手机上没有可取的部分，没有备份任何东西。",
                "所選內容在這台手機上沒有可取的部分，沒有備份任何東西。",
                "Nothing that was chosen is on this phone to take, so nothing was backed up.",
            ])
            .into(),
    }
}
'''

new = '''        BackupRefusal::Nothing => l
            .pick([
                "所选内容在这台手机上没有可取的部分，没有备份任何东西。",
                "所選內容在這台手機上沒有可取的部分，沒有備份任何東西。",
                "Nothing that was chosen is on this phone to take, so nothing was backed up.",
            ])
            .into(),
        BackupRefusal::OlderAndroid { was, now } => tr!(
            l,
            "该备份在 Android {was} 上创建，本机系统为 Android {now}，无法读取其中的 data 分区。",
            "該備份在 Android {was} 上建立，本機系統為 Android {now}，無法讀取其中的 data 分割區。",
            "The backup was made on Android {was}, and this phone runs Android {now}, which can't read its data partition."
        ),
    }
}
'''

assert old in s, 'backup_refusal tail not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('OlderAndroid covered')
