# One source package for the app, the daemon and the GNOME extension.
# The source tarball (packaging/make-srpm.sh) is `git archive` plus
# `cargo vendor`, so the build runs offline against Cargo.lock.

%global ext_uuid uti@noppomario.github.io

Name:           uti
Version:        0.4.0
Release:        1%{?dist}
Summary:        Clipboard history, snippets and launcher on double Ctrl

# uti itself; vendored crates are limited to the licenses allowed in deny.toml.
License:        MIT
URL:            https://github.com/noppomario/uti
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  pkgconfig(gtk4) >= 4.22
BuildRequires:  pkgconfig(libadwaita-1) >= 1.9
BuildRequires:  systemd-rpm-macros
BuildRequires:  desktop-file-utils
BuildRequires:  libappstream-glib

# The minimum versions match the gtk4-rs / libadwaita-rs features (gnome_50, v1_9).
Requires:       gtk4 >= 4.22
Requires:       libadwaita >= 1.9
Recommends:     uti-daemon = %{version}-%{release}
Recommends:     gnome-shell-extension-uti = %{version}-%{release}

%description
GTK4 / libadwaita popup for GNOME with clipboard history, snippets and an
app launcher. Opened by uti-daemon on double Ctrl; positioned by the
"uti for GNOME" extension.

%package -n uti-daemon
Summary:        uti keyboard daemon for double Ctrl detection
Requires:       %{name}%{?_isa} = %{version}-%{release}

%description -n uti-daemon
Monitors keyboards via evdev and notifies uti over D-Bus on double Ctrl.
Also provides the virtual keyboard used for auto-paste. Requires membership
in the input group.

%package -n gnome-shell-extension-uti
Summary:        uti for GNOME (panel menu and window positioning)
BuildArch:      noarch
Requires:       gnome-shell >= 50
Requires:       %{name} = %{version}-%{release}

%description -n gnome-shell-extension-uti
GNOME Shell extension for uti: panel menu, cursor-relative positioning,
always-on-top, hiding from the dash, paste and clipboard watch.

%prep
%autosetup -p1
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

%build
cargo build --release --offline --locked --workspace

%install
install -Dm755 target/release/uti target/release/uti-daemon -t %{buildroot}%{_bindir}
install -Dm644 app/packaging/io.github.noppomario.uti.desktop \
    -t %{buildroot}%{_datadir}/applications
install -Dm644 app/packaging/io.github.noppomario.uti.metainfo.xml \
    -t %{buildroot}%{_metainfodir}
cp -r app/packaging/icons %{buildroot}%{_datadir}/
install -Dm644 daemon/systemd/uti-daemon.service -t %{buildroot}%{_userunitdir}
install -Dm644 daemon/udev/99-uti-uinput.rules -t %{buildroot}%{_udevrulesdir}
install -Dm644 gnome-extension/metadata.json gnome-extension/extension.js \
    -t %{buildroot}%{_datadir}/gnome-shell/extensions/%{ext_uuid}
install -Dm644 gnome-extension/schemas/*.gschema.xml \
    -t %{buildroot}%{_datadir}/glib-2.0/schemas

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/*.desktop
appstream-util validate-relax --nonet %{buildroot}%{_metainfodir}/*.metainfo.xml

%post -n uti-daemon
%systemd_user_post uti-daemon.service
# Apply the uinput group/mode to the existing device node.
udevadm trigger --subsystem-match=misc --sysname-match=uinput 2>/dev/null || :

%preun -n uti-daemon
%systemd_user_preun uti-daemon.service

%postun -n uti-daemon
%systemd_user_postun_with_restart uti-daemon.service

%files
%license LICENSE
%{_bindir}/uti
%{_datadir}/applications/io.github.noppomario.uti.desktop
%{_metainfodir}/io.github.noppomario.uti.metainfo.xml
%{_datadir}/icons/hicolor/*/apps/io.github.noppomario.uti.png

%files -n uti-daemon
%license LICENSE
%{_bindir}/uti-daemon
%{_userunitdir}/uti-daemon.service
%{_udevrulesdir}/99-uti-uinput.rules

%files -n gnome-shell-extension-uti
%license LICENSE
%{_datadir}/gnome-shell/extensions/%{ext_uuid}/
%{_datadir}/glib-2.0/schemas/org.gnome.shell.extensions.uti.gschema.xml
