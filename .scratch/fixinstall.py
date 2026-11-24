"""Show the slot rows only where the partition picked has slots to pick."""

import io

p = 'crates/skyward-ui/src/pages.rs'
s = io.open(p, encoding='utf-8').read()

old = '''            let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(["槽位", "槽位", "Slot"]));
            let titles = Pick::ALL.map(|p| pick_title(l, p, i.slot));
            let rows: Vec<Row> = Pick::ALL.iter().zip(&titles).map(|(p, title)| Row::new(title).trail(Trail::Check(*p == choice.pick))).collect();
            let picks = widgets::list(&mut c, t, f, y, &rows);
            out.extend(hits(&picks.rows, &Pick::ALL.map(Action::FlashSlots)));
            let (label, danger) = confirm_label(Confirm::Flash, cx.state);
            let track = widgets::confirm(&mut c, t, f, picks.bottom + t.m.dp(28.0), label, danger, s.confirm);
'''

new = '''            // A partition in a slot has one to pick, and a logical one is
            // the slot the phone runs from, which is shown with nothing to
            // pick; super is in no slot.
            let bottom = match choice.partitions[choice.partition].place {
                FlashPlace::Slotted => {
                    let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(["槽位", "槽位", "Slot"]));
                    let titles = Pick::ALL.map(|p| pick_title(l, p, i.slot));
                    let rows: Vec<Row> = Pick::ALL.iter().zip(&titles).map(|(p, title)| Row::new(title).trail(Trail::Check(*p == choice.pick))).collect();
                    let picks = widgets::list(&mut c, t, f, y, &rows);
                    out.extend(hits(&picks.rows, &Pick::ALL.map(Action::FlashSlots)));
                    picks.bottom
                }
                FlashPlace::Logical => {
                    let y = widgets::section(&mut c, t, f, parts.bottom, l.pick(["槽位", "槽位", "Slot"]));
                    let title = pick_title(l, Pick::Current, i.slot);
                    widgets::list(&mut c, t, f, y, &[Row::new(&title).trail(Trail::Check(true))]).bottom
                }
                FlashPlace::Single => parts.bottom,
            };
            let (label, danger) = confirm_label(Confirm::Flash, cx.state);
            let track = widgets::confirm(&mut c, t, f, bottom + t.m.dp(28.0), label, danger, s.confirm);
'''

assert old in s, 'slot section not found'
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('slot rows follow the partition place')
