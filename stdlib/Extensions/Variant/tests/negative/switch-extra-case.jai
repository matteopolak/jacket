// test: compile_fail
#import "Basic";
#import "Extensions/Variant";

main :: () {
    v: Variant(s64, string);

    variant_switch(v,
        (it: *s64) {},
        (it: *string) {},
        (it: *float64) {},
    );
}
