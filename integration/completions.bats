#!/usr/bin/env bats

load test_helper

@test "completions: without arguments, lists commands" {
  fixture

  run main completions

  assert_success
  assert_output "$(main commands)"
}

@test "completions: fails gracefully when command is not found" {
  fixture

  run main completions not-found

  assert_failure
  assert_output ""
}

@test "completions: invokes command completions" {
  fixture

  run main completions echo

  assert_success
  assert_output "comp1
comp2"
}

@test "completions: lists nothing if command provides no completions" {
  fixture

  run main completions error

  assert_success
  assert_output ""
}
