.PHONY: integration
integration: build test
	./integration/vendor/bats/bin/bats integration

# Find LLVM tools - check common locations (can override with LLVM_BIN=/path/to/bin)
LLVM_BIN ?= $(shell \
	for path in \
		"$$(which llvm-profdata 2>/dev/null | xargs dirname 2>/dev/null)" \
		"/opt/homebrew/opt/llvm/bin" \
		"/usr/local/opt/llvm/bin" \
		"/usr/lib/llvm-18/bin" \
		"/usr/lib/llvm-17/bin" \
		"/usr/lib/llvm-16/bin" \
		"/usr/lib/llvm-15/bin" \
		"/usr/bin"; do \
		if [ -x "$$path/llvm-profdata" ] && [ -x "$$path/llvm-cov" ]; then \
			echo "$$path"; \
			break; \
		fi; \
	done)

.PHONY: coverage
coverage: test
ifndef LLVM_BIN
	$(error LLVM tools not found. Install LLVM or set LLVM_BIN manually)
endif
	RUSTFLAGS="-C instrument-coverage" cargo build
	rm -f *.profraw
	-LLVM_PROFILE_FILE="sub-%p-%m.profraw" ./integration/vendor/bats/bin/bats integration/
	$(LLVM_BIN)/llvm-profdata merge -sparse *.profraw -o sub.profdata
	$(LLVM_BIN)/llvm-cov report ./target/debug/sub --instr-profile=sub.profdata --ignore-filename-regex='/.cargo/registry|rustc'
	rm -f *.profraw sub.profdata

.PHONY: build
build:
	cargo build

.PHONY: lint
lint:
	cargo clippy -- -D warnings

.PHONY: test
test:
	cargo test

.PHONY: install
install:
	cargo install --path . --force

.PHONY: record
record:
	asciinema rec --command 'doitlive play --commentecho --quiet --shell bash assets/recording.sh' --overwrite assets/alias.cast
