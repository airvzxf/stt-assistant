PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share

# Arch Linux convention: /usr/lib/systemd/user for package installed units
SYSTEMD_USER_DIR ?= $(PREFIX)/lib/systemd/user

# Detect source of binaries (Source build > Container build)
ifneq ("$(wildcard target/release/stt-daemon)","")
	DAEMON_BIN = target/release/stt-daemon
	CLIENT_BIN = target/release/stt-client
else
	DAEMON_BIN = bin/stt-daemon
	CLIENT_BIN = bin/stt-client
endif

.PHONY: all build clean install

all: build

build:
	@echo "Note: Use ./scripts/build for containerized build."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo build --release; \
	else \
		echo "Cargo not found. Skipping local build. Ensure binaries are in bin/ directory."; \
	fi

clean:
	cargo clean || true
	rm -rf bin/

install:
	@if [ ! -f "$(DAEMON_BIN)" ]; then echo "Error: $(DAEMON_BIN) not found. Run make build or ./scripts/build"; exit 1; fi
	install -Dm755 $(DAEMON_BIN) $(DESTDIR)$(BINDIR)/stt-daemon
	install -Dm755 $(CLIENT_BIN) $(DESTDIR)$(BINDIR)/stt-client
	install -Dm644 systemd/stt-daemon.service $(DESTDIR)$(SYSTEMD_USER_DIR)/stt-daemon.service
	install -Dm644 systemd/stt-assistant.service $(DESTDIR)$(SYSTEMD_USER_DIR)/stt-assistant.service
	mkdir -p $(DESTDIR)$(DATADIR)/stt-assistant/models
	if [ -f "models/ggml-base.bin" ]; then \
		install -Dm644 models/ggml-base.bin $(DESTDIR)$(DATADIR)/stt-assistant/models/ggml-base.bin; \
	fi
