#!/usr/bin/env bats
load test_helper

PROJECT_DIR="$SUB_TEST_DIR/project"

@test "args: a required positional value cannot execute code" {
  fixture "project"

  payload='x"$(touch '"$SUB_TEST_DIR"'/PWNED)"y'

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- args-eval "$payload"

  assert_success
  assert_line "name=$payload"
  refute [ -e "$SUB_TEST_DIR/PWNED" ]
}

@test "args: an optional positional value cannot execute code" {
  fixture "project"

  payload='x"$(touch '"$SUB_TEST_DIR"'/PWNED)"y'

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- args-eval safe "$payload"

  assert_success
  assert_line "opt=$payload"
  refute [ -e "$SUB_TEST_DIR/PWNED" ]
}

@test "args: a flag value cannot execute code" {
  fixture "project"

  payload='x`touch '"$SUB_TEST_DIR"'/PWNED`y'

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- args-eval --title="$payload" safe

  assert_success
  assert_line "title=$payload"
  refute [ -e "$SUB_TEST_DIR/PWNED" ]
}

@test "args: a rest value cannot execute code" {
  fixture "project"

  payload='x"$(touch '"$SUB_TEST_DIR"'/PWNED)"y'

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- args-eval safe opt "$payload"

  assert_success
  assert_line "rest=$payload"
  refute [ -e "$SUB_TEST_DIR/PWNED" ]
}

@test "args: an unbalanced quote in a value cannot execute code" {
  fixture "project"

  payload='x" $(touch '"$SUB_TEST_DIR"'/PWNED) "'

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- args-eval "$payload"

  assert_success
  assert_line "name=$payload"
  refute [ -e "$SUB_TEST_DIR/PWNED" ]
}

@test "args: a value cannot fabricate other argument keys" {
  fixture "project"

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- args-eval 'safe" title "spoofed'

  assert_success
  assert_line 'name=safe" title "spoofed'
  assert_line "title="
}

@test "args: a value containing a single quote round trips" {
  fixture "project"

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- args-eval "it's fine"

  assert_success
  assert_line "name=it's fine"
}

@test "args: a value cannot execute code in a command without a usage docstring" {
  fixture "project"

  payload='x"$(touch '"$SUB_TEST_DIR"'/PWNED)"y'

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- no-doc-args "$payload" second

  assert_success
  assert_line "args=$payload second"
  refute [ -e "$SUB_TEST_DIR/PWNED" ]
}
