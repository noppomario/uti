// File, folder, link and recent-files nodes (ADR-041). Fixture
// nodes.launcher.json: 0 Docs/ (directory), 1 Big/ (205 files), 2 b.txt,
// 3 example.com, 4 Recent Files, 5 ---, 6 Drop/, 7 Settings.

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import {check, finish, Input, Keys, log, rowCenter, showLauncher, sleep, snapshot, state, waitFor} from './lib.js';

const ORIGIN = [200, 150];
const work = GLib.getenv('E2E_WORK');
const DATA = `${work}/data`;
const input = new Input();

function read(name) {
    const [, bytes] = GLib.file_get_contents(`${work}/config/uti/${name}`);
    return JSON.parse(new TextDecoder().decode(bytes));
}

const items = () => read('launcher.json').items;
const texts = (depth = 0) => state().levels[depth]?.texts ?? [];

function hover(depth, index) {
    input.glide(...rowCenter(state(), ORIGIN, depth, index), 60);
}

/** Opens row `index` of level `depth` with a click. */
function open(depth, index) {
    hover(depth, index);
    input.click();
    sleep(300);
}

/** Clicks a launchable row; true when the app logged `open: <uri>`. */
function opens(depth, index, uri) {
    hover(depth, index);
    input.click();
    const ok = waitFor(() => log().includes(`open: ${uri}\n`), 2000);
    showLauncher(input, ORIGIN);
    return ok;
}

function menuAt(depth, index) {
    hover(depth, index);
    input.rightClick();
    waitFor(() => state().menu);
    return state().menuItems.map(i => i.label);
}

function pick(label) {
    const item = state().menuItems.find(i => i.label === label);
    if (!item) return false;
    const [x, y, w, h] = item.rect;
    input.glide(ORIGIN[0] + x + w / 2, ORIGIN[1] + y + h / 2, 60);
    input.click();
    sleep(400);
    return true;
}

function keepSnapshot(name) {
    sleep(900);
    Gio.File.new_for_path(`${work}/last.png`).copy(
        Gio.File.new_for_path(`${work}/${name}.png`),
        Gio.FileCopyFlags.OVERWRITE,
        null,
        null,
    );
}

function focusPopup() {
    input.glide(ORIGIN[0] + 190, ORIGIN[1] + 12, 60);
    input.click();
    sleep(300);
}

check(waitFor(() => state() !== null, 15000), 'app answers debug-state');
showLauncher(input, ORIGIN);
check(
    texts().join() === 'Docs,Big,b.txt,example.com,Recent Files,,Drop,Settings',
    `names default to the file name and host: ${texts()}`,
);

// A directory lists "open this folder", then folders, then files.
open(0, 0);
check(
    texts(1).join() === 'Open This Folder,,Sub,a.pdf,b.txt',
    `Docs lists folders first, hidden and backup files left out: ${texts(1)}`,
);
snapshot('nodes-directory');
open(1, 2);
check(texts(2).join() === 'Open This Folder,,c.txt', `Sub opens as a level: ${texts(2)}`);
check(opens(2, 2, `file://${DATA}/tree/Docs/Sub/c.txt`), 'a listed file opens with its app');

open(0, 0);
check(opens(1, 0, `file://${DATA}/tree/Docs`), '"Open This Folder" opens it');

// Past the cap: the last row says how many are left out.
open(0, 1);
const big = texts(1);
check(big.length === 203 && big.at(-1) === '5 More…', `Big lists 200 and "5 More…": ${big.length} ${big.at(-1)}`);
snapshot('nodes-more');

check(opens(0, 2, `file://${DATA}/tree/Docs/b.txt`), 'a file node opens with its app');
check(opens(0, 3, 'https://example.com/page'), 'a link opens with the browser');

// Recent files of every app, from the system history.
open(0, 4);
check(texts(1).includes('draft.txt'), `Recent Files lists the history: ${texts(1)}`);
let labels = menuAt(1, 0);
check(labels.includes('Remove from History'), `recent rows can be removed: ${labels}`);
input.key(Keys.Escape);
input.key(Keys.Escape);
input.key(Keys.Escape);

// A listed file: open its folder, copy its path, or add it.
showLauncher(input, ORIGIN);
open(0, 0);
labels = menuAt(1, 3);
check(labels.join() === 'Open Folder,Copy Path,Add to Launcher', `a listed file's menu: ${labels}`);
pick('Add to Launcher');
check(
    items()[0].type === 'file' && items()[0].path.endsWith('/tree/Docs/a.pdf') && items()[1].path.endsWith('/Docs'),
    `it is added before the folder: ${JSON.stringify(items()[0])}`,
);
check(texts()[0] === 'a.pdf', 'and shown');

// Search finds files and links by name.
input.key(Keys.Escape);
input.type('exam');
sleep(300);
check(texts()[0] === 'example.com', `search finds the link: ${texts()}`);
input.key(Keys.Escape);
sleep(200);

// Editors per kind (pinned: the popup stays while Settings is shown).
input.key(Keys.Up); // the search entry
input.key(Keys.Right); // the pin button
input.key(Keys.Enter);
input.key(Keys.Left);
check(state().pinned, 'pinned');
hover(0, 4);
input.key(Keys.F2);
check(waitFor(() => state().editor === 'Edit Web Link'), `F2 edits the link: ${state().editor}`);
keepSnapshot('nodes-edit-url');
focusPopup();
menuAt(0, 0);
pick('New');
sleep(300);
labels = state().menuItems.map(i => i.label);
check(
    ['File Link…', 'Folder Link…', 'Web Link…'].every(l => labels.includes(l)),
    `New offers the link kinds: ${labels}`,
);
pick('Folder Link…');
check(waitFor(() => state().editor === 'New Folder Link'), `Folder Link… opens its editor: ${state().editor}`);
keepSnapshot('nodes-new-directory');
focusPopup();
hover(0, 1);
input.key(Keys.F2);
check(waitFor(() => state().editor === 'Edit Folder Link'), `F2 edits the folder: ${state().editor}`);
keepSnapshot('nodes-edit-directory');

finish();
