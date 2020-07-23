.PHONY: build

build:
	$(MAKE) -C ichno build

PREFIX := $(HOME)/.local

.PHONY: install

install: build
	$(MAKE) -C ichno install

.PHONY: fmt

fmt:
	cargo fmt

.PHONY: fix

fix:
	cargo fix --allow-staged
