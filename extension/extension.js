/**
 * uti GNOME Shell Extension
 *
 * Positions the uti window at cursor location when daemon sends D-Bus signal.
 *
 * Architecture:
 *   daemon (evdev) --D-Bus--> Extension --move window--> Tauri app
 *
 * D-Bus Interface (daemon):
 *   Bus name: io.github.noppomario.uti
 *   Path: /io/github/noppomario/uti/DoubleTap
 *   Interface: io.github.noppomario.uti.DoubleTap
 *   Signal: Triggered()
 */

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Meta from 'gi://Meta';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';

const DAEMON_BUS_NAME = 'io.github.noppomario.uti';
const DAEMON_OBJECT_PATH = '/io/github/noppomario/uti/DoubleTap';
const DAEMON_INTERFACE = 'io.github.noppomario.uti.DoubleTap';
const UTI_WM_CLASS = 'uti';

export default class UtiExtension extends Extension {
    constructor(metadata) {
        super(metadata);
        this._dbusConnection = null;
        this._signalSubscriptionId = null;
    }

    enable() {
        console.log('[uti] Extension enabled');
        this._connectToDbus();
    }

    disable() {
        console.log('[uti] Extension disabled');
        this._disconnectFromDbus();
    }

    /**
     * Connect to D-Bus and subscribe to daemon signals
     */
    _connectToDbus() {
        try {
            this._dbusConnection = Gio.bus_get_sync(Gio.BusType.SESSION, null);

            this._signalSubscriptionId = this._dbusConnection.signal_subscribe(
                DAEMON_BUS_NAME,
                DAEMON_INTERFACE,
                'Triggered',
                DAEMON_OBJECT_PATH,
                null,
                Gio.DBusSignalFlags.NONE,
                this._onTriggered.bind(this)
            );

            console.log('[uti] Connected to D-Bus, listening for daemon signals');
        } catch (e) {
            console.error(`[uti] Failed to connect to D-Bus: ${e.message}`);
        }
    }

    /**
     * Disconnect from D-Bus
     */
    _disconnectFromDbus() {
        if (this._signalSubscriptionId && this._dbusConnection) {
            this._dbusConnection.signal_unsubscribe(this._signalSubscriptionId);
            this._signalSubscriptionId = null;
        }
        this._dbusConnection = null;
    }

    /**
     * Handler for daemon's Triggered signal
     */
    _onTriggered(connection, sender, path, iface, signal, params) {
        console.log('[uti] Received Triggered signal from daemon');

        // Get cursor position
        const [x, y] = global.get_pointer();
        console.log(`[uti] Cursor position: (${x}, ${y})`);

        // Find and move uti window
        const window = this._findUtiWindow();
        if (window) {
            this._moveWindowToCursor(window, x, y);
            this._activateWindow(window);
        } else {
            console.log('[uti] Window not found, it may not be open yet');
            // Window will be shown by Tauri's existing D-Bus listener
            // We'll try to position it after a short delay
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                const win = this._findUtiWindow();
                if (win) {
                    const [curX, curY] = global.get_pointer();
                    this._moveWindowToCursor(win, curX, curY);
                }
                return GLib.SOURCE_REMOVE;
            });
        }
    }

    /**
     * Find the uti window by WM_CLASS
     */
    _findUtiWindow() {
        const windows = global.get_window_actors();
        for (const actor of windows) {
            const metaWindow = actor.get_meta_window();
            if (metaWindow) {
                const wmClass = metaWindow.get_wm_class();
                if (wmClass && wmClass.toLowerCase() === UTI_WM_CLASS) {
                    return metaWindow;
                }
            }
        }
        return null;
    }

    /**
     * Move window so its top-left corner is at cursor position
     * Adjusts position to keep window within screen bounds
     */
    _moveWindowToCursor(metaWindow, cursorX, cursorY) {
        const rect = metaWindow.get_frame_rect();
        const monitor = metaWindow.get_monitor();
        const workArea = Main.layoutManager.getWorkAreaForMonitor(monitor);

        // Calculate position (cursor at top-left of window)
        let newX = cursorX;
        let newY = cursorY;

        // Adjust to keep window within work area
        if (newX + rect.width > workArea.x + workArea.width) {
            newX = workArea.x + workArea.width - rect.width;
        }
        if (newY + rect.height > workArea.y + workArea.height) {
            newY = workArea.y + workArea.height - rect.height;
        }
        if (newX < workArea.x) {
            newX = workArea.x;
        }
        if (newY < workArea.y) {
            newY = workArea.y;
        }

        console.log(`[uti] Moving window to (${newX}, ${newY})`);
        metaWindow.move_frame(true, newX, newY);
    }

    /**
     * Activate (focus) the window
     */
    _activateWindow(metaWindow) {
        const timestamp = global.get_current_time();
        metaWindow.activate(timestamp);
    }
}
