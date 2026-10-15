#import "Basic";
G :: #import "developer-tools-enums-generated";

main :: () {
    assert(size_of(G.Status) == 4);
    assert(G.Status.Failed == G.Status.NATIVE_FAILED);
    assert(G.Status.Ready == G.Status.NATIVE_READY);
    assert(cast(s32) G.BodyOnly.Failed == -7);
    assert(cast(s32) G.BodyOnly.Ready == 5);
    assert(cast(s32) (G.Mask.Failed | .Ready) == 5);
    assert(G.check_status() == 5);
    assert(G.check_status(.Failed) == -7);
    print("ok\n");
}
