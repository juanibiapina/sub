#!/usr/bin/env bats

load test_helper

@test "nested: without a subcommand, displays help" {
  fixture "project"

  run main nested

  assert_success
  assert_output "Run a nested command

Usage: main nested [commands_with_args]...

Arguments:
  [commands_with_args]...  

Options:
  -h, --help  Print help

Documentation for this group.

Extended documentation.

Available subcommands:
    double    Run a double nested command
    echo      Echo arguments 2"
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
  assert_output "Run a double nested command

Usage: main nested double [commands_with_args]...

Arguments:
  [commands_with_args]...  

Options:
  -h, --help  Print help

Documentation for this double nested group.

Extended documentation.

Available subcommands:
    echo    Echo arguments 3"
}
