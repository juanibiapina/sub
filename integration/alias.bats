#!/usr/bin/env bats

load test_helper

@test "alias: replaces binary name in help text" {
  run $SUB_BIN --alias thename --root .

  assert_failure
  assert_line --partial "thename"
  refute_line --partial "sub "
}
