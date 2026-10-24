install:
	cargo install --path .

build:
	cargo build --release

test:
	cargo test

.PHONY: install build test
