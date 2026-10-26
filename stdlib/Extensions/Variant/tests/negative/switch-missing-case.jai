#import "Basic";
#import "Extensions/Variant";

main :: () {
    v: Variant(s64, string, bool);

    variant_switch(v,
        (it: *s64) {},
        (it: *bool) {},
    );
}
