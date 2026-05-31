BINARY    := vector-screen
TARGET    := armv7-unknown-linux-gnueabihf
PREFIX    ?= /usr/local
BINDIR    := $(PREFIX)/bin
SYSTEMD   := /etc/systemd/system

# Cargo output paths
DEBUG_BIN  := target/debug/$(BINARY)
RELEASE_BIN := target/$(TARGET)/release/$(BINARY)

.PHONY: build release clean install uninstall test check help

## Build debug binary for native development
build:
	cargo build

## Cross-compile release binary for ARM (Raspberry Pi / Adventurer 5M)
release:
	cargo build --release --target $(TARGET)

## Build and run tests
test:
	cargo test

## Check code compiles without producing binary
check:
	cargo check

## Clean build artifacts
clean:
	cargo clean

## Install release binary and systemd service (requires root)
install: release
	install -Dm755 $(RELEASE_BIN) $(DESTDIR)$(BINDIR)/$(BINARY)
	@echo "Installed $(BINARY) to $(DESTDIR)$(BINDIR)/$(BINARY)"
	@if [ -f $(BINARY).service ]; then \
		install -Dm644 $(BINARY).service $(DESTDIR)$(SYSTEMD)/$(BINARY).service; \
		echo "Installed systemd service to $(DESTDIR)$(SYSTEMD)/$(BINARY).service"; \
		echo "Run 'sudo systemctl enable --now $(BINARY)' to start"; \
	fi

## Uninstall binary and service
uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(BINARY)
	rm -f $(DESTDIR)$(SYSTEMD)/$(BINARY).service
	@echo "Uninstalled $(BINARY)"

## Show this help
help:
	@echo "VectorScreen Build System"
	@echo ""
	@echo "Targets:"
	@sed -n 's/^## //p' $(MAKEFILE_LIST) | column -t -s '	' 2>/dev/null || sed -n 's/^## //p' $(MAKEFILE_LIST)
