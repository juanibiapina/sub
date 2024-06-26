#!/usr/bin/env bats

load test_helper

@test "env: sets an env variable with the project root" {
  fixture "project"

  run main env _MAIN_ROOT

  if [ $(uname) = "Darwin" ]; then
    prefix="/private"
  fi

  assert_success
  assert_output "$prefix${SUB_TEST_DIR}/project"
}

@test "env: sets an env variable for a XDG cache directory" {
  fixture "project"

  run main env _MAIN_CACHE

  assert_success
  assert_output "$HOME/.cache/main/cache"
}
