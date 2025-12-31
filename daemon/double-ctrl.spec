Name:           double-ctrl
Version:        0.0.1
Release:        1
Summary:        Hotkey daemon for double key press detection

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
install -m 755 %{_sourcedir}/double-ctrl %{buildroot}%{_bindir}/
install -m 644 %{_sourcedir}/double-ctrl.service %{buildroot}%{_userunitdir}/

%files
%{_bindir}/double-ctrl
%{_userunitdir}/double-ctrl.service

%post
echo ""
echo "==========================================="
echo " double-ctrl installation complete!"
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
echo "    systemctl --user enable --now double-ctrl.service"
echo ""

%preun
# Stop service before uninstalling
systemctl --user stop double-ctrl.service 2>/dev/null || true
systemctl --user disable double-ctrl.service 2>/dev/null || true
