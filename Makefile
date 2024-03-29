output: target/release/tap

.PHONY: install clean man

VERSION := $(shell git tag | tail -n 1 | tr -d v)

INSTALL_DIR = /usr/local/bin
MAN_DIR = /usr/local/share/man

target/release/tap: src
	@cargo build --release

install: /usr/local/bin/tap man
/usr/local/bin/tap: target/release/tap
	@install -pm755 target/release/tap $(INSTALL_DIR)

target/release/man: doc/tap.1
	@echo '.TH "TAP" "1" "2023" "$(VERSION)" "User Commands"' > target/release/man
	@cat doc/tap.1 >> target/release/man

man: /usr/local/share/man/man1
$(MAN_DIR)/man1: target/release/man
	@mkdir -p $(MAN_DIR)/man1
	@install -pm644 target/release/man $(MAN_DIR)/man1/tap.1
	@$(shell command -v mandb >/dev/null && mandb )
	@$(shell command -v makewhatis >/dev/null && makewhatis )

clean:
	@cargo clean

uninstall: clean
	@$(RM) $(INSTALL_DIR)/tap
	@$(RM) $(MAN_DIR)/man1/tap.1
