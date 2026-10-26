#import "Basic";
#import "Extensions/Variant";

main :: () {
    v: Variant(s64, string);

    print("%\n", unwrap(v, s32));
}
