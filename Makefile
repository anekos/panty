

test:
	cargo build --verbose
	./scripts/test_run

build:
	cargo build


release:
	cargo build --release