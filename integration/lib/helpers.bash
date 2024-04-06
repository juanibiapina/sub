fixture() {
  name="$1"
  cp -r "${SUB_ROOT}/integration/fixtures/$name" "$SUB_TEST_DIR"
  export PATH="${SUB_TEST_DIR}/$name/bin:$PATH"
}
