// Files and links dropped from another app onto the pinned popup (ADR-041),
// from drag-source.js offering what Nautilus and browsers offer. Fixture
// external.launcher.json: 0 Docs/ (directory), 1 Big/, 2 b.txt,
// 3 example.com, 4 Recent Files, 5 ---, 6 Drop/ (empty folder), 7 Settings.

import GLib from 'gi://GLib';
import {check, finish, Input, Keys, rowCenter, showLauncher, sleep, snapshot, startDragSource, state, waitFor} from './lib.js';

const work = GLib.getenv('E2E_WORK');
const DATA = `${work}/data`;
const input = new Input();

function read(name) {
    const [, bytes] = GLib.file_get_contents(`${work}/config/uti/${name}`);
    return JSON.parse(new TextDecoder().decode(bytes));
}

const items = () => read('launcher.json').items;
const texts = (depth = 0) => state().levels[depth]?.texts ?? [];
const uriList = (...uris) => ({'text/uri-list': uris.map(u => `${u}\r\n`).join('')});
const file = path => `file://${DATA}/${path}`;

const source = startDragSource(input, [
    {label: 'file', formats: uriList(file('tree/Docs/a.pdf'))},
    {label: 'folder', formats: uriList(file('tree/Docs/Sub'))},
    {label: 'app', formats: uriList(file('applications/uti-e2e-editor.desktop'))},
    {
        label: 'link',
        formats: {...uriList('https://example.org/page'), 'text/plain;charset=utf-8': 'https://example.org/page'},
    },
    {label: 'two files', formats: uriList(file('tree/Docs/b.txt'), file('tree/Docs/Sub/c.txt'))},
    {label: 'text', formats: {'text/plain;charset=utf-8': 'just some words'}},
]);
check(source.origin !== null, `the other app is found: ${source.origin}`);
// The popup goes beside the other app's window.
const ORIGIN = source.origin[0] > 520 ? [100, 150] : [source.origin[0] + 300, 150];

/** A point in row `index` of level `depth`, `f` of its height down. */
function at(depth, index, f = 0.5) {
    const s = state();
    const [x, , , h] = s.levels[depth].rows[index];
    const [cx, cy] = rowCenter(s, ORIGIN, depth, index);
    return [Math.max(cx, ORIGIN[0] + x + 40), cy - h / 2 + h * f];
}

/** Drags row `i` of the other app to `to`. */
function dragFrom(i, to, opts) {
    input.glide(...source.row(i), 100);
    sleep(150);
    input.drag(...to, opts);
}

check(waitFor(() => state() !== null, 15000), 'app answers debug-state');
showLauncher(input, ORIGIN);
const before = JSON.stringify(items());

// Unpinned, pressing in the other app hides the popup: nothing is added.
dragFrom(0, at(0, 0, 0.2));
check(JSON.stringify(items()) === before, 'unpinned: nothing is added');

showLauncher(input, ORIGIN);
// Up to the search entry (a row below the first may be selected), then
// right to the pin button.
for (let i = 0; i < 30 && state().focus !== 'Search'; i++) input.key(Keys.Up);
input.key(Keys.Right);
input.key(Keys.Enter);
input.key(Keys.Left);
check(state().pinned, 'pinned');

// A file between rows, shown with the insertion line while dragging.
let mid = null;
dragFrom(0, at(0, 0, 0.2), {
    during: () => {
        mid = state();
        snapshot('external-line');
    },
});
check(mid.drag && mid.drop?.list === 0 && mid.drop.line !== undefined, `a line shows the spot: ${JSON.stringify(mid.drop)}`);
check(
    items()[0].type === 'file' && items()[0].path.endsWith('/tree/Docs/a.pdf') && texts()[0] === 'a.pdf',
    `a file becomes a file node: ${JSON.stringify(items()[0])}`,
);
check(!state().drag, 'the drag is over');
check(source.log().includes('drag-end delete=false') && !source.log().includes('delete=true'), 'it is copied, never moved');

// A folder into a folder (its middle).
const drop = () => texts().indexOf('Drop');
dragFrom(1, at(0, drop(), 0.5));
let folder = items().find(i => i.name === 'Drop');
check(
    folder.children.length === 1 && folder.children[0].type === 'directory' && folder.children[0].path.endsWith('/Docs/Sub'),
    `a folder becomes a directory node in Drop: ${JSON.stringify(folder.children)}`,
);

// A desktop file of an installed app, before Settings.
dragFrom(2, at(0, texts().indexOf('Settings'), 0.2));
let i = items().findIndex(n => n.name === 'Settings');
check(
    items()[i - 1].type === 'app' && items()[i - 1].desktopId === 'uti-e2e-editor.desktop',
    `a desktop file becomes an app node: ${JSON.stringify(items()[i - 1])}`,
);

// A link at the end (below the rows).
const [, lastY, , lastH] = state().levels[0].rows.at(-1);
dragFrom(3, [ORIGIN[0] + 150, ORIGIN[1] + lastY + lastH + 20]);
check(
    items().at(-1).type === 'url' && items().at(-1).url === 'https://example.org/page' && texts().at(-1) === 'example.org',
    `a link becomes a url node at the end: ${JSON.stringify(items().at(-1))}`,
);

// Two files at once, in order; into an open level by spring-loading.
const count = items().length;
dragFrom(4, at(0, drop(), 0.5), {
    hold: 700,
    during: () => {
        const s = state();
        check(s.levels.length === 2, `resting on Drop opens it: ${s.levels.length}`);
        input.glide(...at(1, 0, 0.2), 200, 6);
        sleep(300);
    },
});
folder = items().find(n => n.name === 'Drop');
check(
    folder.children.map(c => c.path?.split('/').at(-1)).join() === 'b.txt,c.txt,Sub',
    `both files go before Sub in Drop: ${folder.children.map(c => c.path).join()}`,
);
check(items().length === count, 'nothing else changes');

// Plain text that is not a link is not taken.
const now = JSON.stringify(items());
dragFrom(5, at(0, 0, 0.2));
check(JSON.stringify(items()) === now, 'plain text is not added');
sleep(500);
check(!state().drag, 'and the drag is over');

source.stop();
finish();
