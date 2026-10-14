#import "Basic";

using,except(name_not_present) Remaining :: #import "Facet_Pack";

main :: () {
    assert(public_alpha == 17, "except preserves names when its filter is absent");
    assert(public_beta(6) == 8, "all other public names are exposed");
    assert(Remaining.public_alpha == 17, "qualified lookup remains available");
    print("ok\n");
}
