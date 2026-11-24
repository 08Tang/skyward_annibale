import io

p = 'crates/skyward-ui/src/lang.rs'
s = io.open(p, encoding='utf-8').read()

old = '''        assert_eq!([1, 2, 21].map(|n| Lang::En.plural(n)), [One, Other, Other]);
        assert_eq!([1, 2, 5, 11, 21, 22, 25].map(|n| Lang::Ru.plural(n)), [One, Few, Many, Many, One, Few, Many]);
        assert_eq!([1, 2, 5, 22].map(|n| Lang::Pl.plural(n)), [One, Few, Many, Few]);
        assert_eq!([0, 1, 2].map(|n| Lang::Fr.plural(n)), [One, One, Other], "French counts none as one");
        assert_eq!([1, 2].map(|n| Lang::Hans.plural(n)), [Other, Other], "Chinese has one form");
        assert_eq!([1, 2].map(|n| Lang::Ja.plural(n)), [Other, Other]);
'''
new = '''        assert_eq!([1, 2, 21].map(|n| Lang::En.plural(n)), [One, Other, Other]);
        assert_eq!([1, 2].map(|n| Lang::Hans.plural(n)), [Other, Other], "Chinese has one form");
        assert_eq!([1, 2].map(|n| Lang::Hant.plural(n)), [Other, Other], "and so does its traditional form");
'''
assert old in s, 'plural block not found'
s = s.replace(old, new)

# `Few` and `Many` are no longer reachable through the three languages kept.
s = s.replace('        use PluralCategory::{Few, Many, One, Other};\n', '        use PluralCategory::{One, Other};\n')

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('lang.rs plural test trimmed')
