#!/usr/bin/env bats

load test_helper

@test "subcommands: without a subcommand, displays help" {
  fixture "project"

  run main

  assert_success
  assert_output "$(main --help)"
}

@test "subcommands: ignores hidden files" {
  fixture "project"

  run main

  assert_success
  refute_line --partial ".hidden"
}

@test "subcommands: ignores non executable files" {
  fixture "project"

  run main

  assert_success
  refute_line --partial "non-exec"
}

@test "subcommands: invokes a subcommand without arguments" {
  fixture "project"

  run main echo

  assert_success
  assert_output ""
}

@test "subcommands: invokes a subcommand with arguments" {
  fixture "project"

  run main echo arg1 arg2

  assert_success
  assert_output "arg1 arg2"
}

@test "subcommands: accepts dashes in arguments to subcommands" {
  fixture "project"

  run main echo -a --long

  assert_success
  assert_output "-a --long"
}

@test "subcommands: returns the subcommand exit code" {
  fixture "project"

  run main error

  assert_failure 4
}

@test "subcommands: handles non existent commands gracefully" {
  fixture "project"

  run main not-found

  assert_failure
  assert_output "main: no such sub command 'not-found'"
}
