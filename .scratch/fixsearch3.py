import io

p = 'crates/skyward-ui/src/search.rs'
s = io.open(p, encoding='utf-8').read()

old = '''    #[test]
    fn a_language_answers_to_its_own_keys_and_the_english_ones() {
        assert!(find("beifen", Lang::En).is_empty(), "pinyin is Chinese's own");
        for l in Lang::ALL {
            let found: Vec<Screen> = find("sideload", l).iter().map(|e| e.screen).collect();
            assert_eq!(found, [Screen::Install], "{l:?} finds installing from a computer by the name every language borrows");
        }
    }
'''
new = '''    #[test]
    fn a_key_finds_its_page_in_every_language() {
        // The keys are one list the three languages share, so a pinyin and an
        // English word find the same page whichever language is spoken.
        for l in Lang::ALL {
            let found: Vec<Screen> = find("sideload", l).iter().map(|e| e.screen).collect();
            assert_eq!(found, [Screen::Install], "{l:?} finds installing from a computer by the name every language borrows");
            let pinyin: Vec<Screen> = find("beifen", l).iter().map(|e| e.screen).collect();
            assert_eq!(pinyin, [Screen::Backup], "{l:?} finds the backup page by its pinyin");
        }
    }
'''
assert old in s, 'language keys test not found'
s = s.replace(old, new)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('language keys test rewritten')
