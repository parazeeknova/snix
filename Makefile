.PHONY: build test run clean release install

build:
	cargo build

test:
	cargo test

run:
	cargo run

clean:
	cargo clean

release:
	cargo build --release

install: release
	cp target/release/snix ~/.local/bin/