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
