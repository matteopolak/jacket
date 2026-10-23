#import "Basic";

is_ready :: () -> bool {
    return true;
}

main :: () {
    ready := is_ready();
    if ready == true print("ready\n");
    if ready != false print("still ready\n");
    if false == ready print("not ready\n");

    waiting := is_ready() == false;
    a, b := 1, 2;

    if (a < b) == false print("not less\n");
    if (a < b) == true print("less\n");

    print("%\n", waiting);
}
