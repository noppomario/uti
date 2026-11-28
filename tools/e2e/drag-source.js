// Another app to drag from in e2e checks: one row per item of the JSON array
// in argv[1], [{label, formats: {mime: text}}], offered as those MIME types
// (Nautilus offers files as text/uri-list, a browser links as text/uri-list
// and text/plain). Writes the pointer position over its rows to argv[2] on
// every motion, so a scenario can find where the window is, and prints
// `drag-end delete=<bool>` after each drag (true: the target asked to move).

import Gdk from 'gi://Gdk?version=4.0';
import GLib from 'gi://GLib';
import Gtk from 'gi://Gtk?version=4.0';
import System from 'system';

const [itemsJson, probePath] = ARGV;
const items = JSON.parse(itemsJson);
/** Height of each row; rows are 260 px wide. */
const ROW_HEIGHT = 48;

const app = new Gtk.Application({application_id: 'io.github.noppomario.uti.e2e.DragSource'});
app.connect('activate', () => {
    const box = new Gtk.Box({orientation: Gtk.Orientation.VERTICAL});
    for (const item of items) {
        const row = new Gtk.Label({label: item.label, height_request: ROW_HEIGHT, width_request: 260});
        const providers = Object.entries(item.formats).map(([mime, text]) =>
            Gdk.ContentProvider.new_for_bytes(mime, new GLib.Bytes(new TextEncoder().encode(text))),
        );
        const source = new Gtk.DragSource({actions: Gdk.DragAction.COPY | Gdk.DragAction.MOVE});
        source.connect('prepare', () => Gdk.ContentProvider.new_union(providers));
        source.connect('drag-end', (_, _drag, deleteData) => printerr(`drag-end delete=${deleteData}`));
        row.add_controller(source);
        box.append(row);
    }
    const motion = new Gtk.EventControllerMotion();
    motion.connect('motion', (_, x, y) => {
        GLib.file_set_contents(probePath, JSON.stringify([Math.round(x), Math.round(y)]));
    });
    box.add_controller(motion);
    const window = new Gtk.ApplicationWindow({application: app, title: 'drag source', child: box});
    // Without a titlebar, pressing anywhere in the window moves it.
    window.set_titlebar(new Gtk.HeaderBar());
    window.present();
});
app.run([System.programInvocationName]);
