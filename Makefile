.PHONY: integration
integration: build test
	bats integration

.PHONY: build
build:
	go build -o sub main.go

.PHONY: test
test:
	go test ./...

.PHONY: install
install:
	go install .

.PHONY: record
record:
	asciinema rec --command 'doitlive play --commentecho --quiet --shell bash assets/recording.sh' --overwrite assets/alias.cast
