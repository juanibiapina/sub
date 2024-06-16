.PHONY: integration
integration: build test
	bats integration

.PHONY: build
build:
	cargo build

.PHONY: test
test:
	cargo test

.PHONY: install
install:
	cargo install --path . --force

.PHONY: record
record:
	asciinema rec --command 'doitlive play --commentecho --quiet --shell bash assets/recording.sh' --overwrite assets/alias.cast
