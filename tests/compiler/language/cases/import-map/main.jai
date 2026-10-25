#import "Basic";

rename_exports :: (names: [] string) {
    for *names {
        if it.* == "public_alpha" it.* = "alpha_renamed";
        if it.* == "public_beta" it.* = "";
    }
}

using,map(rename_exports) Mapped :: #import "Facet_Pack";

main :: () {
    assert(alpha_renamed == 17, "a mapped export is visible under its new name");
    assert(Mapped.public_alpha == 17, "the namespace alias keeps the original names");
    print("ok\n");
}
