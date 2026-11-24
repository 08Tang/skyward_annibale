import io

p = 'crates/skyward-ui/src/fmt.rs'
s = io.open(p, encoding='utf-8').read()

old = '''        assert_eq!(date(Lang::Hant, built, china()), "2026\u5e745\u67088\u65e5");
        assert_eq!(date(Lang::Tr, built, china()), "8 May\u0131s 2026");
        assert_eq!(date(Lang::Ru, built, china()), "8 \u043c\u0430\u044f 2026 \u0433.");
        assert_eq!(date(Lang::De, built, china()), "8. Mai 2026");
        assert_eq!(date(Lang::Fr, built, china()), "8 mai 2026");
        assert_eq!(date(Lang::Pl, built, china()), "8 maja 2026");
        assert_eq!(date(Lang::Ja, built, china()), "2026\u5e745\u67088\u65e5");
'''
new = '''        assert_eq!(date(Lang::Hant, built, china()), "2026\u5e745\u67088\u65e5");
        assert_eq!(date(Lang::En, built, china()), "8 May 2026");
'''
assert old in s, 'date block not found'
s = s.replace(old, new)

old = '''        assert_eq!([items(Lang::En, 1), items(Lang::En, 2)], ["1 item", "2 items"]);
        assert_eq!([items(Lang::Fr, 0), items(Lang::Fr, 1), items(Lang::Fr, 2)], ["0 \u00e9l\u00e9ment", "1 \u00e9l\u00e9ment", "2 \u00e9l\u00e9ments"], "none is one in French");
        let russian = [1, 2, 5, 11, 21, 22, 25, 111].map(|n| items(Lang::Ru, n));
        assert_eq!(russian, ["1 \u043e\u0431\u044a\u0435\u043a\u0442", "2 \u043e\u0431\u044a\u0435\u043a\u0442\u0430", "5 \u043e\u0431\u044a\u0435\u043a\u0442\u043e\u0432", "11 \u043e\u0431\u044a\u0435\u043a\u0442\u043e\u0432", "21 \u043e\u0431\u044a\u0435\u043a\u0442", "22 \u043e\u0431\u044a\u0435\u043a\u0442\u0430", "25 \u043e\u0431\u044a\u0435\u043a\u0442\u043e\u0432", "111 \u043e\u0431\u044a\u0435\u043a\u0442\u043e\u0432"]);
        let polish = [1, 2, 5, 12, 21, 22].map(|n| items(Lang::Pl, n));
        assert_eq!(polish, ["1 element", "2 elementy", "5 element\u00f3w", "12 element\u00f3w", "21 element\u00f3w", "22 elementy"]);
        assert_eq!([items(Lang::Ja, 1), items(Lang::Ja, 68)], ["1 \u9805\u76ee", "68 \u9805\u76ee"], "one form where the language has one");
'''
new = '''        assert_eq!([items(Lang::En, 1), items(Lang::En, 2)], ["1 item", "2 items"]);
'''
assert old in s, 'items block not found'
s = s.replace(old, new)

old = '''        assert_eq!(list(Lang::De, ["ART-Cache", "App-Daten"]), "ART-Cache, App-Daten", "German keeps its nouns' capitals");
        assert_eq!(list(Lang::Ru, ["\u041a\u044d\u0448 ART", "\u0414\u0430\u043d\u043d\u044b\u0435 \u043f\u0440\u0438\u043b\u043e\u0436\u0435\u043d\u0438\u0439"]), "\u041a\u044d\u0448 ART, \u0434\u0430\u043d\u043d\u044b\u0435 \u043f\u0440\u0438\u043b\u043e\u0436\u0435\u043d\u0438\u0439");
        assert_eq!(list(Lang::Tr, ["ART \u00f6nbelle\u011fi", "\u0130ndirilenler", "Is\u0131 kay\u0131tlar\u0131"]), "ART \u00f6nbelle\u011fi, indirilenler, \u0131s\u0131 kay\u0131tlar\u0131", "by Turkish's own small letters");
        assert_eq!(list(Lang::Ja, ["ART \u30ad\u30e3\u30c3\u30b7\u30e5", "\u30a2\u30d7\u30ea\u306e\u30c7\u30fc\u30bf"]), "ART \u30ad\u30e3\u30c3\u30b7\u30e5\u3001\u30a2\u30d7\u30ea\u306e\u30c7\u30fc\u30bf");
        assert_eq!(users(Lang::Tr, &[0, 10]), "cihaz sahibi ve kullan\u0131c\u0131 10");
        assert_eq!(users(Lang::Ru, &[0, 10, 999]), "\u0432\u043b\u0430\u0434\u0435\u043b\u0435\u0446, \u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u044c 10 \u0438 \u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u044c 999");
        assert_eq!(users(Lang::Fr, &[10]), texts::users::user(Lang::Fr, 10));
'''
new = '''        assert_eq!(list(Lang::En, ["ART cache", "App data"]), "ART cache, App data");
'''
assert old in s, 'list block not found'
s = s.replace(old, new)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('fmt.rs tests trimmed')
