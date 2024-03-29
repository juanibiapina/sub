#!/usr/bin/env bats
load test_helper

PROJECT_DIR="$SUB_TEST_DIR/project"

@test "sub: reject --bin and --absolute given together" {
  fixture

  run $SUB_BIN --name main --bin "$PROJECT_DIR" --absolute "$PROJECT_DIR"
  assert_failure
}

@test "sub: reject --absolute and --relative given together" {
   fixture

   run $SUB_BIN --name main --absolute "$PROJECT_DIR" --relative ".."
   assert_failure
}

@test "sub: reject relative paths given as --absolute" {
    fixture

    run $SUB_BIN --name main --absolute "./foo"
    assert_failure
}

@test "sub: lists commands alphabetically" {
  fixture

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- commands

  assert_success
  assert_output "commands
echo
env
error
help
nested
no-doc"
}

@test "sub: sets an env variable with the project root" {
  fixture

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- env _MAIN_ROOT

  assert_success
  assert_output "$PROJECT_DIR"
}
