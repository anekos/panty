
build:
	cargo build

doc:
	cargo doc --no-deps

release:
	cargo build --release

test:
	cargo build --verbose
	./scripts/test_run

publish:
	cargo package
	cargo publish
