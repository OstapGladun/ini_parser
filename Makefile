run:
	@cargo run -- $(ARGS)

credits:
	@cargo run -- credits

help:
	@cargo run -- --help

fmt:
	@echo "Formatting code..."
	@cargo fmt

lint:
	@echo "Linting code..."
	@cargo clippy -- -D warnings

test:
	@echo "Running tests..."
	@cargo test

check: fmt lint test

doc:
	@cargo doc --open

publish: check
	@echo "Running final checks before publishing..."
	@cargo publish

clean:
	@cargo clean

.PHONY: run credits help fmt lint test check doc publish clean