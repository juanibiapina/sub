#!/usr/bin/env bats

load test_helper

@test "usage: conflicts with help" {
  fixture "project"

  run main --usage --help

  assert_failure
  assert_output "error: the argument '--usage' cannot be used with '--help'

Usage: main --usage [commands_with_args]..."
}

@test "usage: when command has no Usage docstring prints default usage" {
  fixture "project"

  run main --usage no-doc

  assert_success
  assert_output "Usage: main no-doc [args]..."
}

@test "usage: when command has no Usage docstring, accepts any arguments" {
  fixture "project"

  run main no-doc arg1 arg2 -a --long other

  assert_success
  assert_output "arg1 arg2 -a --long other"
}

@test "usage: when command has valid usage docstring, print it" {
  fixture "project"

  run main --usage valid-usage

  assert_success
  assert_output "Usage: main valid-usage [OPTIONS] <positional> [args]..."
}

@test "usage: when command has invalid usage docstring, error with message" {
  fixture "project"

  run main --usage invalid-usage

  assert_failure
  assert_output "main: invalid usage string
  found end of input but expected \"{\""
}

@test "usage: invokes with valid arguments" {
  fixture "project"

  run main valid-usage --long pos -u --value=example extra1 extra2

  assert_success
  assert_output "--long pos -u --value=example extra1 extra2"
}

@test "usage: invoke fails when exclusive argument is combined with another argument" {
  fixture "project"

  run main valid-usage --exclusive -u

  assert_failure
  assert_output "error: the argument '--exclusive' cannot be used with one or more of the other specified arguments

Usage: main valid-usage [OPTIONS] <positional> [args]...

For more information, try '--help'."
}

@test "usage: invoke succeeds when exclusive argument is used alone" {
  fixture "project"

  run main valid-usage --exclusive

  assert_success
  assert_output "--exclusive"
}

@test "usage: invoke with invalid args, prints usage message" {
  fixture "project"

  run main valid-usage --invalid

  assert_failure
  assert_output "error: unexpected argument '--invalid' found

  tip: to pass '--invalid' as a value, use '-- --invalid'

Usage: main valid-usage [OPTIONS] <positional> [args]...

For more information, try '--help'."
}
