// Provenance: reviewed Default_Metaprogram limited path predicate, not OS normalization.
using #import "Basic";
D :: #import "Default_Metaprogram";

main :: () {
    assert(!D.is_absolute_path(""));
    assert(!D.is_absolute_path("relative/file"));
    assert(!D.is_absolute_path("x:"));
    assert(D.is_absolute_path("/file"));
    assert(D.is_absolute_path("\\file"));
    assert(D.is_absolute_path("1:x"));
    assert(D.is_absolute_path("x:relative"));
    assert(!D.verbose);
    assert(D.plugins.count == 0);
    print("ok\n");
}
