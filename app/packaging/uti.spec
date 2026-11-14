Name:           uti
Version:        0.2.0
Release:        1
Summary:        Clipboard history, snippets and launcher on double Ctrl

License:        MIT
URL:            https://github.com/noppomario/uti

# Prebuilt binary from `cargo build --release` (see app/packaging/build-rpm.sh).
# rpmbuild adds the library requirements; the minimum versions match the
# gtk4-rs / libadwaita-rs features (gnome_50, v1_9).
Requires:       gtk4 >= 4.22
Requires:       libadwaita >= 1.9

%description
GTK4 / libadwaita popup for GNOME with clipboard history, snippets and an
app launcher. Opened by uti-daemon on double Ctrl; positioned by the
"uti for GNOME" extension.

%install
install -Dm755 %{_sourcedir}/uti %{buildroot}%{_bindir}/uti
install -Dm644 %{_sourcedir}/io.github.noppomario.uti.desktop \
    %{buildroot}%{_datadir}/applications/io.github.noppomario.uti.desktop
install -Dm644 %{_sourcedir}/io.github.noppomario.uti.metainfo.xml \
    %{buildroot}%{_metainfodir}/io.github.noppomario.uti.metainfo.xml
cp -r %{_sourcedir}/icons %{buildroot}%{_datadir}/

%files
%{_bindir}/uti
%{_datadir}/applications/io.github.noppomario.uti.desktop
%{_metainfodir}/io.github.noppomario.uti.metainfo.xml
%{_datadir}/icons/hicolor/*/apps/io.github.noppomario.uti.png
