#!/usr/bin/env bats

load test_helper

@test "usage: when command has no Usage docstring prints default usage" {
  fixture "project"

  run main help no-doc

  assert_success
  assert_output "Usage: main no-doc"
}

@test "usage: when command has valid usage docstring, print it" {
  fixture "project"

  run main help valid-usage

  assert_success
  assert_output "Usage: main valid-usage"
}

@test "usage: when command has invalid usage docstring, error with message" {
  fixture "project"

  run main help invalid-usage

  assert_failure
  assert_output "main: invalid usage string
  found \"e\" but expected end of input"
}
