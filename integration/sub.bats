#!/usr/bin/env bats
load test_helper

PROJECT_DIR="$SUB_TEST_DIR/project"

@test "sub: when libexec is not a directory, exit with error" {
  run $SUB_BIN --name main --absolute "$PROJECT_DIR"

  assert_failure
  assert_output "main: libexec directory not found in root"
}

@test "sub: reject --bin and --absolute given together" {
  fixture "project"

  run $SUB_BIN --name main --bin "$PROJECT_DIR" --absolute "$PROJECT_DIR"
  assert_failure
}

@test "sub: reject --absolute and --relative given together" {
   fixture "project"

   run $SUB_BIN --name main --absolute "$PROJECT_DIR" --relative ".."
   assert_failure
}

@test "sub: reject relative paths given as --absolute" {
    fixture "project"

    run $SUB_BIN --name main --absolute "./foo"
    assert_failure
}

@test "sub: --infer-long-arguments flag" {
  fixture "project"

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" --infer-long-arguments -- valid-usage --lo pos

  assert_success
  assert_output "--lo pos"
}

@test "sub: sets an env variable with the project root" {
  fixture "project"

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- env _MAIN_ROOT

  assert_success
  assert_output "$PROJECT_DIR"
}

@test "sub: sets an env variable with argument key value pairs" {
  fixture "project"

  run $SUB_BIN --name main --absolute "$PROJECT_DIR" -- env-args --long --value=thing pos ex1 ex2 --more

  assert_success
  assert_output 'name "pos" u "false" long "true" value "thing" args "ex1 ex2 --more"'
}
