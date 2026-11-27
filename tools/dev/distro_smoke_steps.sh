#!/bin/sh
# Runs inside one distro container for tools/dev/distro_smoke.sh: installs the packages and the
# release, then the commands a new user runs first. Each step prints `smoke: STEP`; a failure
# prints `smoke: FAILED STEP` and stops.
set -u

step() {
    name=$1
    shift
    echo "smoke: $name"
    if ! "$@"; then
        echo "smoke: FAILED $name"
        exit 1
    fi
}

step "install packages" sh -c "$SMOKE_SETUP"
step "install jacket" sh /smoke/install.sh
PATH="$HOME/.local/bin:$PATH"
export PATH
step "jacket --version" jacket --version

mkdir -p /work && cd /work || exit 1
cat > hello.jai <<'EOF'
#import "Basic";
main :: () { print("hello from jacket\n"); }
EOF
expect_hello() {
    out=$("$@") || return 1
    echo "$out"
    [ "$out" = "hello from jacket" ]
}
step "jaic -interp" expect_hello jaic hello.jai -interp -quiet
step "jacket run" expect_hello jacket run hello.jai -quiet
step "jacket build" jacket build hello.jai
step "the built program" expect_hello ./hello

step "jacket new" jacket new app
cd app || exit 1
step "jacket run (new program)" jacket run
cd /work || exit 1

step "jacket new --lib" jacket new words --lib
cd words || exit 1
cat > module.jai <<'EOF'
#import "Basic";

// Doubles a number.
//
// ```jai
// assert(twice(2) == 4);
// ```
twice :: (n: int) -> int {
    return n * 2;
}
EOF
mkdir -p tests
cat > tests/twice.jai <<'EOF'
#import "Basic";
#import "Extensions/Testing";
#import "words";

doubles :: (t: *Test) {
    expect_equal(twice(3), 6);
} @test
EOF
step "jacket test" jacket test
step "jacket fmt --check" jacket fmt --check
step "jacket lint" jacket lint
step "jacket toolchain" jacket toolchain
echo "smoke: done"
