// The special item palette (ADR-040): opened from New > Special Item… and
// from Settings, adds by drag, Enter and double-click, and pins the popup
// only while open. Fixture rows as in launcher.js (0 Files, 1 Dev/,
// 2 Office/, 3 ---, 4 Settings, ...).

import GLib from 'gi://GLib';
import {action, check, finish, Input, Keys, rowCenter, showLauncher, sleep, snapshot, state, waitFor} from './lib.js';

// Room on the left for the palette.
const ORIGIN = [600, 150];
const work = GLib.getenv('E2E_WORK');
const input = new Input();

function items() {
    const [, bytes] = GLib.file_get_contents(`${work}/config/uti/launcher.json`);
    return JSON.parse(new TextDecoder().decode(bytes)).items;
}

const label = i => i.name ?? (i.action ? `${i.type}:${i.action}` : i.type);
const labels = (list = items()) => list.map(label);
const texts = () => state().levels[0].texts;
const palette = () => state().palette;

/** Center of rectangle `r` (window coordinates), in global pixels. */
const center = ([x, y, w, h], f = 0.5) => [ORIGIN[0] + x + w / 2, ORIGIN[1] + y + h * f];

/** A point in row `index` of the root list, `f` of its height down. */
function at(index, f = 0.5) {
    const s = state();
    const [, , , h] = s.levels[0].rows[index];
    const [cx, cy] = rowCenter(s, ORIGIN, 0, index);
    return [cx, cy - h / 2 + h * f];
}

/** Right-clicks root row `index` and picks New > Special Item…. */
function openFromMenu(index) {
    input.glide(...at(index), 60);
    input.rightClick();
    waitFor(() => state().menu);
    for (const name of ['New', 'Special Item…']) {
        const item = state().menuItems.find(i => i.label === name);
        if (!item) return false;
        input.glide(...center(item.rect), 60);
        input.click();
        sleep(400);
    }
    return waitFor(() => palette()?.visible);
}

/** Toggles the pin from the keyboard: up to the search entry, right to the pin. */
function togglePin() {
    for (let i = 0; i < 30 && state().focus !== 'Search'; i++) input.key(Keys.Up);
    input.key(Keys.Right);
    input.key(Keys.Enter);
    input.key(Keys.Left);
}

function clickTab(category) {
    input.glide(...center(palette().tabs[category]), 60);
    input.click();
    sleep(200);
}

check(waitFor(() => state() !== null, 15000), 'app answers debug-state');
let s = showLauncher(input, ORIGIN);
check(!s.pinned && s.palette === null, 'unpinned, no palette');

// Opening pins the popup; the palette sits left of it.
check(openFromMenu(4), 'New > Special Item… opens the palette');
s = state();
let p = s.palette;
check(s.pinned && s.focus === 'Palette', `pinned and the palette has the keys: ${s.focus}`);
check(p.category === 0 && p.texts.join() === 'New Folder,Separator', `Basic first: ${p.texts}`);
check(p.rows[0][0] + p.rows[0][2] <= 2, `left of the popup: ${JSON.stringify(p.rows[0])}`);
snapshot('palette-open');

// Places: home, the trash and recent files.
input.key(Keys.Right);
input.key(Keys.Right);
p = palette();
check(
    p.category === 2 && p.texts[0] === 'Home' && p.texts.slice(-2).join() === 'Trash,Recent Files',
    `Places lists home, the trash and recent files: ${p.texts}`,
);
snapshot('palette-places');

// Keys: Right to Power and Session, Enter adds before Settings, in order.
input.key(Keys.Right);
p = palette();
check(p.category === 3 && p.texts.includes('Lock'), `Right switches categories: ${p.texts}`);
input.key(Keys.Down);
check(palette().selected === 1, 'Down selects Lock');
input.key(Keys.Enter);
input.key(Keys.Down);
input.key(Keys.Enter);
sleep(300);
let l = labels();
check(
    l.slice(4, 7).join() === 'power:lock,power:suspend,Settings',
    `Enter adds before the row the palette was opened on: ${l.slice(3, 8)}`,
);
check(texts().slice(4, 7).join() === 'Lock,Suspend,Settings', `shown: ${texts().slice(3, 8)}`);
check(state().palette?.visible && state().focus === 'Palette', 'the palette stays open');

// GNOME Settings panels are commands.
input.key(Keys.Right);
p = palette();
check(p.category === 4 && p.texts.length > 0, `GNOME Settings lists panels: ${p.texts.length}`);
input.key(Keys.Enter);
sleep(300);
const panel = items()[6];
check(
    panel.type === 'command' && panel.command === 'gnome-control-center' && panel.args.length === 1,
    `a panel is added as a command: ${JSON.stringify(panel)}`,
);
snapshot('palette-settings');

// Double-click adds too. First rest on a leaf row: a level opened under the
// resting pointer may flip left over the palette (wide).
input.glide(...at(texts().indexOf('Settings'), 0.5), 60);
sleep(500);
clickTab(0);
check(palette().category === 0, 'a tab click shows Basic');
const separators = () => items().filter(i => i.type === 'separator').length;
const before = separators();
input.glide(...center(palette().rows[1]), 60);
input.click();
input.click();
sleep(400);
check(separators() === before + 1 && items()[7].type === 'separator', 'double-click adds a separator');

// Drag: New Folder before Files, Power Off into Office.
let mid = null;
input.glide(...center(palette().rows[0]), 60);
sleep(100);
input.drag(...at(0, 0.2), {
    during: () => {
        mid = state();
        snapshot('palette-drag');
    },
});
check(mid?.drag && mid.drop?.list === 0 && mid.drop.line !== undefined, `a line shows the spot: ${JSON.stringify(mid?.drop)}`);
check(labels()[0] === 'New Folder' && items()[0].children.length === 0, `dropped before Files: ${labels().slice(0, 3)}`);
clickTab(3);
const office = texts().indexOf('Office');
input.glide(...center(palette().rows[5]), 60);
sleep(100);
input.drag(...at(office, 0.5), {hold: 100});
const officeItem = items().find(i => i.name === 'Office');
check(
    labels(officeItem.children).join() === 'Writer,power:powerOff',
    `dropped into Office: ${labels(officeItem.children)}`,
);
check(state().palette?.visible, 'the palette stays open after dragging');

// Esc closes it and unpins (it was unpinned before).
input.key(Keys.Escape);
s = state();
check(s.palette === null && !s.pinned && s.visible, `Esc closes and unpins: pinned=${s.pinned}`);

// Pinned before: the close button keeps the pin.
showLauncher(input, ORIGIN);
togglePin();
check(openFromMenu(0), 'opens again');
input.glide(...center(palette().close), 60);
input.click();
sleep(300);
s = state();
check(s.palette === null && s.pinned, 'the close button keeps a pin set before');
togglePin();
check(!state().pinned, 'unpinned');

// Settings > Launcher's button: shows the palette, adds at the root end.
action('hide');
waitFor(() => !state().visible);
input.moveTo(...ORIGIN);
action('debug-key', 'palette');
check(waitFor(() => state().visible && state().palette?.visible), 'Settings opens the popup with the palette');
s = state();
check(s.tab === 'launcher' && s.pinned, `on the Launcher, pinned: ${s.tab}`);
check(s.palette.category === 3, `it opens on the last category: ${s.palette.category}`);
input.key(Keys.Left);
input.key(Keys.Left);
input.key(Keys.Left);
input.key(Keys.Enter);
sleep(300);
l = items();
check(label(l[l.length - 1]) === 'New Folder', `added at the root end: ${label(l[l.length - 1])}`);
action('hide');
waitFor(() => !state().visible);
s = state();
check(s.palette === null && !s.pinned, 'hiding closes the palette and unpins');

finish();
