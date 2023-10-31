output: target/release/tap

.PHONY: install clean

INSTALL_DIR = /usr/local/bin

target/release/tap: src
	@cargo build --release

install: /usr/local/bin/tap
/usr/local/bin/tap: target/release/tap
	@install -pm755 target/release/tap $(INSTALL_DIR)

clean:
	@cargo clean
	@$(RM) $(INSTALL_DIR)/tap
