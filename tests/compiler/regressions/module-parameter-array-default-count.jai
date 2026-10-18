// An untyped `.[...]` default gives a `[$N] T` module parameter its count.
#import "Basic";
#import "Array_Param";

main :: () {
    assert(name_count() == 0);
    assert(flag_sum() == 6);
}
