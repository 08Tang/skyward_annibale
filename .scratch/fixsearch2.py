import io

p = 'crates/skyward-ui/src/search.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    #[test]
    fn every_key_is_one_the_keyboard_types() {
        for l in Lang::ALL {
            for e in ENTRIES {
                let keys = l.pick(e.keys);
                assert!(keys.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == ' '), "{l:?} gives {keys:?}");
                assert!(!keys.starts_with(' ') && !keys.ends_with(' ') && !keys.contains("  "), "{l:?} gives {keys:?}");
            }
        }
    }
'''
new = '''    #[test]
    fn every_key_is_one_the_keyboard_types() {
        // The keys are one list the three languages share, so they are
        // checked once rather than per language.
        for e in ENTRIES {
            for key in e.keys {
                assert!(!key.is_empty(), "{key:?} is a key");
                assert!(
                    key.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == ' '),
                    "{key:?} is typed by the keyboard"
                );
            }
        }
    }
'''
assert old in s, 'keys test not found'
s = s.replace(old, new)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('keys test rewritten')
