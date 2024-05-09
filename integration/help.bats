#!/usr/bin/env bats

load test_helper

@test "help: without arguments, displays help for top level command" {
  fixture "project"

  run main help

  assert_success
  assert_output "Usage: main [<subcommands>] [<args>]

Top level command summary

Description of the top level command.

Extended documentation.

Available subcommands:
    commands    List available commands
    echo        Echo arguments
    env         Print the value of an environment variable
    error       Return with error 4
    help        Display help for a sub command
    nested      Run a nested command
    no-doc      

Use 'main help <command>' for information on a specific command."
}

@test "help: displays usage for a non documented command" {
  fixture "project"

  run main help no-doc

  assert_success
  assert_output "Usage: main no-doc"
}

@test "help: displays help for a subcommand" {
  fixture "project"

  run main help echo

  assert_success
  assert_output "Usage: main echo

Echo arguments

This is a complete test script with documentation.

The help section can span multiple lines."
}

@test "help: displays summary for subcommand if help is not available" {
  fixture "project"

  run main help error

  assert_success
  assert_output "Usage: main error

Return with error 4"
}

@test "help: fails gracefully when requested command doesn't exist" {
  fixture "project"

  run main help not-found

  assert_failure
  assert_output "main: no such sub command 'not-found'"
}

@test "help: displays help for a nested command" {
  fixture "project"

  run main help nested

  assert_success
  assert_output "Usage: main nested [<subcommands>] [<args>]

Run a nested command

Documentation for this group.

Extended documentation.

Available subcommands:
    double    Run a double nested command
    echo      Echo arguments 2

Use 'main help nested <command>' for information on a specific command."
}

@test "help: displays help for a nested subcommand" {
  fixture "project"

  run main help nested echo

  assert_success
  assert_output "Usage: main nested echo

Echo arguments 2

This is a complete test script with documentation.

The help section can span multiple lines."
}

@test "help: displays help for a double nested command" {
  fixture "project"

  run main help nested double

  assert_success
  assert_output "Usage: main nested double [<subcommands>] [<args>]

Run a double nested command

Documentation for this double nested group.

Extended documentation.

Available subcommands:
    echo    Echo arguments 3

Use 'main help nested double <command>' for information on a specific command."
}

@test "help: displays help for a double nested sub command" {
  fixture "project"

  run main help nested double echo

  assert_success
  assert_output "Usage: main nested double echo

Echo arguments 3

This is a complete test script with documentation.

The help section can span multiple lines."
}
