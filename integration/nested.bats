#!/usr/bin/env bats

load test_helper

@test "nested: without a subcommand, displays help" {
  fixture

  run main nested

  assert_success
  assert_output "$(main help nested)"
}
