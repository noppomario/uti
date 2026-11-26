// Generated nodes: installed apps by category, one app, power actions inline
// (ADR-037). Power actions only log their D-Bus call (UTI_DRY_POWER).

import {check, finish, Input, Keys, log, rowCenter, showLauncher, sleep, snapshot, state, waitFor} from './lib.js';

const ORIGIN = [200, 150];
const input = new Input();
const show = () => showLauncher(input, ORIGIN);

/** Clicks the row labelled `text` at `depth`; false when there is none. */
function clickRow(depth, text) {
    const s = state();
    const index = s.levels[depth]?.texts.indexOf(text) ?? -1;
    if (index < 0) return false;
    input.glide(...rowCenter(s, ORIGIN, depth, index), 60);
    input.click();
    sleep(300);
    return true;
}

check(waitFor(() => state() !== null, 15000), 'app answers debug-state');
let s = show();
const root = s.levels[0].texts;
check(root.includes('Applications') && root.includes('E2E Editor'), 'apps and app nodes are rows');
check(
    root.slice(-5).join(',') === 'Lock,Suspend,Log Out,Restart,Power Off',
    'inline power puts its actions in place',
);

// Applications > Programming > E2E Editor > its recent file.
check(clickRow(0, 'Applications'), 'Applications is clickable');
s = state();
check(s.levels.length === 2 && s.levels[1].texts.includes('Programming'), 'Applications lists categories');
check(clickRow(1, 'Programming'), 'Programming is clickable');
s = state();
const editor = s.levels[2]?.texts.indexOf('E2E Editor') ?? -1;
check(editor >= 0, 'Programming lists the fixture app');
input.glide(...rowCenter(s, ORIGIN, 2, editor), 60);
sleep(450);
s = state();
check(s.levels.length === 4 && s.levels[3].texts[0] === 'draft.txt', 'the app opens its recent files on hover');
snapshot('apps-open');
input.glide(...rowCenter(s, ORIGIN, 3, 0), 60);
input.click();
sleep(300);
check(log().includes('DEBUG launch Some("uti-e2e-editor.desktop") Some("/tmp/e2e/draft.txt")'), 'a recent file opens with the app');
check(!state().visible, 'launching hides the popup');

// The app node launches the app.
show();
check(clickRow(0, 'E2E Editor'), 'the app node is clickable');
check(log().includes('DEBUG launch Some("uti-e2e-editor.desktop") None'), 'a click on the app node launches it');

// Power actions call D-Bus (dry run).
for (const [label, line] of [
    ['Lock', 'power: Lock via org.gnome.ScreenSaver'],
    ['Suspend', 'power: Suspend via org.freedesktop.login1'],
    ['Power Off', 'power: Shutdown via org.gnome.SessionManager'],
]) {
    show();
    clickRow(0, label);
    check(log().includes(line), `${label} calls ${line.split(' via ')[1]}`);
    check(!state().visible, `${label} hides the popup`);
}

// Search finds the app once although it is both in the tree and installed.
s = show();
input.type('e2e editor');
sleep(300);
s = state();
check(s.levels[0].texts.join(',') === 'E2E Editor', 'search lists the app once');
snapshot('apps-search');
input.key(Keys.Escape);
input.key(Keys.Escape);

finish();
