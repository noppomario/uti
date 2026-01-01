Name:           uti-daemon
Version:        0.0.4
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
echo ""
echo "==========================================="
echo " uti-daemon installation complete!"
echo "==========================================="
echo ""
echo "IMPORTANT: You must add your user to the 'input' group:"
echo ""
echo "    sudo usermod -aG input $USER"
echo ""
echo "Then log out and log back in for the change to take effect."
echo ""
echo "After that, enable and start the service:"
echo ""
echo "    systemctl --user daemon-reload"
echo "    systemctl --user enable --now uti-daemon.service"
echo ""

%preun
# Stop service before uninstalling
systemctl --user stop uti-daemon.service 2>/dev/null || true
systemctl --user disable uti-daemon.service 2>/dev/null || true
