import io

p = 'Cargo.lock'
s = io.open(p, encoding='utf-8').read()
lines = s.split('\n')
out = []
i = 0
n = 0
while i < len(lines):
    if lines[i].startswith('<<<<<<<'):
        j = i + 1
        while not lines[j].startswith('======='):
            j += 1
        k = j + 1
        while not lines[k].startswith('>>>>>>>'):
            k += 1
        out.extend(lines[j + 1:k])
        n += 1
        i = k + 1
    else:
        out.append(lines[i])
        i += 1
s2 = '\n'.join(out)
s2 = s2.replace('version = "0.5.1"', 'version = "0.3.1-annibale.3"')
io.open(p, 'w', encoding='utf-8', newline='\n').write(s2)
print('took upstream in', n, 'hunks, versions rewritten')
