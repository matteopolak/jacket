#import "Basic";
Owner :: #import "context-import-late-owner";

#run {
    assert(size_of(#Context) > 0);
}

read :: () -> *void {
    return Owner.read_original();
}

main :: () {
    assert(read() == null);
    print("ok\n");
}
