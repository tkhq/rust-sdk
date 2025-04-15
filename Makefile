.PHONY: generate
generate:
	cargo run -p tkhq_codegen

.PHONY: check-generate
check-generate:
	cargo run -p tkhq_codegen
	git diff --exit-code client/src/generated/client.rs

.PHONY: lint
lint:
	# Check formatting
	cargo fmt -- --check
	# Run Clippy
	cargo clippy -- -D warnings

.PHONY: build
build:
	cargo build

.PHONY: test
test: build
	cargo test
