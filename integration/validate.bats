#!/usr/bin/env bats
load test_helper

PROJECT_DIR="$SUB_TEST_DIR/v1"

@test "sub: validates all subcommands in the project directory" {
  fixture "v1"

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- --validate

  assert_failure
  assert_output "$PROJECT_DIR/libexec/invalid-usage: invalid usage string
  found end of input but expected \"{\""
}
