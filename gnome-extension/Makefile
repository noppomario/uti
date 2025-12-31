# GNOME Shell Extension Makefile
UUID = uti@noppomario.github.io
INSTALL_DIR = $(HOME)/.local/share/gnome-shell/extensions/$(UUID)

.PHONY: install uninstall reinstall enable disable logs

install:
	@echo "Installing $(UUID)..."
	@mkdir -p $(INSTALL_DIR)
	@cp metadata.json $(INSTALL_DIR)/
	@cp extension.js $(INSTALL_DIR)/
	@echo "Installed to $(INSTALL_DIR)"
	@echo ""
	@echo "To enable the extension:"
	@echo "  make enable"
	@echo ""
	@echo "Or restart GNOME Shell (X11: Alt+F2, 'r', Enter)"
	@echo "On Wayland: Log out and log back in"

uninstall:
	@echo "Uninstalling $(UUID)..."
	@rm -rf $(INSTALL_DIR)
	@echo "Done"

reinstall: uninstall install

enable:
	@gnome-extensions enable $(UUID)
	@echo "Extension enabled"

disable:
	@gnome-extensions disable $(UUID)
	@echo "Extension disabled"

logs:
	@echo "Showing GNOME Shell logs (Ctrl+C to stop)..."
	@journalctl -f -o cat /usr/bin/gnome-shell | grep -i uti

# Development: watch for changes and reinstall
dev:
	@echo "Watching for changes..."
	@while true; do \
		inotifywait -q -e modify extension.js metadata.json; \
		make reinstall; \
		echo "Reinstalled. Restart GNOME Shell to apply changes."; \
	done
