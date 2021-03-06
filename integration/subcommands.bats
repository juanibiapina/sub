#!/usr/bin/env bats

load test_helper

@test "subcommands: without a subcommand, displays help" {
  fixture

  run main

  assert_success
  assert_output "$(main help)"
}

@test "subcommands: ignores hidden files" {
  fixture

  run main

  assert_success
  refute_line --partial ".hidden"
}

@test "subcommands: ignores non executable files" {
  fixture

  run main

  assert_success
  refute_line --partial "non-exec"
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

@test "subcommands: handles non existent commands gracefully" {
  fixture

  run main not-found

  assert_failure
  assert_output "main: no such sub command 'not-found'"
}
