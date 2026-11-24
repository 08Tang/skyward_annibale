import io

p = 'crates/skyward-programs/tests/phones.rs'
s = io.open(p, encoding='utf-8').read()
old_a = 'secure::steps(&phone(codename, &properties)).unwrap()'
new_a = 'secure::steps(&phone(codename, &properties), true, true).unwrap()'
old_b = 'secure::steps(&with_bundle("ishtar", &properties, Some(Path::new(bundle)))).unwrap()'
new_b = 'secure::steps(&with_bundle("ishtar", &properties, Some(Path::new(bundle))), true, true).unwrap()'
assert old_a in s, 'a missing'
assert old_b in s, 'b missing'
s = s.replace(old_a, new_a).replace(old_b, new_b)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('patched both')
