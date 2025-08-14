.PHONY: generate
generate:
	cargo run -p turnkey_codegen
	cargo fmt --
	@$(MAKE) strip-doc file=client/src/generated/google.api.rs
	@$(MAKE) strip-doc file=client/src/generated/google.rpc.rs
	@$(MAKE) strip-doc file=client/src/generated/grpc.gateway.protoc_gen_openapiv2.options.rs

# This is necessary for some google-generated files. They have non-valid comments that rustdoc tries to parse.
strip-doc:
	@echo "Stripping /// doc comments from $(file)..."
	@tmpfile=$$(mktemp) && \
		awk '!/^[[:space:]]*\/{3}/' $(file) > $$tmpfile && \
		mv $$tmpfile $(file)

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
	cargo run -p turnkey_examples --bin whoami
	cargo run -p turnkey_examples --bin sub_organization
	cargo run -p turnkey_examples --bin sub_organization_secp256k1
	cargo run -p turnkey_examples --bin wallet
	cargo run -p turnkey_examples --bin proofs
