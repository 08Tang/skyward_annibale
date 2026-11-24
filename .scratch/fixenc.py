"""Let the entry that says how the partition is encrypted win over one that
only says where the key is kept."""

import io

p = 'crates/skyward-crypto/src/encryption.rs'
s = io.open(p, encoding='utf-8').read()

old = '''pub fn whole(entries: &[Entry]) -> Result<Option<(&Entry, Encryption)>> {
    for entry in entries {
        if let Some(encryption) = Encryption::of(entry)? {
            return Ok(Some((entry, encryption)));
        }
    }
    Ok(None)
}
'''

new = '''pub fn whole(entries: &[Entry]) -> Result<Option<(&Entry, Encryption)>> {
    // An entry that names how the partition is encrypted says more than one
    // that only says where the key is kept, and is the one to read it from:
    // a phone's table names the partition once per filesystem, and only one
    // of those need say it.
    let says_how = entries.iter().find(|e| e.extra("metadata_encryption").is_some());
    let says_where = entries.iter().find(|e| e.extra("keydirectory").is_some());
    for entry in says_how.into_iter().chain(says_where) {
        if let Some(encryption) = Encryption::of(entry)? {
            return Ok(Some((entry, encryption)));
        }
    }
    Ok(None)
}
'''

assert old in s, 'whole not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('whole prefers the entry that says how')
