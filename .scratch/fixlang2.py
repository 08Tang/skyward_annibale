import io

p = 'crates/skyward-ui/src/lang.rs'
s = io.open(p, encoding='utf-8').read()

# The locale test names twelve languages this project does not ship, so it
# keeps only the three it does.
old = '''        assert_eq!(of("en-US"), Lang::En);
        assert_eq!(of("tr-TR"), Lang::Tr);
        assert_eq!(of("ru_RU"), Lang::Ru);
        assert_eq!(of("uk-UA"), Lang::Uk);
        assert_eq!([of("es-ES"), of("es-MX"), of("es-419")], [Lang::Es; 3], "Spanish wherever it is spoken");
        assert_eq!([of("pt-BR"), of("pt-PT")], [Lang::PtBr; 2], "Brazil's Portuguese is read in Portugal too");
        assert_eq!([of("id-ID"), of("in-ID"), of("in_ID")], [Lang::Id; 3], "by the code Android wrote Indonesian with, too");
        assert_eq!(of("vi-VN"), Lang::Vi);
        assert_eq!([of("de-DE"), of("de-AT")], [Lang::De; 2]);
        assert_eq!([of("fr-FR"), of("fr-CA")], [Lang::Fr; 2]);
        assert_eq!(of("it-IT"), Lang::It);
        assert_eq!(of("pl-PL"), Lang::Pl);
        assert_eq!(of("ja-JP"), Lang::Ja);
        assert_eq!(of("ko-KR"), Lang::En, "a language the pages do not speak");
        assert_eq!(of("ar-EG"), Lang::En);
'''
new = '''        assert_eq!(of("en-US"), Lang::En);
        assert_eq!(of("en-GB"), Lang::En);
        assert_eq!(of("ko-KR"), Lang::En, "a language the pages do not speak");
        assert_eq!(of("ar-EG"), Lang::En);
        assert_eq!(of("tr-TR"), Lang::En, "and so is one this project used to speak");
'''
assert old in s, 'locale block not found'
s = s.replace(old, new)

old = '''        assert_eq!(Lang::ALL.map(Lang::name)[..4], ["简体中文", "繁體中文", "English", "Türkçe"]);
        assert_eq!(Lang::Ja.name(), "日本語");
'''
new = '''        assert_eq!(Lang::ALL.map(Lang::name), ["简体中文", "繁體中文", "English"]);
'''
assert old in s, 'name block not found'
s = s.replace(old, new)

old = '''        assert_eq!(Lang::Tr.among("骁龙", "驍龍", "Snapdragon"), "Snapdragon", "a phone describes itself in Chinese and English alone");
'''
new = ''
assert old in s, 'among block not found'
s = s.replace(old, new)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('lang.rs tests trimmed')
