.PHONY: build

build:
	cargo build --release

PREFIX := $(HOME)/.local

.PHONY: install

install: build
	$(MAKE) -C treblo_cli install
	$(MAKE) -C ichno_cli install
	$(MAKE) -C ichnome_cli install

.PHONY: fmt

fmt:
	cargo fmt --all

.PHONY: fix

fix:
	cargo fix --allow-staged

.PHONY: doc

doc:
	cargo doc --open
