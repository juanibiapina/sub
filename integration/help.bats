#!/usr/bin/env bats

load test_helper

@test "help: without arguments, displays help" {
  fixture

  run main help

  assert_success
  assert_output "Usage: main <command> [args]

Available commands:
    commands    List available commands
    echo        Echo arguments
    env         Print the value of an environment variable
    error       Return with error 4
    help        Display help for a sub command
    nested      Run a nested command

Use 'main help <command>' for information on a specific command."
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
  assert_output "Return with error 4"
}

@test "help: fails gracefully with requested command doesn't exist" {
  fixture

  run main help not-found

  assert_success
  assert_output "main: no such sub command 'not-found'"
}
