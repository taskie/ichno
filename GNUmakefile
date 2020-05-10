.PHONY: build

TREBLO_CLI := target/release/treblo-cli

build:
	cd crates/cli && cargo build --release

PREFIX := $(HOME)/.local

.PHONY: install

install: build
	cp $(TREBLO_CLI) $(PREFIX)/bin/treblo

.PHONY: fmt

fmt:
	cargo fmt

.PHONY: fix

fix:
	cargo fix --allow-staged
