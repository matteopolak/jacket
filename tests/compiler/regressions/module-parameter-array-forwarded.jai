// A module forwards its `[$N] T` parameter, still at its `.[]` default, as an import argument.
#import "Basic";
#import "Array_Param_Forward";

main :: () {
    assert(forwarded_count() == 0);
}
