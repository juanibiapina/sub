#!/usr/bin/env bats
load test_helper

@test "edit: takes short flag -e" {
  fixture "project"

  # Use echo as a mock editor that just prints the file path
  export VISUAL="echo"

  run main -e echo

  assert_success
  assert_output --partial "libexec/echo"
}

@test "edit: takes long flag --edit" {
  fixture "project"

  export VISUAL="echo"

  run main --edit echo

  assert_success
  assert_output --partial "libexec/echo"
}

@test "edit: uses VISUAL env var first" {
  fixture "project"

  export VISUAL="echo VISUAL:"
  export EDITOR="echo EDITOR:"

  run main --edit echo

  assert_success
  assert_output --partial "VISUAL:"
  refute_output --partial "EDITOR:"
}

@test "edit: falls back to EDITOR when VISUAL is unset" {
  fixture "project"

  unset VISUAL
  export EDITOR="echo EDITOR:"

  run main --edit echo

  assert_success
  assert_output --partial "EDITOR:"
}

@test "edit: works with nested subcommands" {
  fixture "project"

  export VISUAL="echo"

  run main --edit directory with-help

  assert_success
  assert_output --partial "libexec/directory/with-help"
}

@test "edit: fails gracefully when command doesn't exist" {
  fixture "project"

  export VISUAL="echo"

  run main --edit not-found

  assert_failure
  assert_output "main: no such sub command 'not-found'"
}

@test "edit: fails gracefully when no editor is configured" {
  fixture "project"

  unset VISUAL
  unset EDITOR

  run main --edit echo

  assert_failure
  assert_output --partial "no editor configured"
}

@test "edit: fails gracefully when trying to edit a directory" {
  fixture "project"

  export VISUAL="echo"

  run main --edit directory

  assert_failure
  assert_output --partial "cannot edit a directory"
}
