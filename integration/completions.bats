#!/usr/bin/env bats

load test_helper

@test "completions: without arguments, lists commands" {
  fixture "project"

  run main completions

  assert_success
  assert_output "$(main commands)"
}

@test "completions: fails gracefully when command is not found" {
  fixture "project"

  run main completions not-found

  assert_failure
  assert_output ""
}

@test "completions: invokes command completions" {
  fixture "project"

  run main completions echo

  assert_success
  assert_output "comp1
comp2"
}

@test "completions: lists nothing if command provides no completions" {
  fixture "project"

  run main completions error

  assert_success
  assert_output ""
}

@test "completions: displays nested commands" {
  fixture "project"

  run main completions nested

  assert_success
  assert_output "$(main commands nested)"
}

@test "completions: displays double nested commands" {
  fixture "project"

  run main completions nested double

  assert_success
  assert_output "$(main commands nested double)"
}

@test "completions: displays double nested subcommands" {
  fixture "project"

  run main completions nested double echo

  assert_success
  assert_output "compn1
compn2"
}
