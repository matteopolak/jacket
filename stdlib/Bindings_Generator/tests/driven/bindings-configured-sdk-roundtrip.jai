#import "Basic";
G :: #import "developer-tools-configured-generated";

main :: () {
    assert(G.configured_sdk_sum(left = 17, right = 25) == 42);
    assert(G.configured_sdk_sum(left = -9, right = 4) == -5);
    print("ok\n");
}
