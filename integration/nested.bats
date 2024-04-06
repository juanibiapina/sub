#!/usr/bin/env bats

load test_helper

@test "nested: without a subcommand, displays help" {
  fixture "project"

  run main nested

  assert_success
  assert_output "$(main help nested)"
}

@test "nested: with a non existent subcommand, displays error message" {
  fixture "project"

  run main nested not-found

  assert_failure
  assert_output "main: no such sub command 'not-found'"
}

@test "nested: with a subcommand, runs it" {
  fixture "project"

  run main nested echo arg1 arg2

  assert_success
  assert_output "arg1 arg2"
}

@test "nested: with a nested subcommand, displays help" {
  fixture "project"

  run main nested double

  assert_success
  assert_output "$(main help nested double)"
}
