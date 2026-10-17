#import "Basic";
G :: #import "developer-tools-sdk-generated";

main :: () {
    value: G.Sized;
    value.x = 4;
    value.y = 9;
    assert(G.sized_sum(value = *value) == 13);
    print("ok\n");
}
