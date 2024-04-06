#!/usr/bin/env bats

load test_helper

@test "commands: lists commands alphabetically" {
  fixture "project"

  run main commands

  assert_success
  assert_output "commands
echo
env
error
help
nested
no-doc"
}

@test "commands: lists nested commands" {
  fixture "project"

  run main commands nested

  assert_success
  assert_output "double
echo"
}

@test "commands: lists nested subcommands" {
  fixture "project"

  run main commands nested double

  assert_success
  assert_output "echo"
}
