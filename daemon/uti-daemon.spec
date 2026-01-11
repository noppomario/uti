Name:           uti-daemon
Version:        0.1.1
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
Also provides virtual keyboard support for auto-paste functionality.

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_userunitdir}
mkdir -p %{buildroot}/etc/udev/rules.d
install -m 755 %{_sourcedir}/uti-daemon %{buildroot}%{_bindir}/
install -m 644 %{_sourcedir}/uti-daemon.service %{buildroot}%{_userunitdir}/
install -m 644 %{_sourcedir}/99-uti-uinput.rules %{buildroot}/etc/udev/rules.d/

%files
%{_bindir}/uti-daemon
%{_userunitdir}/uti-daemon.service
/etc/udev/rules.d/99-uti-uinput.rules

%post
# Reload udev rules for uinput access
udevadm control --reload-rules 2>/dev/null || true
udevadm trigger 2>/dev/null || true

# Reload and restart if already enabled (for upgrades)
if systemctl --user is-enabled uti-daemon.service >/dev/null 2>&1; then
    systemctl --user daemon-reload
    systemctl --user restart uti-daemon.service
fi

%preun
# Stop service before uninstalling
systemctl --user stop uti-daemon.service 2>/dev/null || true
systemctl --user disable uti-daemon.service 2>/dev/null || true

%postun
# Reload udev rules after complete removal (not on upgrade)
if [ $1 -eq 0 ]; then
    udevadm control --reload-rules 2>/dev/null || true
fi
