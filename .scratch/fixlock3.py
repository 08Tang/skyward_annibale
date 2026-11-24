import io

p = 'Cargo.lock'
lines = io.open(p, encoding='utf-8').read().split('\n')

# `skyward-ui` gained a vendored `qrcodegen` dependency; the lock must list it
# among that package's dependencies, in the order cargo writes them.
start = None
for i, line in enumerate(lines):
    if line.strip() == 'name = "skyward-ui"':
        start = i
        break
assert start is not None, 'skyward-ui not found'

open_bracket = None
for j in range(start, start + 5):
    if lines[j].strip() == 'dependencies = [':
        open_bracket = j
        break
assert open_bracket is not None, 'dependencies not found'

end = None
for j in range(open_bracket + 1, len(lines)):
    if lines[j].strip() == ']':
        end = j
        break
assert end is not None, 'end of dependencies not found'

deps = [line.strip().strip('"') for line in lines[open_bracket + 1:end]]
if 'qrcodegen' in deps:
    print('already listed')
else:
    deps.append('qrcodegen')
    deps.sort()
    body = [' "%s",' % d for d in deps]
    lines[open_bracket + 1:end] = body
    io.open(p, 'w', encoding='utf-8', newline='\n').write('\n'.join(lines))
    print('added qrcodegen to skyward-ui')
