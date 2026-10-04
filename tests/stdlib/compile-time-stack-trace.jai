// context.stack_trace is maintained during #run too.
#import "Basic";

depth_here :: () -> int {
    node := context.stack_trace;
    count := 0;
    while node { count += 1; node = node.next; }
    return count;
}

outer :: () -> int { return inner(); }
inner :: () -> int { return depth_here(); }

DEPTH :: #run outer();

main :: () {
    if DEPTH < 3 { print("FAIL: depth %\n", DEPTH); exit(1); }
    print("ok\n");
}
