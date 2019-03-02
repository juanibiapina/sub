fixture() {
  cp -r "${SUB_ROOT}/integration/fixtures/project" "$SUB_TEST_DIR"
  export PATH=${SUB_TEST_DIR}/project/bin:$PATH
}
