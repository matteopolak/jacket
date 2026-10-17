#import "Basic";
G :: #import "developer-tools-signature-generated";

main :: () {
    assert(G.call() == 42 && G.call(8) == 9);
    print("ok\n");
}
