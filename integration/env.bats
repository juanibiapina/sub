#!/usr/bin/env bats

load test_helper

@test "env: sets an env variable with the project root" {
  fixture

  run main env _MAIN_ROOT

  assert_success
  assert_output "/private${SUB_TEST_DIR}/project"
}
