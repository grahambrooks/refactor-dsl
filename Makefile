# Developer targets. `make check` is the pre-push gate and runs exactly what the
# CI `check` job runs (.github/workflows/ci.yml); `make hooks` wires it into git.
CARGO ?= cargo
.DEFAULT_GOAL := help

.PHONY: help check pre-commit ci all fmt fmt-check fmt-fix lint clippy test doc hooks clean

help: ## List targets
	@grep -hE '^[a-zA-Z0-9_-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{ printf "  %-16s %s\n", $$1, $$2 }'

check: fmt-check lint test ## The pre-push gate: the CI check job (fmt, clippy, test)

pre-commit: check ## Alias for check
ci: check ## Alias for check
all: check

fmt: ## Format the code
	$(CARGO) fmt --all

fmt-fix: fmt

fmt-check: ## Fail if the code is not formatted
	$(CARGO) fmt --all --check

lint: ## Clippy with warnings denied, as CI runs it
	$(CARGO) clippy --all-targets --all-features --locked -- -D warnings

clippy: lint

test: ## Tests, as CI runs them
	$(CARGO) test --all-features --locked

doc: ## Build the API docs with warnings denied (CI's docs job)
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --no-deps --all-features --locked

hooks: ## Install the prek git hooks (pre-commit and pre-push)
	prek install

clean: ## Remove build artifacts
	$(CARGO) clean

include release.mk
