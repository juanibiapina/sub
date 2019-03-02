#!/usr/bin/env bats

load test_helper

@test "name: replaces binary name in help text" {
  run $SUB_BIN --name thename --root . --

  assert_success
  assert_line --partial "thename"
  refute_line --partial "sub "
}
