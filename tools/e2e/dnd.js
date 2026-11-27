// Drag-and-drop organizing while pinned (ADR-039): checks launcher.json,
// snippets.json and the rows after each drop. Fixture rows as in
// launcher.js (0 Files, 1 Dev/, 2 Office/, 3 ---, 4 Settings, 5 ---,
// 6 Applications, ...).

import GLib from 'gi://GLib';
import {check, finish, Input, Keys, rowCenter, showLauncher, sleep, snapshot, state, waitFor} from './lib.js';

const ORIGIN = [200, 150];
const work = GLib.getenv('E2E_WORK');
const input = new Input();

function read(name) {
    const [, bytes] = GLib.file_get_contents(`${work}/config/uti/${name}`);
    return JSON.parse(new TextDecoder().decode(bytes));
}

const items = () => read('launcher.json').items;
const names = list => list.map(i => i.name ?? i.type);
const texts = (depth = 0) => state().levels[depth]?.texts ?? [];
const find = (depth, text) => texts(depth).indexOf(text);

/** A point in row `index` of level `depth`, `f` of its height down. */
function at(depth, index, f = 0.5) {
    const s = state();
    const [x, , , h] = s.levels[depth].rows[index];
    const [cx, cy] = rowCenter(s, ORIGIN, depth, index);
    return [Math.max(cx, ORIGIN[0] + x + 40), cy - h / 2 + h * f];
}

/** Drags row `from` of level `fromDepth` to `to` ([x, y]). */
function drag(fromDepth, from, to, opts) {
    input.glide(...at(fromDepth, from), 60);
    sleep(100);
    input.drag(...to, opts);
}

check(waitFor(() => state() !== null, 15000), 'app answers debug-state');
showLauncher(input, ORIGIN);
const before = JSON.stringify(items());

drag(0, 4, at(0, 0, 0.2));
let s = state();
check(s.visible && JSON.stringify(items()) === before, 'unpinned: no drag, the popup stays');

input.key(Keys.Up); // the search entry
input.key(Keys.Right); // the pin button
input.key(Keys.Enter);
input.key(Keys.Left);
check(state().pinned, 'pinned');

// Between rows: Settings before Files.
let mid = null;
drag(0, 4, at(0, 0, 0.2), {
    during: () => {
        mid = state();
        snapshot('dnd-line');
    },
});
check(mid.drag && mid.drop?.list === 0 && mid.drop.line !== undefined, `a line shows the spot: ${JSON.stringify(mid.drop)}`);
check(names(items()).slice(0, 2).join() === 'Settings,Files', `Settings moved first: ${names(items()).join()}`);
check(texts().slice(0, 2).join() === 'Settings,Files', 'and is shown first');
check(!state().drag && !state().drop, 'nothing shown after the drop');

// Into a folder: Files to the end of Office.
drag(0, 1, at(0, find(0, 'Office')), {
    during: () => {
        mid = state();
        snapshot('dnd-into');
    },
});
check(mid.drop?.into === find(0, 'Office') + 1, `the folder row is highlighted: ${JSON.stringify(mid.drop)}`);
let office = items().find(i => i.name === 'Office');
check(names(office.children).join() === 'Writer,Files', `Files went to the end of Office: ${names(office.children)}`);
check(!texts().includes('Files'), 'and left the root');

// Spring-loading: hold over Dev, then drop inside its level (across
// levels).
input.glide(...at(0, find(0, 'Settings')), 60);
sleep(100);
input.button(true);
input.glide(input.x + 4, input.y + 6, 80, 4);
input.glide(...at(0, find(0, 'Dev')), 400, 20);
sleep(700);
mid = state();
snapshot('dnd-spring');
check(mid.levels.length === 2 && mid.levels[1].texts[0] === 'Terminal', 'holding over Dev opens it');
input.glide(...at(1, 0, 0.2), 300, 15);
sleep(200);
input.button(false);
sleep(700);
const dev = () => items().find(i => i.name === 'Dev');
check(names(dev().children)[0] === 'Settings', `Settings dropped first in Dev: ${names(dev().children)}`);
check(state().levels.length === 2, 'Dev stays open after the drop');

// Back out of a closing level: Terminal before Office at the root. The
// level it came from closes while the pointer rests on the root.
const terminal = find(1, 'Terminal');
drag(1, terminal, at(0, find(0, 'Office'), 0.1), {hold: 600});
check(names(items()).indexOf('Terminal') === names(items()).indexOf('Office') - 1, `Terminal before Office: ${names(items()).join()}`);
check(!names(dev().children).includes('Terminal'), 'and out of Dev');

// Into its own descendant: refused, nothing changes.
input.glide(...at(0, find(0, 'Dev')), 60);
input.click();
waitFor(() => state().levels.length === 2);
const kept = JSON.stringify(items());
drag(0, find(0, 'Dev'), at(1, find(1, 'Tools')), {
    hold: 500,
    during: () => {
        mid = state();
    },
});
check(mid.drop === null && mid.levels.length === 2, `no spot inside Dev itself: ${JSON.stringify(mid.drop)}`);
check(JSON.stringify(items()) === kept, 'launcher.json unchanged');

// A generated row is copied: Restart from the Power folder to the top.
input.glide(...at(0, find(0, 'Power')), 60);
input.click();
waitFor(() => state().levels.length === 2);
drag(1, find(1, 'Restart'), at(0, 0, 0.1));
check(items()[0].type === 'power' && items()[0].action === 'restart', `Restart added first: ${JSON.stringify(items()[0])}`);
check(texts()[0] === 'Restart' && texts().includes('Power'), 'shown first, Power still there');

// Snippets: Delta to the top.
input.key(Keys.Escape);
input.key(Keys.Left);
waitFor(() => state().tab === 'snippets');
const snip = index => {
    const [x, y, w, h] = state().snippets.rows[index];
    return [ORIGIN[0] + x + w / 2, ORIGIN[1] + y + h / 2];
};
input.glide(...snip(3), 60);
sleep(100);
const [sx, sy] = snip(0);
input.drag(sx, sy - 8, {
    during: () => {
        mid = state();
        snapshot('dnd-snippets');
    },
});
check(mid.drop?.list === 'snippets' && mid.drop.line !== undefined, `a line in Snippets: ${JSON.stringify(mid.drop)}`);
const values = read('snippets.json').items.map(i => i.value);
check(values.join() === 'Delta,Alpha,Beta,Gamma', `snippets.json order: ${values.join()}`);
check(state().snippets.texts.join() === 'Delta,Alpha,Beta,Gamma', 'and the rows');
check(state().snippets.selected === 0, 'the moved snippet stays selected');

finish();
