#import "Basic";
G :: #import "developer-tools-linkage-generated";

main :: () {
    assert(G.value == 17 && G.read() == 17);
    G.value = 81;
    assert(G.read() == 81 && G.original_same() == 82);
    print("ok\n");
}
