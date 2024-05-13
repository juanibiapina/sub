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
  assert_output "Usage: main no-doc"
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
  assert_output "Usage: main valid-usage"
}

@test "usage: when command has invalid usage docstring, error with message" {
  fixture "project"

  run main --usage invalid-usage

  assert_failure
  assert_output "main: invalid usage string
  found \"e\" but expected end of input"
}
