Name:           uti-daemon
Version:        0.0.6
Release:        1
Summary:        uti keyboard daemon for hotkey detection

License:        MIT
URL:            https://github.com/noppomario/uti

# Build requires Rust toolchain (not included in BuildRequires for portability)

# Define _userunitdir if not already defined (for Ubuntu/Debian compatibility)
%{!?_userunitdir: %define _userunitdir /usr/lib/systemd/user}

%description
A daemon that monitors keyboard events and sends D-Bus signals
on double key press (e.g., double Ctrl). Part of the uti desktop utility.

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_userunitdir}
install -m 755 %{_sourcedir}/uti-daemon %{buildroot}%{_bindir}/
install -m 644 %{_sourcedir}/uti-daemon.service %{buildroot}%{_userunitdir}/

%files
%{_bindir}/uti-daemon
%{_userunitdir}/uti-daemon.service

%post
# Reload and restart if already enabled (for upgrades)
if systemctl --user is-enabled uti-daemon.service >/dev/null 2>&1; then
    systemctl --user daemon-reload
    systemctl --user restart uti-daemon.service
fi

%preun
# Stop service before uninstalling
systemctl --user stop uti-daemon.service 2>/dev/null || true
systemctl --user disable uti-daemon.service 2>/dev/null || true
