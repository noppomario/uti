// Editing the launcher from its context menus (ADR-038): checks launcher.json
// and the rows after each edit. Fixture rows as in launcher.js (0 Files,
// 1 Dev/, 2 Office/, 3 ---, 4 Settings, 5 ---, 6 Applications, ...).

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import {check, finish, Input, Keys, rowCenter, showLauncher, sleep, snapshot, state, waitFor} from './lib.js';

const ORIGIN = [200, 150];
const work = GLib.getenv('E2E_WORK');
const CONFIG = `${work}/config/uti/launcher.json`;
const input = new Input();

function read(path) {
    const [, bytes] = GLib.file_get_contents(path);
    return new TextDecoder().decode(bytes);
}

const items = () => JSON.parse(read(CONFIG)).items;
const texts = (depth = 0) => state().levels[depth]?.texts ?? [];

/** Right-clicks row `index` of level `depth`; the menu's labels. */
function menuAt(depth, index) {
    input.glide(...rowCenter(state(), ORIGIN, depth, index), 60);
    input.rightClick();
    waitFor(() => state().menu);
    return state().menuItems.map(i => i.label);
}

/** Clicks the shown menu entry `label`; false when there is none. */
function pick(label) {
    const item = state().menuItems.find(i => i.label === label);
    if (!item) return false;
    const [x, y, w, h] = item.rect;
    input.glide(ORIGIN[0] + x + w / 2, ORIGIN[1] + y + h / 2, 60);
    input.click();
    sleep(400);
    return true;
}

/** Copies the last debug snapshot (taken when the settings window opened). */
function keepSnapshot(name) {
    sleep(900);
    Gio.File.new_for_path(`${work}/last.png`).copy(
        Gio.File.new_for_path(`${work}/${name}.png`),
        Gio.FileCopyFlags.OVERWRITE,
        null,
        null,
    );
}

/** Gives the popup keyboard focus again (after the settings window). */
function focusPopup() {
    input.glide(ORIGIN[0] + 190, ORIGIN[1] + 12, 60);
    input.click();
    sleep(300);
}

check(waitFor(() => state() !== null, 15000), 'app answers debug-state');
let s = showLauncher(input, ORIGIN);
check(s.levels[0].more === 0, 'no more buttons while unpinned');
input.key(Keys.Up); // the search entry
input.key(Keys.Right); // past the last tab: the pin button
input.key(Keys.Enter);
input.key(Keys.Left);
s = state();
check(s.pinned && s.levels[0].more === 11, 'pinned rows show their more buttons');

// New > Separator goes before the row.
let labels = menuAt(0, 4);
check(
    ['Edit…', 'New', 'Move Up', 'Move Down', 'Delete'].every(l => labels.includes(l)),
    `a node's menu: ${labels.join(', ')}`,
);
snapshot('edit-menu');
check(pick('New'), 'New opens its submenu');
labels = state().menuItems.map(i => i.label);
check(labels.includes('Folder…') && labels.includes('Separator'), `New lists the kinds: ${labels.join(', ')}`);
check(state().menuItems.find(i => i.label === 'Special Item…')?.sensitive, 'Special Item… is offered');
snapshot('edit-new');
pick('Separator');
check(items()[4].type === 'separator' && items()[5].name === 'Settings', 'a separator is saved before Settings');
check(texts()[4] === '' && texts()[5] === 'Settings', 'and shown there');
check(items().every(i => typeof i.id === 'string' && i.id.length > 0), 'every item is saved with its id');

// Move Up, then delete the new separator (no Edit on separators).
menuAt(0, 5);
pick('Move Up');
check(items()[4].name === 'Settings' && texts()[4] === 'Settings', 'Move Up swaps Settings and the separator');
labels = menuAt(0, 5);
check(!labels.includes('Edit…'), 'a separator has no Edit');
pick('Delete');
check(items().length === 13 && texts()[5] === '', 'Delete removes the separator');

// Inside a level: the level stays open after the edit.
input.glide(...rowCenter(state(), ORIGIN, 0, 1), 60);
input.click();
sleep(300);
menuAt(1, 0);
snapshot('edit-level-menu');
pick('Move Down');
s = state();
check(items()[1].children[1].name === 'Terminal', 'Move Down inside Dev is saved');
check(s.levels.length === 2 && s.levels[1].texts[1] === 'Terminal', 'Dev stays open, showing the move');

// Add to Launcher from a generated row: before the node that generated it.
input.key(Keys.Escape);
input.glide(...rowCenter(state(), ORIGIN, 0, 6), 60);
input.click();
sleep(300);
const programming = texts(1).indexOf('Programming');
input.glide(...rowCenter(state(), ORIGIN, 1, programming), 60);
input.click();
sleep(300);
labels = menuAt(2, texts(2).indexOf('E2E Editor'));
check(labels.join() === 'Add to Launcher', 'a generated app offers Add to Launcher');
pick('Add to Launcher');
check(
    items()[6].type === 'app' && items()[6].desktopId === 'uti-e2e-editor.desktop' && items()[7].type === 'apps',
    'the app is saved before Applications',
);

// A folder with items asks first.
input.key(Keys.Escape);
input.key(Keys.Escape);
menuAt(0, 2);
pick('Delete');
sleep(400);
check(items()[2].name === 'Office', 'the folder stays until confirmed');
input.key(Keys.Enter);
sleep(400);
check(!items().some(i => i.name === 'Office') && !texts().includes('Office'), 'confirming deletes the folder');

// F2 edits the selected row in Settings; Enter saves.
input.glide(...rowCenter(state(), ORIGIN, 0, 0), 60);
input.key(Keys.F2);
check(waitFor(() => state().editor === 'Edit Command'), 'F2 opens the command in the editor');
keepSnapshot('edit-editor');
input.key(Keys.BackSpace);
input.type('Docs');
input.key(Keys.Enter);
check(waitFor(() => items()[0].name === 'Docs'), 'the new name is saved');
check(items()[0].historySource?.path?.endsWith('files.xbel'), 'unchanged fields are kept');
check(texts()[0] === 'Docs', 'and shown');

// New > Folder… in the empty space below the rows: appended.
focusPopup();
const [, lastY, , lastH] = state().levels[0].rows.at(-1);
input.glide(ORIGIN[0] + 150, ORIGIN[1] + lastY + lastH + 24, 60);
input.rightClick();
waitFor(() => state().menu);
check(state().menuItems.map(i => i.label).join() === 'New', 'the empty space offers New');
pick('New');
sleep(300); // the popup grows to the submenu after its slide
snapshot('edit-empty-new');
pick('Folder…');
check(waitFor(() => state().editor === 'New Folder'), 'Folder… opens a new folder editor');
input.type('Web');
input.key(Keys.Enter);
check(waitFor(() => items().at(-1).name === 'Web'), 'the folder is saved at the end');

// A broken file is not edited.
focusPopup();
GLib.file_set_contents(CONFIG, '{ broken');
labels = menuAt(0, 0);
check(labels.join() === 'launcher.json Has Errors…', 'a broken file only offers to fix it');
input.key(Keys.Escape);
check(read(CONFIG) === '{ broken', 'and is left as it is');

finish();
