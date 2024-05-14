#!/usr/bin/env bats

load test_helper

@test "completions: without arguments, lists commands" {
  fixture "completions"

  run main completions

  assert_success
  assert_output "$(main --commands)"
}

@test "completions: fails gracefully when command is not found" {
  fixture "completions"

  run main completions not-found

  assert_failure
  assert_output ""
}

@test "completions: invokes command completions" {
  fixture "completions"

  run main completions with-completions

  assert_success
  assert_output "comp1
comp2"
}

@test "completions: lists nothing if command provides no completions" {
  fixture "completions"

  run main completions no-completions

  assert_success
  assert_output ""
}

@test "completions: displays for directory commands" {
  fixture "completions"

  run main completions directory

  assert_success
  assert_output "$(main --commands directory)"
}

@test "completions: displays double nested directory commands" {
  fixture "completions"

  run main completions directory double

  assert_success
  assert_output "$(main --commands directory double)"
}

@test "completions: displays double nested subcommands" {
  fixture "completions"

  run main completions directory double with-completions

  assert_success
  assert_output "comp11
comp21"
}
