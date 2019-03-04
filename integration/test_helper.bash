load vendor/bats-support/load
load vendor/bats-assert/load

export SUB_ROOT="${BATS_TEST_DIRNAME}/.."

export SUB_TEST_DIR="${BATS_TMPDIR}/sub"

if [ -z $SUB_BIN ]; then
  export SUB_BIN=$SUB_ROOT/target/debug/sub
fi

mkdir -p $SUB_TEST_DIR

teardown() {
  rm -rf "$SUB_TEST_DIR"
}

load lib/helpers
