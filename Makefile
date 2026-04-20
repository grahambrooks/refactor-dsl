CARGO ?= cargo
export RUSTFLAGS := -Dwarnings
export RUSTDOCFLAGS := -Dwarnings

.PHONY: all pre-commit ci check fmt fmt-fix clippy test doc clean help

all: pre-commit

pre-commit: fmt clippy check test doc

ci: pre-commit

check:
	$(CARGO) check --all-features

fmt:
	$(CARGO) fmt --all -- --check

fmt-fix:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --all-targets --all-features

test:
	$(CARGO) test --all-features

doc:
	$(CARGO) doc --no-deps --all-features

clean:
	$(CARGO) clean

help:
	@echo "Targets:"
	@echo "  pre-commit  Run all checks performed in CI (fmt, clippy, check, test, doc)"
	@echo "  ci          Alias for pre-commit"
	@echo "  check       cargo check --all-features"
	@echo "  fmt         cargo fmt --all -- --check"
	@echo "  fmt-fix     cargo fmt --all (apply formatting)"
	@echo "  clippy      cargo clippy --all-targets --all-features"
	@echo "  test        cargo test --all-features"
	@echo "  doc         cargo doc --no-deps --all-features"
	@echo "  clean       cargo clean"
