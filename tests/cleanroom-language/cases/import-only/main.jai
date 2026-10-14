#import "Basic";

using,only(public_alpha, public_beta) Picked :: #import "Facet_Pack";

main :: () {
    assert(public_alpha == 17, "only imports the selected name");
    assert(public_beta(5) == 7, "selected procedure is brought into scope");
    assert(Picked.public_alpha == 17, "named module alias remains available");
    print("ok\n");
}
