// rules: ctdata.5
// A #run result that points at memory of unknown size (a `*void` from alloc) cannot be
// turned into a constant.
#import "Basic";

raw_block :: () -> *void {
    return alloc(32);
}

BLOCK :: #run raw_block();

main :: () {
    print("%\n", BLOCK);
}
