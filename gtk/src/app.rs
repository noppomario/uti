//! Main popup window component

use relm4::gtk::{self, gio, glib, prelude::*};
use relm4::prelude::*;

use crate::dbus;
use crate::state::window::{WindowCommand, WindowState};

/// Title the GNOME extension uses to find the main window.
pub const MAIN_WINDOW_TITLE: &str = "uti";
const RELEASES_URL: &str = "https://github.com/noppomario/uti/releases";

/// Messages handled by the main window.
#[derive(Debug)]
pub enum Msg {
    Toggle,
    Show,
    Hide,
    /// Internal: the extension has the position request; show the window now.
    Present,
    FocusChanged(bool),
    SetPinned(bool),
    OpenSettings,
    CheckUpdates,
    Quit,
}

/// Startup options parsed from the command line.
#[derive(Debug, Default)]
pub struct Init {
    pub minimized: bool,
}

/// Main window model.
pub struct App {
    root: gtk::ApplicationWindow,
    window: WindowState,
    conn: Option<gio::DBusConnection>,
    _subscriptions: Vec<gio::SignalSubscription>,
}

/// Widgets updated from the model.
pub struct Widgets {
    pin: gtk::ToggleButton,
}

impl SimpleComponent for App {
    type Init = Init;
    type Input = Msg;
    type Output = ();
    type Root = gtk::ApplicationWindow;
    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        gtk::ApplicationWindow::builder()
            .title(MAIN_WINDOW_TITLE)
            .decorated(false)
            .resizable(false)
            .default_width(500)
            .default_height(700)
            .build()
    }

    fn init(init: Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let stack = gtk::Stack::new();
        for (name, title) in [
            ("prompt", "Prompt"),
            ("clipboard", "Clipboard"),
            ("snippets", "Snippets"),
            ("launcher", "Launcher"),
        ] {
            stack.add_titled(&gtk::Label::new(Some(title)), Some(name), title);
        }
        stack.set_visible_child_name("clipboard");
        stack.set_vexpand(true);

        let pin = gtk::ToggleButton::builder()
            .icon_name("view-pin-symbolic")
            .tooltip_text("Pin window")
            .build();
        let s = sender.clone();
        pin.connect_toggled(move |b| s.input(Msg::SetPinned(b.is_active())));

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let switcher = gtk::StackSwitcher::builder().stack(&stack).hexpand(true).build();
        header.append(&switcher);
        header.append(&pin);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content.append(&header);
        content.append(&stack);
        root.set_child(Some(&content));

        let s = sender.clone();
        root.connect_is_active_notify(move |w| s.input(Msg::FocusChanged(w.is_active())));
        let s = sender.clone();
        root.connect_close_request(move |_| {
            s.input(Msg::Hide);
            glib::Propagation::Stop
        });

        let app = relm4::main_application();
        register_actions(&app, &sender);
        let s = sender.clone();
        app.connect_activate(move |_| s.input(Msg::Show));

        let conn = app.dbus_connection();
        let mut subscriptions = Vec::new();
        if let Some(conn) = &conn {
            let s = sender.clone();
            subscriptions.push(dbus::on_triggered(conn, move || s.input(Msg::Toggle)));
        } else {
            eprintln!("No D-Bus connection: double Ctrl and extension features disabled");
        }

        if !init.minimized {
            sender.input(Msg::Show);
        }

        let model = App {
            root: root.clone(),
            window: WindowState::new(),
            conn,
            _subscriptions: subscriptions,
        };
        ComponentParts {
            model,
            widgets: Widgets { pin },
        }
    }

    fn update(&mut self, msg: Msg, sender: ComponentSender<Self>) {
        let command = match msg {
            Msg::Toggle => self.window.toggle(),
            Msg::Show => self.window.show(),
            Msg::Hide => self.window.hide(),
            Msg::FocusChanged(false) => self.window.focus_lost(),
            Msg::FocusChanged(true) => None,
            Msg::SetPinned(pinned) => self.window.set_pinned(pinned),
            Msg::Present => {
                self.root.present();
                None
            },
            Msg::OpenSettings => {
                eprintln!("Settings window is not implemented yet");
                None
            },
            Msg::CheckUpdates => {
                gtk::UriLauncher::new(RELEASES_URL).launch(
                    Some(&self.root),
                    None::<&gio::Cancellable>,
                    |r| {
                        if let Err(e) = r {
                            eprintln!("Failed to open {RELEASES_URL}: {e}");
                        }
                    },
                );
                None
            },
            Msg::Quit => {
                relm4::main_application().quit();
                None
            },
        };
        if let Some(command) = command {
            self.apply(command, &sender);
        }
    }

    fn update_view(&self, widgets: &mut Widgets, _sender: ComponentSender<Self>) {
        if widgets.pin.is_active() != self.window.is_pinned() {
            widgets.pin.set_active(self.window.is_pinned());
        }
    }
}

impl App {
    fn apply(&self, command: WindowCommand, sender: &ComponentSender<Self>) {
        match command {
            WindowCommand::PositionAndPresent => match self.conn.clone() {
                Some(conn) => {
                    let s = sender.clone();
                    relm4::spawn_local(async move {
                        dbus::position_at_pointer(&conn).await;
                        s.input(Msg::Present);
                    });
                },
                None => self.root.present(),
            },
            WindowCommand::Hide => self.root.set_visible(false),
            WindowCommand::SetAbove(above) => {
                if let Some(conn) = self.conn.clone() {
                    relm4::spawn_local(async move { dbus::set_above(&conn, above).await });
                }
            },
        }
    }
}

/// An action name and the message it sends.
type ActionSpec = (&'static str, fn() -> Msg);

/// Registers the GActions the extension's panel menu invokes via
/// `org.freedesktop.Application.ActivateAction`.
fn register_actions(app: &impl IsA<gio::ActionMap>, sender: &ComponentSender<App>) {
    let actions: [ActionSpec; 6] = [
        ("toggle", || Msg::Toggle),
        ("show", || Msg::Show),
        ("hide", || Msg::Hide),
        ("settings", || Msg::OpenSettings),
        ("check-updates", || Msg::CheckUpdates),
        ("quit", || Msg::Quit),
    ];
    for (name, make) in actions {
        let action = gio::SimpleAction::new(name, None);
        let s = sender.clone();
        action.connect_activate(move |_, _| s.input(make()));
        app.add_action(&action);
    }
}
