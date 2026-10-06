#import "Basic";

VERBOSE :: true;

is_ready :: () -> bool {
    return true;
}

main :: () {
    ready := is_ready();
    other := !ready;
    if ready print("ready\n");
    if !other print("not other\n");
    // Two variables: neither side is a literal.
    if ready == other print("same\n");
    // Constants are folded at compile time.
    #if VERBOSE == true {
        print("verbose\n");
    }
    count := 3;
    if count == 3 print("three\n");
}
