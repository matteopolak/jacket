// test: should_fail("invalid workspace 999")
C :: #import "Compiler";

#run {
    options := C.get_build_options(999);
}

main :: () {}
