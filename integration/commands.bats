#!/usr/bin/env bats

load test_helper

@test "commands: lists commands alphabetically" {
  fixture "commands"

  run main --commands

  assert_success
  assert_output "a.sh
b
c.other
invalid-usage
nested"
}

@test "commands: filter commands by extension" {
  fixture "commands"

  run main --commands --extension=sh

  assert_success
  assert_output "a.sh"
}

@test "commands: lists nested commands" {
  fixture "commands"

  run main --commands nested

  assert_success
  assert_output "d
double"
}

@test "commands: lists nested subcommands" {
  fixture "commands"

  run main --commands nested double

  assert_success
  assert_output "e"
}
