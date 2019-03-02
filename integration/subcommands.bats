#!/usr/bin/env bats

load test_helper

@test "subcommands: lists files as subcommands" {
  fixture

  run main

  assert_failure
  assert_line "SUBCOMMANDS:"
  assert_line --partial "command1"
}
