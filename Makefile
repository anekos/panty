

test:
	cargo build --verbose
	./scripts/test_run

build:
	cargo build

doc:
	cargo doc --no-deps

release:
	cargo build --release

publish:
	cargo package
	cargo publish
