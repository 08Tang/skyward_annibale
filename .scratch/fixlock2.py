import io

p = 'Cargo.lock'
s = io.open(p, encoding='utf-8').read()
lines = s.split('\n')

# Third-party crates that carry their own version, which must not be rewritten.
for name, version in (('ghash', '0.5.1'), ('universal-hash', '0.5.1'), ('wyz', '0.5.1')):
    idx = None
    for i, line in enumerate(lines):
        if line.strip() == 'name = "%s"' % name:
            idx = i
            break
    assert idx is not None, name
    # The version line follows the name within a few lines.
    for j in range(idx, idx + 4):
        if lines[j].strip().startswith('version = '):
            lines[j] = 'version = "%s"' % version
            print('restored', name, 'to', version)
            break
    else:
        raise AssertionError('no version for ' + name)

io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines))
