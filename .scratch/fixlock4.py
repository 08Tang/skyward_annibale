import io

p = 'Cargo.lock'
s = io.open(p, encoding='utf-8').read()
# Undo the stray trailing quote the previous pass left on each dependency line.
s = s.replace('",",\n', '",\n')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('cleaned')
