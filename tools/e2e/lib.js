// Helpers for e2e scenarios: input through Mutter's RemoteDesktop API, the
// app's GActions, and assertions on its `debug-state` JSON.

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import System from 'system';

const RD = 'org.gnome.Mutter.RemoteDesktop';
const APP = 'io.github.noppomario.uti';
const APP_PATH = '/io/github/noppomario/uti';
const BTN_LEFT = 0x110;

export const Keys = {
    Up: 0xff52,
    Down: 0xff54,
    Left: 0xff51,
    Right: 0xff53,
    Enter: 0xff0d,
    Escape: 0xff1b,
};

const work = GLib.getenv('E2E_WORK');
const failures = [];

function call(name, path, iface, method, params = null, type = null) {
    return Gio.DBus.session.call_sync(
        name,
        path,
        iface,
        method,
        params,
        type ? new GLib.VariantType(type) : null,
        Gio.DBusCallFlags.NONE,
        -1,
        null,
    );
}

export function sleep(ms) {
    GLib.usleep(ms * 1000);
}

/** Waits until `fn` returns true (polling), or gives up after `ms`. */
export function waitFor(fn, ms = 5000) {
    for (let t = 0; t < ms; t += 100) {
        try {
            if (fn()) return true;
        } catch {
            // Not ready yet.
        }
        sleep(100);
    }
    return false;
}

/** Virtual pointer and keyboard. Coordinates are global (monitor) pixels. */
export class Input {
    constructor() {
        const [session] = call(RD, '/org/gnome/Mutter/RemoteDesktop', RD, 'CreateSession', null, '(o)')
            .deepUnpack();
        this.session = session;
        this._rd('Start');
        // Relative motion only (no screencast stream): park at the origin.
        this._rd('NotifyPointerMotionRelative', new GLib.Variant('(dd)', [-10000, -10000]));
        this.x = 0;
        this.y = 0;
    }

    _rd(method, params = null) {
        call(RD, this.session, `${RD}.Session`, method, params);
    }

    _move(x, y) {
        this._rd('NotifyPointerMotionRelative', new GLib.Variant('(dd)', [x - this.x, y - this.y]));
        this.x = x;
        this.y = y;
    }

    /** Jumps to (x, y) in one motion event. */
    moveTo(x, y) {
        this._move(x, y);
        sleep(30);
    }

    /** Moves in `steps` straight-line steps over about `ms`. */
    glide(x, y, ms = 150, steps = 10) {
        const [x0, y0] = [this.x, this.y];
        for (let i = 1; i <= steps; i++) {
            this._move(x0 + ((x - x0) * i) / steps, y0 + ((y - y0) * i) / steps);
            sleep(ms / steps);
        }
        sleep(30);
    }

    click() {
        this._rd('NotifyPointerButton', new GLib.Variant('(ib)', [BTN_LEFT, true]));
        this._rd('NotifyPointerButton', new GLib.Variant('(ib)', [BTN_LEFT, false]));
        sleep(100);
    }

    key(keysym) {
        this._rd('NotifyKeyboardKeysym', new GLib.Variant('(ub)', [keysym, true]));
        this._rd('NotifyKeyboardKeysym', new GLib.Variant('(ub)', [keysym, false]));
        sleep(120);
    }

    type(text) {
        for (const ch of text) this.key(ch.codePointAt(0));
    }
}

/** Activates a GAction of the app (see `register_actions`). */
export function action(name, param = null) {
    const params = param === null ? [] : [new GLib.Variant('s', param)];
    call(
        APP,
        APP_PATH,
        'org.freedesktop.Application',
        'ActivateAction',
        new GLib.Variant('(sava{sv})', [name, params, {}]),
    );
}

/** The app's Launcher state (`debug-state`). */
export function state() {
    const path = `${work}/state.json`;
    GLib.unlink(path);
    action('debug-state', path);
    let text = null;
    waitFor(() => {
        const [ok, bytes] = GLib.file_get_contents(path);
        if (ok) text = new TextDecoder().decode(bytes);
        return ok;
    }, 2000);
    return JSON.parse(text);
}

/** Center of row `index` of level `depth`, in global pixels. */
export function rowCenter(s, origin, depth, index) {
    const [x, y, w, h] = s.levels[depth].rows[index];
    return [origin[0] + x + w / 2, origin[1] + y + h / 2];
}

/** Lines the app printed so far. */
export function log() {
    const [, bytes] = GLib.file_get_contents(GLib.getenv('E2E_LOG'));
    return new TextDecoder().decode(bytes);
}

export function check(ok, what) {
    print(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
    if (!ok) failures.push(what);
}

/** Saves a PNG of the window and its open levels to target/e2e/<name>.png. */
export function snapshot(name) {
    action('debug-key', 'snap');
    sleep(800);
    Gio.File.new_for_path(`${work}/last.png`).copy(
        Gio.File.new_for_path(`${work}/${name}.png`),
        Gio.FileCopyFlags.OVERWRITE,
        null,
        null,
    );
}

export function finish() {
    if (failures.length) {
        print(`${failures.length} failed`);
        System.exit(1);
    }
}
