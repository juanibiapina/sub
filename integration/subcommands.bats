#!/usr/bin/env bats

load test_helper

@test "subcommands: lists files as subcommands" {
  fixture

  run main

  assert_success
  assert_line "SUBCOMMANDS:"
  assert_line --partial "echo"
}

@test "subcommands: invokes a subcommand without arguments" {
  fixture

  run main echo

  assert_success
  assert_output ""
}

@test "subcommands: invokes a subcommand with arguments" {
  fixture

  run main echo arg1 arg2

  assert_success
  assert_output "arg1 arg2"
}

@test "subcommands: accepts dashes in arguments to subcommands" {
  fixture

  run main echo -a -b

  assert_success
  assert_output "-a -b"
}

@test "subcommands: returns the subcommand exit code" {
  fixture

  run main error

  assert_failure 4
}
