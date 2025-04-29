.PHONY: generate
generate:
	cargo run -p tkhq_codegen
	cargo fmt --

.PHONY: check-generate
check-generate:
	make generate
	git diff --exit-code client/src/generated/client.rs

.PHONY: fmt
fmt:
	cargo fmt --

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

.PHONY: examples
examples: build
	cargo run -p tkhq_examples --bin whoami
	cargo run -p tkhq_examples --bin create_sub_organization
	cargo run -p tkhq_examples --bin wallet
