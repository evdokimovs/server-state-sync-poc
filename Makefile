fmt:
	cargo +nightly fmt

lint:
	cargo +nightly clippy

check:
	cargo check

build:
	cargo build

run:
	cargo run

.PHONY: check build fmt lint run