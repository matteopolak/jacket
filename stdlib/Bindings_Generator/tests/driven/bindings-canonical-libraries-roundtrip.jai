#import "Basic";
G :: #import "developer-tools-libraries-generated";

main :: () {
    assert(G.primary_value() == 42);
    assert(G.static_value() == 17);
    assert(G.system_value() == 3);
    print("ok\n");
}
