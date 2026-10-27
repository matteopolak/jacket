// test: compile_fail
#import "Basic";
H :: #import "Hash";

main :: () {
    assert(H.HASH_INIT == 5381);
}
