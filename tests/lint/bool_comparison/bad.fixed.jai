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
    print("%\n", waiting);
}
