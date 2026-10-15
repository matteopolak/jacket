#import "Basic";
G :: #import "developer-tools-macro-merge-generated";

main :: () {
    assert(G.Existing.GROUP_ONE == 1 && G.Existing.EXTRA_MEMBER == 9);
    assert(G.merged_value(.GROUP_ONE) == 2);
    assert(G.merged_value(.EXTRA_MEMBER) == 10);
    print("ok\n");
}
