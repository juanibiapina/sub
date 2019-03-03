#!/usr/bin/env bats

load test_helper

@test "help: without arguments, displays help" {
  fixture

  run main help

  assert_success
  assert_line --partial "main"
  assert_line "USAGE:"
  assert_line "SUBCOMMANDS:"
  assert_line --partial "echo"
}

@test "help: includes summaries of commands" {
  fixture

  run main help

  assert_success
  assert_line --partial "Echoes its arguments"
  assert_line --partial "Returns with error"
}

@test "help: displays help for a subcommand" {
  fixture

  run main help echo

  assert_success
  assert_output "This is a complete test script with documentation.

The help section can span multiple lines."
}

@test "help: displays summary for subcommand if help is not available" {
  fixture

  run main help error

  assert_success
  assert_output "Returns with error"
}

@test "help: fails gracefully with requested command doesn't exist" {
  fixture

  run main help not-found

  assert_success
  assert_output "main: no such sub command 'not-found'"
}
