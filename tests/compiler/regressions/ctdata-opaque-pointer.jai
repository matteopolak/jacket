// rules: ctdata.5
// A `#run` result that points at memory of unknown size (a `*void` from `alloc`) stays the address
// it was: it is not an error and no data is captured.
#import "Basic";

raw_block :: () -> *void {
    return alloc(32);
}

BLOCK :: #run raw_block();

main :: () {
    assert(BLOCK != null);
}
