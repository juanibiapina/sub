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
  assert_output "Echo arguments

This is a complete test script with documentation.

The help section can span multiple lines."
}

@test "help: displays summary for subcommand if help is not available" {
  fixture

  run main help error

  assert_success
  assert_output "Return with error 4"
}

@test "help: fails gracefully when requested command doesn't exist" {
  fixture

  run main help not-found

  assert_failure
  assert_output "main: no such sub command 'not-found'"
}

@test "help: displays help for a nested command" {
  fixture

  run main help nested

  assert_success
  assert_output "Run a nested command

Documentation for this group.

Extended documentation."
}

@test "help: displays help for a nested subcommand" {
  fixture

  run main help nested echo

  assert_success
  assert_output "Echo arguments 2

This is a complete test script with documentation.

The help section can span multiple lines."
}

@test "help: displays help for a double nested command" {
  fixture

  run main help nested double

  assert_success
  assert_output "Run a double nested command

Documentation for this double nested group.

Extended documentation."
}

@test "help: displays help for a double nested sub command" {
  fixture

  run main help nested double echo

  assert_success
  assert_output "Echo arguments 3

This is a complete test script with documentation.

The help section can span multiple lines."
}
