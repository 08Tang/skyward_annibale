"""Finish the LOGIN constant the earlier restore left half written."""

import io

p = 'crates/skyward-ui/src/fmt.rs'
s = io.open(p, encoding='utf-8').read()

old = '''const LOGIN: Text = [

/// The rules of each of [`TIME_ZONES`], read once from the ones the
'''
new = '''const LOGIN: Text = [
    "需要先在网页上登录才能访问互联网，Skyward 无法打开网页登录。可以改用无需登录的网络，例如手机热点。",
    "需要先在網頁上登入才能連上網際網路，Skyward 無法開啟網頁登入。可以改用無需登入的網路，例如手機熱點。",
    "needs a sign-in in a browser before it reaches the internet, and Skyward has no browser to sign in with. Use a network that needs no sign-in instead, such as a phone's hotspot.",
];

/// The rules of each of [`TIME_ZONES`], read once from the ones the
'''

assert old in s, 'half-written LOGIN not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('LOGIN completed')
