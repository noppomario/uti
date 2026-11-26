// Launcher tree: hover delays, the diagonal grace, keys and clicks (ADR-037).
// Fixture rows: 0 Files (recent files), 1 Dev/, 2 Office/, 3 ---, 4 Settings;
// Dev: 0 Terminal, 1 ---, 2 Tools/ (Calculator).

import {action, check, finish, Input, Keys, log, rowCenter, sleep, snapshot, state, waitFor} from './lib.js';

const ORIGIN = [200, 150]; // the window opens with its top-left at the pointer
const input = new Input();

function show() {
    action('hide');
    waitFor(() => !state().visible);
    input.moveTo(...ORIGIN);
    action('show');
    waitFor(() => state().visible);
    sleep(600);
    // A shown window may not get keyboard focus here: click its header.
    input.moveTo(ORIGIN[0] + 190, ORIGIN[1] + 12);
    input.click();
    waitFor(() => state().active !== undefined && log().includes('active=true'));
    input.key(Keys.Right);
    input.key(Keys.Right);
    return state();
}

function levels() {
    return state().levels.length;
}

check(waitFor(() => state() !== null, 15000), 'app answers debug-state');
let s = show();
check(s.tab === 'launcher', 'Right twice reaches the Launcher tab');

// Hover: select at once, open after the delay.
input.glide(...rowCenter(s, ORIGIN, 0, 1));
sleep(100);
s = state();
check(s.levels.length === 1 && s.levels[0].selected === 1, 'hover selects Dev without opening yet');
sleep(400);
s = state();
check(s.levels.length === 2 && s.levels[1].owner === 1, 'Dev opens after the hover delay');
check(s.levels[1].rows[0][1] === s.levels[0].rows[1][1], 'first row of Dev lines up with the Dev row');
snapshot('dev-open');

// Diagonal move into Dev's level, crossing Office: Dev stays open.
input.glide(...rowCenter(s, ORIGIN, 1, 2), 150);
sleep(500);
s = state();
check(s.levels[1]?.owner === 1, 'moving diagonally across Office keeps Dev open');
check(s.levels.length === 3 && s.levels[2].owner === 2, 'Tools opens on hover');
snapshot('tools-open');

// Hovering another root row closes the open level after the delay.
input.moveTo(...rowCenter(s, ORIGIN, 0, 4));
check(levels() === 3, 'a sibling hover keeps the open level during the delay');
sleep(400);
check(levels() === 1, 'and closes it after the delay');

// Keys walk the levels; the pointer stays away.
input.moveTo(1200, 800);
input.key('2'.codePointAt(0));
s = state();
check(s.levels.length === 2 && s.active === 1 && s.levels[1].selected === 0, '2 opens Dev and enters it');
input.key(Keys.Down);
check(state().levels[1].selected === 2, 'Down skips the separator');
input.key(Keys.Right);
s = state();
check(s.levels.length === 3 && s.active === 2, 'Right opens and enters Tools');
input.key(Keys.Left);
s = state();
check(s.levels.length === 2 && s.active === 1, 'Left closes one level');
input.key(Keys.Escape);
s = state();
check(s.levels.length === 1 && s.active === 0 && s.levels[0].selected === 1, 'Esc returns to the Dev row');

// Recent files open as children and launch with the command.
input.key(Keys.Up);
input.key(Keys.Right);
s = state();
check(s.levels.length === 2 && s.levels[1].rows.length === 2, 'Right on Files lists its two recent files');
snapshot('recent-files');
input.key(Keys.Enter);
sleep(300);
check(log().includes('"true", "files", "/tmp/e2e/notes.txt"'), 'Enter opens the newest file with the command');
check(!state().visible, 'launching hides the popup');

// Clicks: folders open at once, commands launch.
s = show();
input.glide(...rowCenter(s, ORIGIN, 0, 2), 60);
input.click();
s = state();
check(s.levels.length === 2 && s.levels[1].owner === 2, 'a click opens Office at once');
input.glide(...rowCenter(s, ORIGIN, 1, 0), 60);
input.click();
sleep(300);
check(log().includes('"true", "writer"'), 'a click on Writer launches it');
check(!state().visible, 'and hides the popup');

// Clicks inside a level keep the popup, pinned or not (levels do not grab).
for (const pinned of [false, true]) {
    s = show();
    if (pinned) {
        for (let i = 0; i < 6; i++) input.key(Keys.Up); // up to the search entry
        input.key(Keys.Right); // past the last tab: the pin button
        input.key(Keys.Enter);
        input.key(Keys.Left);
        check(state().pinned, 'Enter on the pin pins the popup');
    }
    input.glide(...rowCenter(state(), ORIGIN, 0, 1), 60);
    sleep(450);
    input.glide(...rowCenter(state(), ORIGIN, 1, 2), 60);
    input.click();
    sleep(400);
    s = state();
    check(s.visible && s.levels.length === 3, `a click on Tools inside Dev keeps everything (pinned: ${pinned})`);
    input.key(Keys.Escape);
    input.key(Keys.Escape);
}
// Search covers the whole tree.
s = show();
input.type('calc');
sleep(300);
s = state();
check(s.levels[0].rows.length >= 1, 'typing searches the tree and apps');
snapshot('search');
input.key(Keys.Escape);
input.key(Keys.Escape);

finish();
