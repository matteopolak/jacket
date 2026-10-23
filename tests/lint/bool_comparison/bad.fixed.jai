#import "Basic";

is_ready :: () -> bool {
    return true;
}

main :: () {
    ready := is_ready();
    if ready print("ready\n");
    if ready print("still ready\n");
    if !ready print("not ready\n");

    waiting := !is_ready();
    a, b := 1, 2;

    if !(a < b) print("not less\n");
    if (a < b) print("less\n");

    print("%\n", waiting);
}
