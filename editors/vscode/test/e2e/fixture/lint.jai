#import "Basic";

is_ready :: () -> bool {
    return true;
}

main :: () {
    ready := is_ready();
    if ready == true print("ready\n");
}
