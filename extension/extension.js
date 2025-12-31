/**
 * uti GNOME Shell Extension
 *
 * Features:
 * 1. StatusNotifierItem host for uti's tray icon (replaces AppIndicator)
 * 2. Positions uti window at cursor location on Ctrl double-tap
 *
 * Architecture:
 *   daemon (evdev) --D-Bus--> Extension --move window--> Tauri app
 *   Tauri app --StatusNotifierItem--> Extension --panel icon--> GNOME Shell
 */

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';
import St from 'gi://St';
import Clutter from 'gi://Clutter';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';

const DAEMON_BUS_NAME = 'io.github.noppomario.uti';
const DAEMON_OBJECT_PATH = '/io/github/noppomario/uti/DoubleTap';
const DAEMON_INTERFACE = 'io.github.noppomario.uti.DoubleTap';
const UTI_WM_CLASS = 'uti';

// StatusNotifierItem interface
const SNIIface = `
<node>
  <interface name="org.kde.StatusNotifierItem">
    <property name="Id" type="s" access="read"/>
    <property name="IconName" type="s" access="read"/>
    <property name="Menu" type="o" access="read"/>
    <method name="Activate">
      <arg type="i" direction="in"/>
      <arg type="i" direction="in"/>
    </method>
  </interface>
</node>`;

// DBusMenu interface
const DBusMenuIface = `
<node>
  <interface name="com.canonical.dbusmenu">
    <method name="GetLayout">
      <arg type="i" direction="in"/>
      <arg type="i" direction="in"/>
      <arg type="as" direction="in"/>
      <arg type="u" direction="out"/>
      <arg type="(ia{sv}av)" direction="out"/>
    </method>
    <method name="Event">
      <arg type="i" direction="in"/>
      <arg type="s" direction="in"/>
      <arg type="v" direction="in"/>
      <arg type="u" direction="in"/>
    </method>
    <signal name="LayoutUpdated">
      <arg type="u"/>
      <arg type="i"/>
    </signal>
  </interface>
</node>`;

const SNIProxy = Gio.DBusProxy.makeProxyWrapper(SNIIface);
const DBusMenuProxy = Gio.DBusProxy.makeProxyWrapper(DBusMenuIface);

/**
 * Panel indicator that displays uti's StatusNotifierItem
 */
const UtiIndicator = GObject.registerClass(
class UtiIndicator extends PanelMenu.Button {
    _init(extension, busName) {
        super._init(0.0, 'uti', false);
        this._extension = extension;
        this._busName = busName;
        this._sniProxy = null;
        this._menuProxy = null;
        this._menuLayoutId = null;

        this._icon = new St.Icon({
            icon_name: 'edit-paste-symbolic',
            style_class: 'system-status-icon',
        });
        this.add_child(this._icon);

        this._connectSNI();

        // Left-click: toggle window
        this.connect('button-press-event', (actor, event) => {
            if (event.get_button() === 1) {
                this._onActivate();
                return Clutter.EVENT_STOP;
            }
            return Clutter.EVENT_PROPAGATE;
        });

        // Refresh menu when opened
        this.menu.connect('open-state-changed', (menu, open) => {
            if (open) this._loadMenu();
        });
    }

    _connectSNI() {
        try {
            this._sniProxy = new SNIProxy(
                Gio.DBus.session,
                this._busName,
                '/StatusNotifierItem'
            );

            // Set icon
            const iconName = this._sniProxy.IconName;
            if (iconName) this._icon.icon_name = iconName;

            // Connect to menu
            const menuPath = this._sniProxy.Menu;
            if (menuPath) {
                this._menuProxy = new DBusMenuProxy(
                    Gio.DBus.session,
                    this._busName,
                    menuPath
                );
                this._menuLayoutId = this._menuProxy.connectSignal(
                    'LayoutUpdated',
                    () => { if (this.menu.isOpen) this._loadMenu(); }
                );
            }

            console.log(`[uti] Connected to SNI: ${this._busName}`);
        } catch (e) {
            console.error(`[uti] SNI connection failed: ${e.message}`);
        }
    }

    _onActivate() {
        if (this._sniProxy) {
            try {
                const [x, y] = global.get_pointer();
                this._sniProxy.ActivateRemote(x, y);
                // Position window at cursor after activation
                GLib.timeout_add(GLib.PRIORITY_DEFAULT, 50, () => {
                    const win = this._extension._findUtiWindow();
                    if (win) this._extension._moveWindowToCursor(win, x, y);
                    return GLib.SOURCE_REMOVE;
                });
            } catch (e) {
                console.error(`[uti] Activate failed: ${e.message}`);
            }
        }
    }

    _loadMenu() {
        if (!this._menuProxy) return;

        this._menuProxy.GetLayoutRemote(0, -1, [], (result, error) => {
            if (error) {
                console.error(`[uti] GetLayout error: ${error.message}`);
                return;
            }
            this.menu.removeAll();
            const [, layout] = result;
            this._buildMenu(layout);
        });
    }

    _buildMenu(layout) {
        const [, , children] = layout;
        for (const child of children) {
            const item = child.recursiveUnpack();
            this._addMenuItem(item);
        }
    }

    _addMenuItem(item) {
        const [id, props, children] = item;
        const type = props['type'] || '';
        const label = (props['label'] || '').replace(/_/g, '');
        const enabled = props['enabled'] !== false;
        const visible = props['visible'] !== false;
        const toggleType = props['toggle-type'] || '';
        const toggleState = props['toggle-state'] || 0;

        if (!visible) return;

        if (type === 'separator') {
            this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
            return;
        }

        let menuItem;
        if (toggleType === 'checkmark') {
            menuItem = new PopupMenu.PopupSwitchMenuItem(label, toggleState === 1);
            menuItem.connect('toggled', () => this._emitEvent(id));
        } else {
            menuItem = new PopupMenu.PopupMenuItem(label);
            menuItem.connect('activate', () => this._emitEvent(id));
        }
        menuItem.setSensitive(enabled);
        this.menu.addMenuItem(menuItem);
    }

    _emitEvent(id) {
        if (!this._menuProxy) return;
        try {
            this._menuProxy.EventRemote(
                id, 'clicked',
                new GLib.Variant('i', 0),
                Math.floor(Date.now() / 1000)
            );
        } catch (e) {
            console.error(`[uti] Event failed: ${e.message}`);
        }
    }

    destroy() {
        if (this._menuLayoutId && this._menuProxy) {
            this._menuProxy.disconnectSignal(this._menuLayoutId);
        }
        this._sniProxy = null;
        this._menuProxy = null;
        super.destroy();
    }
});

export default class UtiExtension extends Extension {
    constructor(metadata) {
        super(metadata);
        this._indicator = null;
        this._dbusConnection = null;
        this._signalSubscriptionId = null;
        this._sniBusWatcherId = null;
        this._sniBusName = null;
    }

    enable() {
        console.log('[uti] Extension enabled');
        this._connectToDbus();
        this._watchForSNI();
    }

    disable() {
        console.log('[uti] Extension disabled');
        this._removeIndicator();
        this._unwatchSNI();
        this._disconnectFromDbus();
    }

    /**
     * Watch for uti's StatusNotifierItem on D-Bus
     */
    _watchForSNI() {
        // Watch for new bus names matching org.kde.StatusNotifierItem-*
        this._sniBusWatcherId = Gio.DBus.session.signal_subscribe(
            'org.freedesktop.DBus',
            'org.freedesktop.DBus',
            'NameOwnerChanged',
            '/org/freedesktop/DBus',
            null,
            Gio.DBusSignalFlags.NONE,
            this._onNameOwnerChanged.bind(this)
        );

        // Check existing names
        Gio.DBus.session.call(
            'org.freedesktop.DBus',
            '/org/freedesktop/DBus',
            'org.freedesktop.DBus',
            'ListNames',
            null,
            new GLib.VariantType('(as)'),
            Gio.DBusCallFlags.NONE,
            -1,
            null,
            (conn, result) => {
                try {
                    const [names] = conn.call_finish(result).deep_unpack();
                    for (const name of names) {
                        if (name.startsWith('org.kde.StatusNotifierItem-')) {
                            this._checkIfUtiSNI(name);
                        }
                    }
                } catch (e) {
                    console.error(`[uti] ListNames failed: ${e.message}`);
                }
            }
        );
    }

    _unwatchSNI() {
        if (this._sniBusWatcherId) {
            Gio.DBus.session.signal_unsubscribe(this._sniBusWatcherId);
            this._sniBusWatcherId = null;
        }
    }

    _onNameOwnerChanged(conn, sender, path, iface, signal, params) {
        const [name, oldOwner, newOwner] = params.deep_unpack();

        if (!name.startsWith('org.kde.StatusNotifierItem-')) return;

        if (newOwner && !oldOwner) {
            // New SNI appeared
            this._checkIfUtiSNI(name);
        } else if (oldOwner && !newOwner && name === this._sniBusName) {
            // Our SNI disappeared
            console.log(`[uti] SNI vanished: ${name}`);
            this._removeIndicator();
            this._sniBusName = null;
        }
    }

    _checkIfUtiSNI(busName) {
        // Check if this SNI belongs to uti by reading its Id property
        try {
            const proxy = new SNIProxy(
                Gio.DBus.session,
                busName,
                '/StatusNotifierItem'
            );
            const id = proxy.Id;
            if (id && id.toLowerCase() === 'uti') {
                console.log(`[uti] Found uti SNI: ${busName}`);
                this._sniBusName = busName;
                this._createIndicator(busName);
            }
        } catch (e) {
            // Not our SNI, ignore
        }
    }

    _createIndicator(busName) {
        if (this._indicator) return;
        this._indicator = new UtiIndicator(this, busName);
        Main.panel.addToStatusArea(this.uuid, this._indicator);
        console.log('[uti] Indicator created');
    }

    _removeIndicator() {
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
            console.log('[uti] Indicator removed');
        }
    }

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
            console.log('[uti] D-Bus connected');
        } catch (e) {
            console.error(`[uti] D-Bus failed: ${e.message}`);
        }
    }

    _disconnectFromDbus() {
        if (this._signalSubscriptionId && this._dbusConnection) {
            this._dbusConnection.signal_unsubscribe(this._signalSubscriptionId);
            this._signalSubscriptionId = null;
        }
        this._dbusConnection = null;
    }

    _onTriggered() {
        console.log('[uti] Triggered signal received');
        const [x, y] = global.get_pointer();
        const window = this._findUtiWindow();
        if (window) {
            this._moveWindowToCursor(window, x, y);
            this._activateWindow(window);
        } else {
            GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                const win = this._findUtiWindow();
                if (win) {
                    const [cx, cy] = global.get_pointer();
                    this._moveWindowToCursor(win, cx, cy);
                }
                return GLib.SOURCE_REMOVE;
            });
        }
    }

    _findUtiWindow() {
        for (const actor of global.get_window_actors()) {
            const win = actor.get_meta_window();
            if (win) {
                const wmClass = win.get_wm_class();
                if (wmClass && wmClass.toLowerCase() === UTI_WM_CLASS) {
                    return win;
                }
            }
        }
        return null;
    }

    _moveWindowToCursor(window, x, y) {
        const rect = window.get_frame_rect();
        const workArea = Main.layoutManager.getWorkAreaForMonitor(window.get_monitor());

        let newX = x, newY = y;
        if (newX + rect.width > workArea.x + workArea.width)
            newX = workArea.x + workArea.width - rect.width;
        if (newY + rect.height > workArea.y + workArea.height)
            newY = workArea.y + workArea.height - rect.height;
        if (newX < workArea.x) newX = workArea.x;
        if (newY < workArea.y) newY = workArea.y;

        window.move_frame(true, newX, newY);
    }

    _activateWindow(window) {
        window.activate(global.get_current_time());
    }
}
