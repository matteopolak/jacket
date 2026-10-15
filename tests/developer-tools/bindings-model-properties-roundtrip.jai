#import "Basic";
G :: #import "developer-tools-properties-generated";

main :: () {
    assert(G.properties_count() == 0);
    G.inspect_flags(true);
    G.inspect_flags(false);
    assert(G.properties_count() == 4);
    print("ok\n");
}
