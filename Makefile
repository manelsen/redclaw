.PHONY: all build release clean help onboard

BINARY_NAME=redclaw

all: release

build:
	cargo build
	cp target/debug/$(BINARY_NAME) .

release:
	cargo build --release
	strip target/release/$(BINARY_NAME)
	cp target/release/$(BINARY_NAME) .
	@echo "\n✅ $(BINARY_NAME) is ready in the root directory!"
	@ls -lh $(BINARY_NAME)

onboard: release
	./$(BINARY_NAME) onboard

clean:
	cargo clean
	rm -f $(BINARY_NAME)

help:
	@echo "RedClaw 🦀 Makefile"
	@echo "Usage:"
	@echo "  make          - Build optimized release binary and copy to root"
	@echo "  make onboard  - Build and start the configuration wizard"
	@echo "  make build    - Build debug binary and copy to root"
	@echo "  make clean    - Remove build artifacts and binary"
