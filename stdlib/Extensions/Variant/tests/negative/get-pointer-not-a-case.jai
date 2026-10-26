#import "Basic";
#import "Extensions/Variant";

main :: () {
    v: Variant(s64, string);

    pointer := get_pointer(*v, u8);
}
