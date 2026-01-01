/**
 * uti GNOME Shell Extension
 *
 * Features:
 * 1. StatusNotifierWatcher service (replaces AppIndicator extension)
 * 2. StatusNotifierItem host for uti's tray icon
 * 3. Positions uti window at cursor location on Ctrl double-tap
 *
 * Architecture:
 *   daemon (evdev) --D-Bus--> Extension --move window--> Tauri app
 *   Tauri app --RegisterStatusNotifierItem--> Extension (Watcher)
 *   Extension (Watcher) --create indicator--> GNOME Shell panel
 */

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';
import St from 'gi://St';
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
    <property name="Title" type="s" access="read"/>
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

// StatusNotifierWatcher D-Bus interface (we implement this as a host)
const SNWatcherIface = `
<node>
  <interface name="org.kde.StatusNotifierWatcher">
    <method name="RegisterStatusNotifierItem">
      <arg type="s" direction="in"/>
    </method>
    <signal name="StatusNotifierItemRegistered">
      <arg type="s"/>
    </signal>
    <signal name="StatusNotifierItemUnregistered">
      <arg type="s"/>
    </signal>
    <property name="RegisteredStatusNotifierItems" type="as" access="read"/>
    <property name="IsStatusNotifierHostRegistered" type="b" access="read"/>
  </interface>
</node>`;

/**
 * StatusNotifierWatcher implementation
 * Provides the org.kde.StatusNotifierWatcher D-Bus service so that
 * StatusNotifierItem clients (like Tauri's tray) can register with us.
 */
class StatusNotifierWatcher {
    constructor(extension) {
        this._extension = extension;
        this._registeredItems = [];
        this._nameOwnerId = null;
        this._dbusImpl = null;
        this._nameWatchers = new Map();
    }

    enable() {
        const nodeInfo = Gio.DBusNodeInfo.new_for_xml(SNWatcherIface);
        this._dbusImpl = Gio.DBusExportedObject.wrapJSObject(
            nodeInfo.interfaces[0],
            this
        );

        try {
            this._dbusImpl.export(Gio.DBus.session, '/StatusNotifierWatcher');
        } catch (e) {
            console.error(`[uti] Failed to export StatusNotifierWatcher: ${e.message}`);
            return;
        }

        // Own the well-known name
        this._nameOwnerId = Gio.bus_own_name(
            Gio.BusType.SESSION,
            'org.kde.StatusNotifierWatcher',
            Gio.BusNameOwnerFlags.NONE,
            null, // bus acquired
            () => console.log('[uti] StatusNotifierWatcher name acquired'),
            () => console.log('[uti] StatusNotifierWatcher name lost')
        );

        console.log('[uti] StatusNotifierWatcher enabled');
    }

    disable() {
        if (this._nameOwnerId) {
            Gio.bus_unown_name(this._nameOwnerId);
            this._nameOwnerId = null;
        }

        // Unwatch all registered items
        for (const watchId of this._nameWatchers.values()) {
            Gio.bus_unwatch_name(watchId);
        }
        this._nameWatchers.clear();

        if (this._dbusImpl) {
            this._dbusImpl.unexport();
            this._dbusImpl = null;
        }

        this._registeredItems = [];
        console.log('[uti] StatusNotifierWatcher disabled');
    }

    // D-Bus method: RegisterStatusNotifierItem
    // Called via D-Bus with invocation object to get sender
    RegisterStatusNotifierItemAsync(params, invocation) {
        const [service] = params;
        const sender = invocation.get_sender();

        // Determine bus name and object path
        let busName;
        let objectPath;
        if (service.startsWith('/')) {
            // Service is an object path (ayatana-appindicator style)
            busName = sender;
            objectPath = service;
        } else if (service.startsWith(':')) {
            // Unique bus name
            busName = service;
            objectPath = '/StatusNotifierItem';
        } else {
            // Well-known name
            busName = service;
            objectPath = '/StatusNotifierItem';
        }

        const itemId = `${busName}${objectPath}`;

        // Avoid duplicates
        if (this._registeredItems.includes(itemId)) {
            console.log(`[uti] SNI already registered: ${itemId}`);
            invocation.return_value(null);
            return;
        }

        this._registeredItems.push(itemId);
        console.log(`[uti] SNI registered: ${itemId}`);

        // Watch for the bus name to vanish
        const watchId = Gio.bus_watch_name(
            Gio.BusType.SESSION,
            busName,
            Gio.BusNameWatcherFlags.NONE,
            null,
            () => this._onItemVanished(itemId)
        );
        this._nameWatchers.set(itemId, watchId);

        // Emit signal
        this._dbusImpl.emit_signal(
            'StatusNotifierItemRegistered',
            new GLib.Variant('(s)', [itemId])
        );

        // Notify extension to check this item
        this._extension._checkIfUtiSNI(busName, objectPath);

        invocation.return_value(null);
    }

    _onItemVanished(itemId) {
        const index = this._registeredItems.indexOf(itemId);
        if (index >= 0) {
            this._registeredItems.splice(index, 1);
            console.log(`[uti] SNI unregistered: ${itemId}`);

            // Emit signal
            this._dbusImpl.emit_signal(
                'StatusNotifierItemUnregistered',
                new GLib.Variant('(s)', [itemId])
            );

            // Notify extension to remove indicator if it was uti's
            this._extension._onSNIVanished(itemId);
        }

        // Stop watching
        const watchId = this._nameWatchers.get(itemId);
        if (watchId) {
            Gio.bus_unwatch_name(watchId);
            this._nameWatchers.delete(itemId);
        }
    }

    // D-Bus property: RegisteredStatusNotifierItems
    get RegisteredStatusNotifierItems() {
        return this._registeredItems;
    }

    // D-Bus property: IsStatusNotifierHostRegistered
    get IsStatusNotifierHostRegistered() {
        return true; // We are the host
    }
}

/**
 * Panel indicator that displays uti's StatusNotifierItem
 */
const UtiIndicator = GObject.registerClass(
class UtiIndicator extends PanelMenu.Button {
    _init(extension, busName, objectPath) {
        super._init(0.0, 'uti', false);
        this._extension = extension;
        this._busName = busName;
        this._objectPath = objectPath;
        this._sniProxy = null;
        this._menuProxy = null;
        this._menuLayoutId = null;

        this._icon = new St.Icon({
            icon_name: 'edit-paste-symbolic',
            style_class: 'system-status-icon',
        });
        this.add_child(this._icon);

        this._connectSNI();

        // Refresh menu when opened
        this.menu.connect('open-state-changed', (_menu, open) => {
            if (open) this._loadMenu();
        });

        // Load menu initially so it's not empty
        this._loadMenu();
    }

    _connectSNI() {
        try {
            this._sniProxy = new SNIProxy(
                Gio.DBus.session,
                this._busName,
                this._objectPath
            );

            // Set icon - IconName may be a file path or theme icon name
            const iconName = this._sniProxy.IconName;
            if (iconName) {
                if (iconName.startsWith('/')) {
                    // File path - create GIcon from file
                    const file = Gio.File.new_for_path(iconName);
                    if (file.query_exists(null)) {
                        this._icon.gicon = Gio.FileIcon.new(file);
                    }
                } else {
                    // Theme icon name
                    this._icon.icon_name = iconName;
                }
            }

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

            console.log(`[uti] Connected to SNI: ${this._busName} at ${this._objectPath}`);
        } catch (e) {
            console.error(`[uti] SNI connection failed: ${e.message}`);
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
        this._settings = null;
        this._settingsChangedId = null;
        this._sniWatcher = null;
    }

    enable() {
        console.log('[uti] Extension enabled');

        // Load settings
        this._settings = this.getSettings();
        this._settingsChangedId = this._settings.connect(
            'changed::enable-tray-icon',
            this._onSettingsChanged.bind(this)
        );

        // Always connect to D-Bus for cursor positioning
        this._connectToDbus();

        // Start StatusNotifierWatcher if tray icon is enabled
        if (this._settings.get_boolean('enable-tray-icon')) {
            this._sniWatcher = new StatusNotifierWatcher(this);
            this._sniWatcher.enable();
            this._watchForSNI();
        }
    }

    disable() {
        console.log('[uti] Extension disabled');
        this._removeIndicator();
        this._unwatchSNI();
        this._disconnectFromDbus();

        if (this._sniWatcher) {
            this._sniWatcher.disable();
            this._sniWatcher = null;
        }

        if (this._settingsChangedId && this._settings) {
            this._settings.disconnect(this._settingsChangedId);
            this._settingsChangedId = null;
        }
        this._settings = null;
    }

    _onSettingsChanged() {
        const enabled = this._settings.get_boolean('enable-tray-icon');
        console.log(`[uti] Tray icon setting changed: ${enabled}`);

        if (enabled) {
            if (!this._sniWatcher) {
                this._sniWatcher = new StatusNotifierWatcher(this);
                this._sniWatcher.enable();
            }
            this._watchForSNI();
        } else {
            this._removeIndicator();
            this._unwatchSNI();
            this._sniBusName = null;
            if (this._sniWatcher) {
                this._sniWatcher.disable();
                this._sniWatcher = null;
            }
        }
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

    _checkIfUtiSNI(busName, objectPath = '/StatusNotifierItem') {
        // Check if this SNI belongs to uti by reading its Title property
        // (ayatana-appindicator uses Title='uti', Id='tray-icon tray app')
        try {
            const proxy = new SNIProxy(
                Gio.DBus.session,
                busName,
                objectPath
            );
            const title = proxy.Title;
            const id = proxy.Id;
            console.log(`[uti] Checking SNI: busName=${busName}, path=${objectPath}, Title=${title}, Id=${id}`);
            if ((title && title.toLowerCase() === 'uti') ||
                (id && id.toLowerCase() === 'uti')) {
                console.log(`[uti] Found uti SNI: ${busName} at ${objectPath}`);
                this._sniBusName = busName;
                this._sniObjectPath = objectPath;
                this._createIndicator(busName, objectPath);
            }
        } catch (e) {
            console.log(`[uti] SNI check failed: ${e.message}`);
        }
    }

    _createIndicator(busName, objectPath) {
        if (this._indicator) return;
        this._indicator = new UtiIndicator(this, busName, objectPath);
        Main.panel.addToStatusArea(this.uuid, this._indicator);
        console.log('[uti] Indicator created');
    }

    _onSNIVanished(itemId) {
        // Check if the vanished SNI was uti's
        if (this._sniBusName && this._sniObjectPath) {
            const ourItemId = `${this._sniBusName}${this._sniObjectPath}`;
            if (itemId === ourItemId) {
                console.log(`[uti] Our SNI vanished: ${itemId}`);
                this._removeIndicator();
                this._sniBusName = null;
                this._sniObjectPath = null;
            }
        }
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

        // Calculate space available in each direction from cursor
        const spaceRight = workArea.x + workArea.width - x;
        const spaceLeft = x - workArea.x;
        const spaceBottom = workArea.y + workArea.height - y;
        const spaceTop = y - workArea.y;

        // Determine horizontal position: prefer right, flip to left if not enough space
        let newX;
        if (rect.width <= spaceRight) {
            newX = x; // Place to the right of cursor
        } else if (rect.width <= spaceLeft) {
            newX = x - rect.width; // Flip: place to the left of cursor
        } else {
            // Not enough space on either side, clamp to workArea
            newX = Math.max(workArea.x, workArea.x + workArea.width - rect.width);
        }

        // Determine vertical position: prefer below, flip to above if not enough space
        let newY;
        if (rect.height <= spaceBottom) {
            newY = y; // Place below cursor
        } else if (rect.height <= spaceTop) {
            newY = y - rect.height; // Flip: place above cursor
        } else {
            // Not enough space on either side, clamp to workArea
            newY = Math.max(workArea.y, workArea.y + workArea.height - rect.height);
        }

        window.move_frame(true, newX, newY);
    }

    _activateWindow(window) {
        window.activate(global.get_current_time());
    }
}
