#!/usr/bin/env bats

load test_helper

@test "help: without arguments, displays help for top level command" {
  fixture "commands"

  run main help

  assert_success
  assert_output "Usage: main [<subcommands>] [<args>]

Top level command summary

Description of the top level command.

Extended documentation.

Available subcommands:
    a.sh        
    b           
    c.other     
    commands    List available commands
    help        Display help for a sub command
    nested      

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

  run main help with-help

  assert_success
  assert_output "Usage: main with-help

Command with complete help

This is a complete test script with documentation.

The help section can span multiple lines."
}

@test "help: displays summary for subcommand if help is not available" {
  fixture "project"

  run main help only-summary

  assert_success
  assert_output "Usage: main only-summary

Return with error 4"
}

@test "help: fails gracefully when requested command doesn't exist" {
  fixture "project"

  run main help not-found

  assert_failure
  assert_output "main: no such sub command 'not-found'"
}

@test "help: displays help for a directory command" {
  fixture "project"

  run main help directory

  assert_success
  assert_output "Usage: main directory [<subcommands>] [<args>]

A directory subcommand

Documentation for this group.

Extended documentation.

Available subcommands:
    double       Run a double nested command
    with-help    Help 2

Use 'main help directory <command>' for information on a specific command."
}

@test "help: displays help for a nested subcommand" {
  fixture "project"

  run main help directory with-help

  assert_success
  assert_output "Usage: main directory with-help

Help 2

This is a complete test script with documentation.

The help section can span multiple lines."
}

@test "help: displays help for a double nested directory command" {
  fixture "project"

  run main help directory double

  assert_success
  assert_output "Usage: main directory double [<subcommands>] [<args>]

Run a double nested command

Documentation for this double nested group.

Extended documentation.

Available subcommands:
    with-help    Help 3

Use 'main help directory double <command>' for information on a specific command."
}

@test "help: displays help for a double nested sub command" {
  fixture "project"

  run main help directory double with-help

  assert_success
  assert_output "Usage: main directory double with-help

Help 3

This is a complete test script with documentation.

The help section can span multiple lines."
}
