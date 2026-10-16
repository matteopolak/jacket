#import "Basic";
G :: #import "developer-tools-wrappers-generated";

main :: () {
    pair := G.OriginalWrapperPair.{ 7, 9 };

    assert(G.pair_sum(pair) == 16 && G.pair_sum(*pair) == 16);
    assert(G.format_message("value %", 42) == 675);
    assert(G.format_message("a\0b") == 195);
    print("ok\n");
}
