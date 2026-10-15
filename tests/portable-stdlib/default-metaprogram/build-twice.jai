// Independent ownership regression: shutdown must not retain freed plugin storage.
D :: #import "Default_Metaprogram";
using #import "Basic";

run_build_twice :: () {
    D.build();
    assert(D.plugins.count == 0 && D.plugins.data == null);
    D.build();
    assert(D.plugins.count == 0 && D.plugins.data == null);
}
