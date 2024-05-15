#!/usr/bin/env bats

load test_helper

@test "help: takes short flag" {
  fixture "commands"

  run main -h

  assert_success
  assert_output "$(main --help)"
}

@test "help: displays help for top level command" {
  fixture "commands"

  run main --help

  assert_success
  assert_output "Usage: main [OPTIONS] [commands_with_args]...

Top level command summary

Description of the top level command.

Extended documentation.

Available subcommands:
    a.sh       A sh script
    b          
    c.other    
    nested     "
}

@test "help: displays usage for a non documented command" {
  fixture "project"

  run main --help no-doc

  assert_success
  assert_output "Usage: main no-doc [args]..."
}

@test "help: displays help for a subcommand" {
  fixture "project"

  run main --help with-help

  assert_success
  assert_output "Usage: main with-help

Command with complete help

This is a complete test script with documentation.

The help section can span multiple lines."
}

@test "help: displays summary for subcommand if help is not available" {
  fixture "project"

  run main --help only-summary

  assert_success
  assert_output "Usage: main only-summary [args]...

Return with error 4"
}

@test "help: fails gracefully when requested command doesn't exist" {
  fixture "project"

  run main --help not-found

  assert_failure
  assert_output "main: no such sub command 'not-found'"
}

@test "help: displays help for a directory command" {
  fixture "project"

  run main --help directory

  assert_success
  assert_output "Usage: main directory [<subcommands>] [<args>]

A directory subcommand

Documentation for this group.

Extended documentation.

Available subcommands:
    double       Run a double nested command
    with-help    Help 2"
}

@test "help: displays help for a nested subcommand" {
  fixture "project"

  run main --help directory with-help

  assert_success
  assert_output "Usage: main directory with-help [args]...

Help 2

This is a complete test script with documentation.

The help section can span multiple lines."
}

@test "help: displays help for a double nested directory command" {
  fixture "project"

  run main --help directory double

  assert_success
  assert_output "Usage: main directory double [<subcommands>] [<args>]

Run a double nested command

Documentation for this double nested group.

Extended documentation.

Available subcommands:
    with-help    Help 3"
}

@test "help: displays help for a double nested sub command" {
  fixture "project"

  run main --help directory double with-help

  assert_success
  assert_output "Usage: main directory double with-help [args]...

Help 3

This is a complete test script with documentation.

The help section can span multiple lines."
}
