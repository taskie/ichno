BIN := target/release/treblo

.PHONY: build

build:
	cargo build --release

PREFIX := $(HOME)/.local

.PHONY: install

install: build
	install -m755 $(BIN) $(PREFIX)/bin/treblo

.PHONY: fmt

fmt:
	cargo fmt

.PHONY: fix

fix:
	cargo fix --allow-staged
