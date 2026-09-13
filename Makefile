install:
	cargo install --path .
	stfu install-extension

build:
	cargo build --release

.PHONY: install build
