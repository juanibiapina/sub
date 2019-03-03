#!/usr/bin/env bats

load test_helper

@test "commands: lists commands alphabetically" {
  fixture

  run main commands

  assert_success
  assert_output "commands
echo
env
error
help
nested"
}
