import io

p = 'crates/skyward-ui/src/widgets.rs'
s = io.open(p, encoding='utf-8').read()

# The keyboard test compared a wide-script language against English; with
# only three languages left, it compares a long English word instead.
old_a = '        let wide = keyboard(&mut c, &t, &f, Lang::Ru, top, Layer::Letters, None, "\u0420\u0430\u0441\u0448\u0438\u0444\u0440\u043e\u0432\u043a\u0430");\n'
new_a = '        let wide = keyboard(&mut c, &t, &f, Lang::En, top, Layer::Letters, None, "Decryption");\n'
assert old_a in s, 'wide line not found'
s = s.replace(old_a, new_a)

io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('widgets.rs keyboard test retargeted')
