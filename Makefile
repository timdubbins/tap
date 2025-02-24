output: target/release/tap

.PHONY: install clean man uninstall

VERSION := $(shell git tag | tail -n 1 | tr -d v)

# Allow users to specify an installation prefix (default: /usr/local)
PREFIX ?= /usr/local
INSTALL_DIR = $(PREFIX)/bin
MAN_DIR = $(PREFIX)/share/man

CACHE_FILE = $(HOME)/.cache/tap/data

# Ensure XDG_CONFIG_HOME is set, or default to ~/.config
XDG_CONFIG_HOME ?= $(HOME)/.config

CONFIG_FILES := \
	$(XDG_CONFIG_HOME)/tap/tap.yml \
	$(XDG_CONFIG_HOME)/tap.yml \
	$(HOME)/.config/tap/tap.yml \
	$(HOME)/.tap.yml

target/release/tap: src
	@cargo build --release

install: $(INSTALL_DIR)/tap man
$(INSTALL_DIR)/tap: target/release/tap
	@mkdir -p $(INSTALL_DIR)
	@install -m755 target/release/tap $(INSTALL_DIR)/tap

man: $(MAN_DIR)/man1
$(MAN_DIR)/man1: doc/tap.1
	@mkdir -p $(MAN_DIR)/man1
	@install -m644 doc/tap.1 $(MAN_DIR)/man1/tap.1

clean:
	@cargo clean

uninstall:
	@echo "Removing tap binary..."
	@if [ -f "$(INSTALL_DIR)/tap" ]; then $(RM) "$(INSTALL_DIR)/tap"; fi

	@echo "Removing man page..."
	@if [ -f "$(MAN_DIR)/man1/tap.1" ]; then $(RM) "$(MAN_DIR)/man1/tap.1"; fi

	@echo "Removing cache file..."
	@if [ -f "$(CACHE_FILE)" ]; then $(RM) "$(CACHE_FILE)"; fi

	@echo "Removing configuration files..."
	@for config in $(CONFIG_FILES); do \
		if [ -f "$$config" ]; then \
			echo "Removing $$config"; \
			$(RM) "$$config"; \
		fi \
	done

	@echo "Uninstall complete."
